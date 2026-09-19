//! Forces a storage-sync failure through Databend's real vote persistence path
//! and verifies that current and later callers resolve instead of hanging.

use std::fs::File;
use std::io;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;

use chunked_wal::adversary;
use chunked_wal::adversary::BatchReceive;
use chunked_wal::adversary::Controller;
use chunked_wal::adversary::Event;
use databend_meta_raft_log::RaftLog;
use databend_meta_raft_log::RaftLogConfig;
use databend_meta_raft_log::RaftLogStore;
use openraft::Vote;
use openraft::storage::RaftLogStorage;

#[derive(Default)]
struct FailingSync {
    events: Mutex<Vec<Event>>,
    sync_calls: AtomicUsize,
}

impl Controller for FailingSync {
    fn checkpoint(&self, event: Event) {
        self.events.lock().unwrap().push(event);
    }

    fn batch_receive(&self, _items: usize) -> BatchReceive {
        BatchReceive::Timeout
    }

    fn sync_data(&self, file: &File) -> io::Result<()> {
        let call = self.sync_calls.fetch_add(1, Ordering::SeqCst);
        if call == 0 {
            Err(io::Error::new(
                io::ErrorKind::StorageFull,
                "injected sync failure",
            ))
        } else {
            file.sync_data()
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let root = std::env::var("ADVERSARY_TEST_DIR")?;
    let dir = tempfile::tempdir_in(root)?;
    let config = Arc::new(RaftLogConfig::new(dir.path().to_str().unwrap()));
    let raft_log = RaftLog::open(config)?;
    let controller = Arc::new(FailingSync::default());
    let controller_guard = adversary::install(controller.clone());
    let mut store = RaftLogStore::new(1, raft_log);

    let first = tokio::time::timeout(
        Duration::from_secs(2),
        store.save_vote(&Vote::new(7, 1)),
    )
    .await
    .expect("first save_vote hung");
    let first_error = first.expect_err("injected sync unexpectedly succeeded");
    assert_eq!(io::ErrorKind::StorageFull, first_error.kind());

    let idle_error = store
        .read()
        .await
        .wait_worker_idle()
        .expect_err("failed worker reported idle success");
    assert_eq!(io::ErrorKind::StorageFull, idle_error.kind());

    let second = tokio::time::timeout(
        Duration::from_secs(2),
        store.save_vote(&Vote::new(8, 1)),
    )
    .await
    .expect("later save_vote hung");
    let second_error = second.expect_err("later save_vote unexpectedly succeeded");

    let events = controller.events.lock().unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        Event::AfterSync {
            succeeded: false,
            ..
        }
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        Event::WorkerStopped { succeeded: false }
    )));

    println!(
        "{{\"first_error\":{:?},\"idle_error\":{:?},\"later_error\":{:?},\"sync_calls\":{},\"events\":{}}}",
        format!("{:?}", first_error.kind()),
        format!("{:?}", idle_error.kind()),
        format!("{:?}", second_error.kind()),
        controller.sync_calls.load(Ordering::SeqCst),
        events.len(),
    );

    drop(events);
    drop(store);
    drop(controller_guard);
    Ok(())
}

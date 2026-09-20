//! Same vote-sync injection and oracle as the original native failure probe.
//! The process watchdog replaces its Tokio-only timeout in every backend.
use std::fs::File;
use std::io;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};

use chunked_wal::adversary::{self, BatchReceive, Controller, Event};
use databend_meta_raft_log::{RaftLog, RaftLogConfig, RaftLogStore};
use openraft::storage::RaftLogStorage;
use openraft::Vote;

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
        if self.sync_calls.fetch_add(1, Ordering::SeqCst) == 0 {
            Err(io::Error::new(io::ErrorKind::StorageFull, "injected sync failure"))
        } else {
            file.sync_data()
        }
    }
}

pub async fn run() -> anyhow::Result<()> {
    let dir = tempfile::tempdir_in(std::env::var("ADVERSARY_TEST_DIR")?)?;
    let config = Arc::new(RaftLogConfig::new(dir.path().to_str().unwrap()));
    let raft_log = RaftLog::open(config)?;
    let controller = Arc::new(FailingSync::default());
    let guard = adversary::install(controller.clone());
    let mut store = RaftLogStore::new(1, raft_log);

    let first = store.save_vote(&Vote::new(7, 1)).await.expect_err("sync unexpectedly succeeded");
    assert_eq!(first.kind(), io::ErrorKind::StorageFull);
    let idle = store.read().await.wait_worker_idle().expect_err("failed worker reported success");
    assert_eq!(idle.kind(), io::ErrorKind::StorageFull);
    let later = store.save_vote(&Vote::new(8, 1)).await.expect_err("later vote unexpectedly succeeded");
    let events = controller.events.lock().unwrap();
    assert!(events.iter().any(|event| matches!(event, Event::AfterSync { succeeded: false, .. })));
    assert!(events.iter().any(|event| matches!(event, Event::WorkerStopped { succeeded: false })));
    if let Ok(trace_dir) = std::env::var("ADVERSARY_TRACE_DIR") {
        std::fs::create_dir_all(&trace_dir)?;
        std::fs::write(format!("{trace_dir}/sync-failure.txt"),
                       events.iter().map(|event| format!("{event:?}\n")).collect::<String>())?;
    }
    println!("{{\"first_error\":{:?},\"idle_error\":{:?},\"later_error\":{:?},\"sync_calls\":{},\"events\":{}}}",
             format!("{:?}", first.kind()), format!("{:?}", idle.kind()), format!("{:?}", later.kind()),
             controller.sync_calls.load(Ordering::SeqCst), events.len());
    drop(events);
    drop(store);
    drop(guard);
    Ok(())
}

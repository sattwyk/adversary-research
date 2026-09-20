//! Frozen workload copied from baseline 7a53b55; only the entry point is factored.
//! Runs Databend's production Raft log and native WAL worker under a fixed
//! semantic schedule. The controller changes neither record encoding nor the
//! worker's queue, batching, short-write loop, callbacks, rotation, or recovery.

use std::fs::File;
use std::io;
use std::io::IoSlice;
use std::io::Write;
use std::sync::Arc;
use std::sync::Condvar;
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
use databend_meta_types::Cmd;
use databend_meta_types::LogEntry;
use databend_meta_types::UpsertKV;
use databend_meta_types::raft_types::Entry;
use databend_meta_types::raft_types::EntryPayload;
use databend_meta_types::raft_types::IOFlushed;
use databend_meta_types::raft_types::TypeConfig;
use databend_meta_types::raft_types::new_log_id;
use openraft::RaftLogReader;
use openraft::storage::RaftLogStorage;
use openraft::type_config::TypeConfigExt;

#[derive(Default)]
struct ScheduleState {
    sent: u64,
    completed: u64,
    producer_done: bool,
    worker_events: Vec<Event>,
    batches: Vec<usize>,
}

struct FixedController {
    state: Mutex<ScheduleState>,
    changed: Condvar,
    batch_items: usize,
    short_write_max: usize,
    write_calls: AtomicUsize,
    sync_calls: AtomicUsize,
}

impl FixedController {
    fn new(batch_items: usize, short_write_max: usize) -> Self {
        Self {
            state: Mutex::new(ScheduleState::default()),
            changed: Condvar::new(),
            batch_items,
            short_write_max,
            write_calls: AtomicUsize::new(0),
            sync_calls: AtomicUsize::new(0),
        }
    }

    fn producer_done(&self) {
        let mut state = self.state.lock().unwrap();
        state.producer_done = true;
        self.changed.notify_all();
    }

    fn observations(&self) -> (Vec<usize>, Vec<Event>, usize, usize) {
        let state = self.state.lock().unwrap();
        (
            state.batches.clone(),
            state.worker_events.clone(),
            self.write_calls.load(Ordering::SeqCst),
            self.sync_calls.load(Ordering::SeqCst),
        )
    }
}

impl Controller for FixedController {
    fn checkpoint(&self, event: Event) {
        let mut state = self.state.lock().unwrap();

        if event == Event::BeforeFirstReceive {
            while !state.producer_done
                && state.sent < state.completed + self.batch_items as u64
            {
                state = self.changed.wait(state).unwrap();
            }
        }

        match &event {
            Event::AfterSend {
                seq,
                accepted: true,
            } => {
                state.sent = state.sent.max(*seq);
                self.changed.notify_all();
            }
            Event::AfterComplete { seq } => {
                state.completed = state.completed.max(*seq);
                self.changed.notify_all();
            }
            Event::BeforeWrite { items, .. } => state.batches.push(*items),
            _ => {}
        }

        if !matches!(
            event,
            Event::BeforeSend { .. }
                | Event::AfterReserve { .. }
                | Event::AfterSend { .. }
        ) {
            state.worker_events.push(event);
        }
    }

    fn batch_receive(&self, items: usize) -> BatchReceive {
        let state = self.state.lock().unwrap();
        let input_exhausted = state.producer_done
            && state.sent <= state.completed + items as u64;
        if items >= self.batch_items || input_exhausted {
            BatchReceive::Timeout
        } else {
            BatchReceive::Native
        }
    }

    fn write_vectored(
        &self,
        file: &File,
        bufs: &[IoSlice<'_>],
    ) -> io::Result<usize> {
        self.write_calls.fetch_add(1, Ordering::SeqCst);
        let mut remaining = self.short_write_max;
        let mut limited = Vec::new();
        for buf in bufs {
            if remaining == 0 {
                break;
            }
            let take = remaining.min(buf.len());
            limited.push(IoSlice::new(&buf[..take]));
            remaining -= take;
        }
        let mut file = file;
        file.write_vectored(&limited)
    }

    fn sync_data(&self, file: &File) -> io::Result<()> {
        self.sync_calls.fetch_add(1, Ordering::SeqCst);
        file.sync_data()
    }
}

pub async fn run() -> anyhow::Result<()> {
    let root = std::env::var("ADVERSARY_TEST_DIR")?;
    let trials: usize = std::env::var("ADVERSARY_TRIALS")
        .unwrap_or_else(|_| "10".to_string())
        .parse()?;
    let count = 64u64;
    let mut reference: Option<(Vec<usize>, Vec<Event>, usize, usize)> = None;

    for trial in 0..trials {
        let controller = Arc::new(FixedController::new(4, 137));
        let controller_guard = adversary::install(controller.clone());
        let dir = tempfile::tempdir_in(&root)?;
        let mut config = RaftLogConfig::new(dir.path().to_str().unwrap());
        config.wal.read_buffer_size = Some(64 * 1024);
        config.wal.flush_batch_wait = Some(Duration::from_secs(30));
        config.wal.flush_batch_max_items = Some(32);
        config.wal.chunk_max_records = Some(17);
        config.wal.flush_queue_max_bytes = Some(256 * 1024);
        let config = Arc::new(config);
        let mut store = RaftLogStore::new(1, RaftLog::open(config.clone())?);
        let mut expected = Vec::new();
        let mut waiters = Vec::new();

        for i in 0..count {
            let value = vec![(i % 251) as u8; 4096];
            let entry = Entry {
                log_id: new_log_id(1, 1, i),
                payload: EntryPayload::Normal(LogEntry::new_with_time(
                    Cmd::UpsertKV(UpsertKV::update(
                        format!("controlled/{i}"),
                        &value,
                    )),
                    Some(1_000_000),
                )),
            };
            expected.push(entry.clone());
            let (tx, rx) = TypeConfig::oneshot();
            store.append([entry], IOFlushed::signal(tx)).await?;
            if i % 11 == 0 {
                drop(rx);
            } else {
                waiters.push(rx);
            }
        }
        controller.producer_done();

        for rx in waiters {
            rx.await??;
        }
        store.read().await.wait_worker_idle()?;
        drop(store);

        let mut recovered = RaftLogStore::new(1, RaftLog::open(config)?);
        let actual = recovered.try_get_log_entries(0..count).await?;
        assert_eq!(actual, expected, "real codec/recovery changed the entries");
        drop(recovered);

        let observed = controller.observations();
        if let Ok(trace_dir) = std::env::var("ADVERSARY_TRACE_DIR") {
            std::fs::create_dir_all(&trace_dir)?;
            let trace = observed.1.iter().map(|event| format!("{event:?}\n")).collect::<String>();
            std::fs::write(format!("{trace_dir}/trial-{trial:03}.txt"), trace)?;
        }
        if let Some(expected_observation) = &reference {
            assert_eq!(
                &observed, expected_observation,
                "controlled worker trace did not replay"
            );
        } else {
            reference = Some(observed.clone());
        }

        println!(
            "{{\"trial\":{trial},\"entries\":{count},\"recovered_equal\":true,\"batches\":{:?},\"worker_events\":{},\"short_write_calls\":{},\"sync_calls\":{}}}",
            observed.0,
            observed.1.len(),
            observed.2,
            observed.3,
        );
        drop(controller_guard);
    }

    Ok(())
}

//! Local reliability experiment. No production algorithms are replaced.
//! Measures whether fixed input reproduces native worker batch boundaries.
use databend_meta_raft_log::{RaftLog, RaftLogConfig, RaftLogStore};
use databend_meta_types::raft_types::{Entry, EntryPayload, IOFlushed, TypeConfig, new_log_id};
use databend_meta_types::{Cmd, LogEntry, UpsertKV};
use openraft::type_config::TypeConfigExt;
use openraft::{RaftLogReader, storage::RaftLogStorage};
use std::sync::{Arc, Mutex};
use std::time::Duration;

// The batch sizes are observed from upstream diagnostic messages. This logger
// is not a stable tracing ABI and does not prove worker scheduling control.
struct BatchLog(Mutex<Vec<usize>>);
static LOG: BatchLog = BatchLog(Mutex::new(Vec::new()));
impl log::Log for BatchLog {
    fn enabled(&self, m: &log::Metadata) -> bool {
        m.target().contains("flush_worker") && m.level() <= log::Level::Debug
    }
    fn log(&self, r: &log::Record) {
        if self.enabled(r.metadata()) {
            if let Some(n) = r.args().to_string().strip_prefix("batched write: ") {
                LOG.0.lock().unwrap().push(n.parse().unwrap());
            }
        }
    }
    fn flush(&self) {}
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    log::set_logger(&LOG).unwrap();
    log::set_max_level(log::LevelFilter::Debug);
    let root = std::env::var("ADVERSARY_TEST_DIR")?;
    let trials: usize = std::env::var("ADVERSARY_TRIALS")
        .unwrap_or("20".into())
        .parse()?;
    let count = 256u64;
    for trial in 0..trials {
        let dir = tempfile::tempdir_in(&root)?;
        let mut config = RaftLogConfig::new(dir.path().to_str().unwrap());
        config.wal.read_buffer_size = Some(64 * 1024);
        config.wal.flush_batch_wait = Some(Duration::from_millis(1));
        config.wal.flush_batch_max_items = Some(32);
        config.wal.chunk_max_records = Some(71);
        config.wal.flush_queue_max_bytes = Some(256 * 1024);
        let config = Arc::new(config);
        let mut store = RaftLogStore::new(1, RaftLog::open(config.clone())?);
        LOG.0.lock().unwrap().clear();
        let mut expected = Vec::new();
        let mut waiters = Vec::new();
        let mut abandoned_waiters = 0;
        for i in 0..count {
            // Fixed bytes, indices and order; no entropy or host sleeps in workload.
            let value = vec![(i % 251) as u8; 4096];
            let entry = Entry {
                log_id: new_log_id(1, 1, i),
                payload: EntryPayload::Normal(LogEntry::new_with_time(
                    Cmd::UpsertKV(UpsertKV::update(format!("probe/{i}"), &value)),
                    Some(1_000_000),
                )),
            };
            expected.push(entry.clone());
            let (tx, rx) = TypeConfig::oneshot();
            store.append([entry], IOFlushed::signal(tx)).await?;
            // Some callers stop waiting; this must not withdraw the queued write.
            if i % 11 == 0 {
                abandoned_waiters += 1;
                drop(rx);
            } else {
                waiters.push(rx);
            }
        }
        for rx in waiters {
            rx.await??;
        }
        store.read().await.wait_worker_idle()?;
        let batches = LOG.0.lock().unwrap().clone();
        drop(store); // Graceful close; explicitly NOT a process or power crash.
        let mut recovered = RaftLogStore::new(1, RaftLog::open(config)?);
        let actual = recovered.try_get_log_entries(0..count).await?;
        assert_eq!(actual, expected, "real codec/recovery changed the entries");
        println!(
            "{{\"trial\":{trial},\"entries\":{count},\"abandoned_waiters\":{abandoned_waiters},\"recovered_equal\":true,\"batches\":{batches:?}}}"
        );
        drop(recovered);
    }
    Ok(())
}

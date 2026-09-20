//! Historical raft-log reader test, not a model of the WAL or file cursor.
//! Setup and reads use the historical implementation and a real Linux file.
use std::cell::Cell;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::api::raft_log_writer::{RaftLogWriter, blocking_flush};
use crate::tests::context::TestContext;

thread_local! { static CONTROLLED_READER: Cell<bool> = const { Cell::new(false) }; }
static LEVEL: AtomicUsize = AtomicUsize::new(0);

/// The sole added production call site: after seek on the parent, immediately
/// before pread on the fixed revision. Setup/recovery reads are not scheduled.
pub(crate) fn checkpoint() {
    if LEVEL.load(Ordering::Relaxed) >= 2 && CONTROLLED_READER.get() {
        shuttle::thread::yield_now();
    }
}

#[test]
fn adversary_reader_boundary() {
    let level = std::env::var("READER_LEVEL")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    LEVEL.store(level, Ordering::Relaxed);
    let executions = Arc::new(AtomicUsize::new(0));
    let executions_in_test = executions.clone();
    let outcome = std::panic::catch_unwind(move || {
        let test_case = move || {
            // Every explored execution starts from a fresh actual WAL. Its
            // native flush worker runs during setup, then becomes idle.
            let mut ctx = TestContext::new().unwrap();
            ctx.config.chunk_max_records = Some(20);
            ctx.config.log_cache_capacity = Some(0);
            let mut writer = ctx.new_raft_log().unwrap();
            for i in 0..40 {
                writer
                    .append([((1, i), format!("payload_{i:04}"))])
                    .unwrap();
            }
            blocking_flush(&mut writer).unwrap();
            drop(writer);
            let log = Arc::new(ctx.new_raft_log().unwrap());
            let mut handles = Vec::new();
            for index in 0..2u64 {
                let log = log.clone();
                handles.push(shuttle::thread::spawn(move || {
                    CONTROLLED_READER.set(true);
                    // Level 1 only schedules at the caller synchronization
                    // boundary. No lock exists inside the faulty seek/read.
                    if level >= 1 {
                        shuttle::thread::yield_now();
                    }
                    let read = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let entries = log.read(index, index + 1).collect::<Result<Vec<_>, _>>();
                        match entries {
                            Ok(entries) => {
                                entries == vec![((1, index), format!("payload_{index:04}"))]
                            }
                            Err(_) => false,
                        }
                    }));
                    CONTROLLED_READER.set(false);
                    read.unwrap_or(false)
                }));
            }
            let valid = handles
                .into_iter()
                .map(|h| h.join().unwrap())
                .collect::<Vec<_>>();
            let iteration = executions_in_test.fetch_add(1, Ordering::Relaxed);
            println!("READER {{\"execution\":{iteration},\"level\":{level},\"valid\":{valid:?}}}");
            assert!(
                valid.iter().all(|valid| *valid),
                "historical shared-cursor race"
            );
        };
        if let Ok(schedule) = std::env::var("READER_REPLAY") {
            shuttle::replay(test_case, &schedule);
        } else {
            shuttle::check_dfs(test_case, Some(1000));
        }
    });
    let found = outcome.is_err();
    println!(
        "READER_SUMMARY {{\"level\":{level},\"executions\":{},\"found\":{found}}}",
        executions.load(Ordering::Relaxed)
    );
    assert_eq!(found, std::env::var("EXPECT_RACE").unwrap() == "1");
}

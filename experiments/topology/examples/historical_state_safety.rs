//! Public-API reproducer for raft-log state-safety regressions.
//!
//! Run against both the fixing revision and its first parent. EXPECT_REJECT
//! states whether the operation is expected to be rejected by that revision.

use std::io;
use std::sync::Arc;
use std::sync::mpsc::SyncSender;

use raft_log::Config;
use raft_log::RaftLog;
use raft_log::api::raft_log_writer::RaftLogWriter;
use raft_log::api::types::Types;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ProbeTypes;

impl Types for ProbeTypes {
    type LogId = (u64, u64);
    type LogPayload = String;
    type Vote = (u64, u64);
    type UserData = String;
    type Callback = SyncSender<io::Result<()>>;

    fn log_index(log_id: &Self::LogId) -> u64 {
        log_id.1
    }

    fn payload_size(payload: &Self::LogPayload) -> u64 {
        payload.len() as u64
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scenario = std::env::var("HISTORICAL_SCENARIO")?;
    let expect_reject = std::env::var("EXPECT_REJECT")? == "1";
    let temp_dir = tempfile::tempdir()?;
    let config = Arc::new(Config::new(temp_dir.path().to_str().unwrap()));
    let mut log = RaftLog::<ProbeTypes>::open(config)?;

    let result = match scenario.as_str() {
        "commit-beyond-last" => log.commit((1, 100)).map(|_| ()),
        "truncate-committed" => {
            log.append([
                ((1, 0), "zero".to_string()),
                ((1, 1), "one".to_string()),
                ((1, 2), "two".to_string()),
            ])?;
            log.commit((1, 1))?;
            log.truncate(1).map(|_| ())
        }
        "purge-conflicting-id" => {
            log.append([
                ((1, 0), "zero".to_string()),
                ((1, 1), "one".to_string()),
                ((2, 2), "two".to_string()),
                ((2, 3), "three".to_string()),
            ])?;
            log.purge((3, 1)).map(|_| ())
        }
        other => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown scenario: {other}"),
            )
            .into());
        }
    };

    let rejected = result.is_err();
    let error = result.err().map(|err| err.to_string());
    assert_eq!(
        rejected, expect_reject,
        "unexpected historical behavior for {scenario}: {error:?}"
    );
    println!(
        "{{\"scenario\":\"{scenario}\",\"rejected\":{rejected},\"last\":{:?},\"committed\":{:?},\"purged\":{:?},\"error\":{:?}}}",
        format!("{:?}", log.log_state().last()),
        format!("{:?}", log.log_state().committed()),
        format!("{:?}", log.log_state().purged()),
        error.unwrap_or_default(),
    );
    Ok(())
}

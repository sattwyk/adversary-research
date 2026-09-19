//! Public-API reproducer for the no-sync callback regression fixed in 5a71534.

use std::io;
use std::sync::Arc;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::sync_channel;
use std::time::Duration;

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
    let expect_callback = std::env::var("EXPECT_CALLBACK")? == "1";
    let temp_dir = tempfile::tempdir()?;
    let config = Arc::new(Config::new(temp_dir.path().to_str().unwrap()));
    let mut log = RaftLog::<ProbeTypes>::open(config)?;
    log.append([((1, 0), "one".to_string())])?;

    let (tx, rx) = sync_channel(1);
    log.flush(false, Some(tx))?;
    let observed_callback = match rx.recv_timeout(Duration::from_millis(250)) {
        Ok(result) => {
            result?;
            true
        }
        Err(RecvTimeoutError::Timeout) => false,
        Err(RecvTimeoutError::Disconnected) => false,
    };
    assert_eq!(observed_callback, expect_callback);
    println!(
        "{{\"callback_observed\":{observed_callback},\"expected\":{expect_callback}}}"
    );
    Ok(())
}

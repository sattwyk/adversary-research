//! Native comparison for stack preservation and cancellation cleanup.
//!
//! Coroutines yield synchronous Write calls to this driver. Ordinary coroutine
//! Drop unwinds application state, so it must not be used as a process-kill model.

use corosensei::{Coroutine, CoroutineResult, Yielder};
use std::{
    io::{self, Write},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
#[derive(Debug, PartialEq)]
enum Request {
    Write(Vec<u8>),
    Sync,
}
struct SimulatedFile<'context>(&'context Yielder<usize, Request>);
impl Write for SimulatedFile<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        Ok(self.0.suspend(Request::Write(bytes.to_vec())))
    }
    // This models a durability barrier; std::io::Write::flush is NOT fsync.
    fn flush(&mut self) -> io::Result<()> {
        self.0.suspend(Request::Sync);
        Ok(())
    }
}
// Ordinary nested synchronous code, including std::io::Write::write_all.
fn commit(file: &mut impl Write) -> io::Result<u64> {
    let canary = 123456;
    file.write_all(b"payload")?;
    file.flush()?;
    Ok(canary + 7)
}
fn nested(file: &mut impl Write) -> io::Result<u64> {
    commit(file)
}
struct GuestCleanupMarker(Arc<AtomicUsize>);
impl Drop for GuestCleanupMarker {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
fn create_coroutine(dropped: Arc<AtomicUsize>) -> Coroutine<usize, Request, u64> {
    Coroutine::new(move |yielder, _| {
        let _marker = GuestCleanupMarker(dropped);
        nested(&mut SimulatedFile(yielder)).unwrap()
    })
}
fn main() {
    for _ in 0..100 {
        let drops = Arc::new(AtomicUsize::new(0));
        let mut first = create_coroutine(drops.clone());
        let mut second = create_coroutine(drops.clone());
        assert_eq!(
            first.resume(0),
            CoroutineResult::Yield(Request::Write(b"payload".to_vec()))
        );
        assert_eq!(
            second.resume(0),
            CoroutineResult::Yield(Request::Write(b"payload".to_vec()))
        );
        assert_eq!(
            first.resume(3),
            CoroutineResult::Yield(Request::Write(b"load".to_vec()))
        );
        assert_eq!(first.resume(4), CoroutineResult::Yield(Request::Sync));
        assert_eq!(first.resume(0), CoroutineResult::Return(123463));
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        // Dropping the second suspended coroutine executes application destructors.
        // This is cancellation/cleanup, not abrupt process crash semantics.
        drop(second);
        assert_eq!(drops.load(Ordering::SeqCst), 2);
    }
    println!(
        "PASS: 100 native fiber schedules preserve nested synchronous write_all and stack locals."
    );
    println!(
        "PASS: dropping a suspended native fiber executes the application Drop marker. It is not an abrupt process kill."
    );
}

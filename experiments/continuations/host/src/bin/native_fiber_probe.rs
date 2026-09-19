use corosensei::{Coroutine,CoroutineResult,Yielder};
use std::{io::{self,Write},sync::{Arc,atomic::{AtomicUsize,Ordering}}};
#[derive(Debug,PartialEq)]
enum Request { Write(Vec<u8>), Sync }
struct File<'a>(&'a Yielder<usize,Request>);
impl Write for File<'_> {
    fn write(&mut self,b:&[u8])->io::Result<usize> {Ok(self.0.suspend(Request::Write(b.to_vec())))}
    fn flush(&mut self)->io::Result<()> {self.0.suspend(Request::Sync);Ok(())}
}
// Ordinary nested synchronous code, including std::io::Write::write_all.
fn commit(file:&mut impl Write)->io::Result<u64> {
    let canary=123456;
    file.write_all(b"payload")?;
    file.flush()?;
    Ok(canary+7)
}
fn nested(file:&mut impl Write)->io::Result<u64> {commit(file)}
struct Marker(Arc<AtomicUsize>);
impl Drop for Marker {fn drop(&mut self){self.0.fetch_add(1,Ordering::SeqCst);}}
fn task(dropped:Arc<AtomicUsize>)->Coroutine<usize,Request,u64> {
    Coroutine::new(move |y,_| {
        let _marker=Marker(dropped);
        nested(&mut File(y)).unwrap()
    })
}
fn main() {
    for _ in 0..100 {
        let drops=Arc::new(AtomicUsize::new(0));
        let mut a=task(drops.clone());let mut b=task(drops.clone());
        assert_eq!(a.resume(0),CoroutineResult::Yield(Request::Write(b"payload".to_vec())));
        assert_eq!(b.resume(0),CoroutineResult::Yield(Request::Write(b"payload".to_vec())));
        assert_eq!(a.resume(3),CoroutineResult::Yield(Request::Write(b"load".to_vec())));
        assert_eq!(a.resume(4),CoroutineResult::Yield(Request::Sync));
        assert_eq!(a.resume(0),CoroutineResult::Return(123463));
        assert_eq!(drops.load(Ordering::SeqCst),1);
        // Dropping a native suspended coroutine executes application destructors.
        // This is cancellation/cleanup, not abrupt process crash semantics.
        drop(b);
        assert_eq!(drops.load(Ordering::SeqCst),2);
    }
    println!("PASS: 100 native fiber schedules preserve nested synchronous write_all and stack locals.");
    println!("PASS: dropping a suspended native fiber executes the application Drop marker. It is not an abrupt process kill.");
}

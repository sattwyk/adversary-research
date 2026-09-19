use std::{collections::BTreeMap, future::{Future,poll_fn}, pin::Pin, sync::{Arc,Mutex}, task::{Context,Poll,Waker}};
use wasmtime::{Caller,Engine,Linker,Module,Result,Store};
#[derive(Clone,Debug)]
enum Op { Len, Read(u64,usize), Write(u64,Vec<u8>), Resize(u64), Sync }
#[derive(Default)]
struct World { volatile:Vec<u8>, durable:Vec<u8>, phase:u32, next:u64,
    pending:BTreeMap<u64,Op>, ready:BTreeMap<u64,u64>, trace:Vec<String>, commit_writes:usize, guest_drops:usize }
type Shared=Arc<Mutex<World>>;
async fn submit(w:Shared,op:Op)->Result<u64> {
    let id={let mut w=w.lock().unwrap();let id=w.next;w.next+=1;
        let label=match &op {Op::Write(o,b)=>format!("Write({o},{})",b.len()),x=>format!("{x:?}")};
        let phase=w.phase; w.trace.push(format!("p{phase} r{id} {label}"));
        w.pending.insert(id,op);id};
    Ok(poll_fn(|_|{let mut w=w.lock().unwrap();match w.ready.remove(&id) {
        Some(n)=>{w.pending.remove(&id);Poll::Ready(n)},None=>Poll::Pending}}).await)
}
fn memory(c:&mut Caller<'_,Shared>)->wasmtime::Memory { c.get_export("memory").unwrap().into_memory().unwrap() }
type Running=Pin<Box<dyn Future<Output=Result<u64>>>>;
fn start(engine:&Engine,module:&Module,w:Shared,entry:&str)->Result<Running> {
    let mut linker=Linker::new(engine);
    linker.func_wrap_async("sim","storage_len",|c:Caller<'_,Shared>,():()|Box::new(submit(c.data().clone(),Op::Len)))?;
    linker.func_wrap_async("sim","storage_set_len",|c:Caller<'_,Shared>,(n,):(u64,)|Box::new(async move {
        submit(c.data().clone(),Op::Resize(n)).await?;Ok(0i32)
    }))?;
    linker.func_wrap_async("sim","storage_sync",|c:Caller<'_,Shared>,():()|Box::new(async move {
        submit(c.data().clone(),Op::Sync).await?;Ok(0i32)
    }))?;
    linker.func_wrap_async("sim","storage_write",|mut c:Caller<'_,Shared>,(offset,ptr,len):(u64,u32,u32)|Box::new(async move {
        let mut bytes=vec![0;len as usize]; memory(&mut c).read(&c,ptr as usize,&mut bytes)?;
        submit(c.data().clone(),Op::Write(offset,bytes)).await?;Ok(0i32)
    }))?;
    linker.func_wrap_async("sim","storage_read",|mut c:Caller<'_,Shared>,(offset,ptr,len):(u64,u32,u32)|Box::new(async move {
        submit(c.data().clone(),Op::Read(offset,len as usize)).await?;
        let bytes=c.data().lock().unwrap().volatile[offset as usize..offset as usize+len as usize].to_vec();
        memory(&mut c).write(&mut c,ptr as usize,&bytes)?;Ok(0i32)
    }))?;
    linker.func_wrap("sim","phase",|c:Caller<'_,Shared>,p:u32| {c.data().lock().unwrap().phase=p;})?;
    linker.func_wrap("sim","cleanup",|c:Caller<'_,Shared>| {c.data().lock().unwrap().guest_drops+=1;})?;
    let engine=engine.clone();let module=module.clone();let entry=entry.to_owned();
    Ok(Box::pin(async move {
        let mut store=Store::new(&engine,w);
        let instance=linker.instantiate_async(&mut store,&module).await?;
        instance.get_typed_func::<(),u64>(&mut store,&entry)?.call_async(&mut store,()).await
    }))
}
fn drive(mut f:Running,w:Shared,kill_write:Option<usize>,persist_partial:bool)->Result<Option<u64>> {
    loop {
        match f.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
            Poll::Ready(r)=>return r.map(Some),Poll::Pending=>{}
        }
        let mut m=w.lock().unwrap();
        let (&id,op)=m.pending.iter().next().expect("pending import");let op=op.clone();
        let should_kill=if matches!(&op,Op::Write(..)) && m.phase==2 {
            m.commit_writes+=1;kill_write==Some(m.commit_writes)
        } else { matches!(&op,Op::Sync) && m.phase==2 && kill_write==Some(usize::MAX) };
        let n=match op {
            Op::Len=>m.volatile.len() as u64,
            Op::Read(offset,len)=>{assert!(offset as usize+len<=m.volatile.len());0},
            Op::Resize(n)=>{m.volatile.resize(n as usize,0);0},
            Op::Sync=>{m.durable=m.volatile.clone();0},
            Op::Write(offset,bytes)=>{
                let len=if should_kill {bytes.len()/2} else {bytes.len()};
                m.volatile[offset as usize..offset as usize+len].copy_from_slice(&bytes[..len]);
                if should_kill && persist_partial {
                    // Persist only this changed prefix, not all earlier unsynced writes.
                    if m.durable.len()<offset as usize+len {m.durable.resize(offset as usize+len,0);}
                    m.durable[offset as usize..offset as usize+len].copy_from_slice(&bytes[..len]);
                }
                0
            }
        };
        if should_kill {
            m.trace.push(format!("CRASH during operation {id}; persisted_prefix={persist_partial}"));
            drop(m);drop(f); // no application error return, no guest cleanup
            let mut m=w.lock().unwrap();m.pending.clear();m.ready.clear();
            m.volatile=m.durable.clone();m.phase=0;
            return Ok(None)
        }
        m.ready.insert(id,n);drop(m);
    }
}
fn scenario(engine:&Engine,module:&Module,kill:Option<usize>,persist:bool)->Result<(Vec<String>,usize)> {
    let w=Arc::new(Mutex::new(World::default()));
    let run=drive(start(engine,module,w.clone(),"run")?,w.clone(),kill,persist)?;
    if kill.is_none() {assert_eq!(run,Some(42));}
    else {assert!(run.is_none());}
    assert_eq!(w.lock().unwrap().guest_drops,if kill.is_none(){1}else{0});
    let writes=w.lock().unwrap().commit_writes;
    {let mut m=w.lock().unwrap();m.volatile=m.durable.clone();m.phase=0;}
    let recovered=drive(start(engine,module,w.clone(),"recover")?,w.clone(),None,false)?.unwrap();
    assert!(recovered==0 || recovered==42);
    if kill.is_none(){assert_eq!(recovered,42);}
    if kill==Some(usize::MAX){assert_eq!(recovered,42);}
    let mut m=w.lock().unwrap();m.trace.push(format!("recovered={recovered}"));
    Ok((m.trace.clone(),writes))
}
fn main()->Result<()> {
    let path=std::env::args().nth(1).expect("guest wasm path");
    let engine=Engine::default();let module=Module::from_file(&engine,path)?;
    let (baseline,writes)=scenario(&engine,&module,None,false)?;
    println!("Unmodified redb 4.1.0: commit+read=42, power-loss reopen=42; {writes} backend writes during commit.");
    println!("Baseline import count: {}",baseline.len()-1);
    for k in 1..=writes {for persist in [false,true] {
        let (reference,_)=scenario(&engine,&module,Some(k),persist)?;
        for _ in 0..9 {assert_eq!(reference,scenario(&engine,&module,Some(k),persist)?.0);}
        println!("PASS commit write {k}: half effect, persist={persist}, kill suspended guest; {}; 10 identical traces",reference.last().unwrap());
    }}
    let (sync_trace,_)=scenario(&engine,&module,Some(usize::MAX),false)?;
    for _ in 0..9 {assert_eq!(sync_trace,scenario(&engine,&module,Some(usize::MAX),false)?.0);}
    println!("PASS: sync effect persisted, killed before completion returned, recovered=42; 10 identical traces.");
    println!("PASS: guest Drop marker ran on normal completion and never on suspended-call kill.");
    println!("Scope: real synchronous redb engine, single caller, simple abstract disk. No native concurrency/OS conformance or bug-discovery claim.");
    Ok(())
}

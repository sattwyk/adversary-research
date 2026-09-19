use wasmtime::Result;
use std::{collections::BTreeMap, future::{Future, poll_fn}, pin::Pin,
    sync::{Arc, Mutex}, task::{Context, Poll, Waker}};
use wasmtime::{Engine, Store};
use wasmtime::component::{Component, Linker};

// A component with synchronous canonical imports and live nested call frames.
const GUEST: &str = r#"(component
 (import "write" (func $write (param "offset" u32) (param "length" u32) (result u32)))
 (import "sync" (func $sync (result u32)))
 (core module $m
  (import "io" "write" (func $write (param i32 i32) (result i32)))
  (import "io" "sync" (func $sync (result i32)))
  (func $write_all (param $length i32) (result i32) (local $offset i32)
   (loop $again
    (local.set $offset (i32.add (local.get $offset)
      (call $write (local.get $offset) (i32.sub (local.get $length) (local.get $offset)))))
    (br_if $again (i32.lt_u (local.get $offset) (local.get $length))))
   (local.get $offset))
  (func $commit (param $canary i32) (result i32) (local $written i32)
   (local.set $written (call $write_all (i32.const 7)))
   (if (i32.ne (call $sync) (i32.const 0)) (then unreachable))
   (i32.add (local.get $canary) (local.get $written)))
  (func $nested (param $canary i32) (result i32)
   (call $commit (local.get $canary)))
  (func (export "run") (result i32) (call $nested (i32.const 123456))))
 (core func $w (canon lower (func $write)))
 (core func $s (canon lower (func $sync)))
 (core instance $i (instantiate $m (with "io" (instance
   (export "write" (func $w)) (export "sync" (func $s))))))
 (func (export "run") (result u32) (canon lift (core func $i "run"))))"#;

#[derive(Clone, Debug)]
enum Kind { Write { offset: usize, length: usize }, Sync }
#[derive(Clone, Debug)]
struct Request { node: usize, kind: Kind }
#[derive(Default)]
struct World {
    next: u64, pending: BTreeMap<u64, Request>, ready: BTreeMap<u64, u32>,
    volatile: [Vec<u8>; 2], durable: [Vec<u8>; 2], trace: Vec<String>, host_drops: usize,
}
type Shared = Arc<Mutex<World>>;
#[derive(Clone)]
struct Node { id: usize, world: Shared }
struct HostGuard(Shared);
impl Drop for HostGuard {
    fn drop(&mut self) { self.0.lock().unwrap().host_drops += 1; }
}
async fn request(node: Node, kind: Kind) -> Result<(u32,)> {
    let _guard = HostGuard(node.world.clone());
    let id = {
        let mut w = node.world.lock().unwrap();
        let id = w.next; w.next += 1;
        w.trace.push(format!("submit n{} r{} {:?}", node.id, id, kind));
        w.pending.insert(id, Request { node: node.id, kind }); id
    };
    let result = poll_fn(|_cx| {
        let mut w = node.world.lock().unwrap();
        match w.ready.remove(&id) {
            Some(n) => { w.pending.remove(&id); Poll::Ready(n) }
            None => Poll::Pending,
        }
    }).await;
    node.world.lock().unwrap().trace.push(format!("return r{id} {result}"));
    Ok((result,))
}
type Running = Pin<Box<dyn Future<Output = Result<u32>>>>;
fn node(engine: Engine, component: Component, id: usize, world: Shared) -> Result<Running> {
    let mut linker = Linker::<Node>::new(&engine);
    linker.root().func_wrap_async("write", |cx, (offset, length): (u32,u32)| {
        Box::new(request(cx.data().clone(), Kind::Write { offset: offset as usize, length: length as usize }))
    })?;
    linker.root().func_wrap_async("sync", |cx, (): ()| {
        Box::new(request(cx.data().clone(), Kind::Sync))
    })?;
    Ok(Box::pin(async move {
        let mut store = Store::new(&engine, Node { id, world: world.clone() });
        let instance = linker.instantiate_async(&mut store, &component).await?;
        let run = instance.get_typed_func::<(),(u32,)>(&mut store,"run")?;
        let (n,) = run.call_async(&mut store,()).await?;
        world.lock().unwrap().trace.push(format!("published n{id} {n}"));
        Ok(n)
    }))
}
fn poll(f: &mut Running) -> Poll<Result<u32>> {
    f.as_mut().poll(&mut Context::from_waker(Waker::noop()))
}
fn pending(f: &mut Running) { assert!(poll(f).is_pending()); }
fn finish(f: &mut Running) -> Result<()> {
    match poll(f) { Poll::Ready(r) => { assert_eq!(r?,123463); Ok(()) },
        Poll::Pending => Err(wasmtime::Error::msg("unexpected suspension")) }
}
fn effect(w: &Shared, id: u64, bytes: usize) {
    let mut w=w.lock().unwrap(); let r=w.pending[&id].clone();
    match r.kind {
        Kind::Write{offset,length} => {
            assert!(bytes<=length); w.volatile[r.node].resize(7,0);
            for i in offset..offset+bytes { w.volatile[r.node][i]=10+r.node as u8; }
        },
        Kind::Sync => { w.durable[r.node]=w.volatile[r.node].clone(); }
    }
    w.trace.push(format!("effect r{id} bytes={bytes}"));
}
fn complete(w:&Shared,id:u64,n:u32) {
    let mut w=w.lock().unwrap(); w.ready.insert(id,n); w.trace.push(format!("ready r{id} {n}"));
}
fn schedule(engine:&Engine, component:&Component, kill:bool, persist_prefix:bool)->Result<Vec<String>> {
    let w=Arc::new(Mutex::new(World::default()));
    let mut a=node(engine.clone(),component.clone(),0,w.clone())?;
    let mut b=node(engine.clone(),component.clone(),1,w.clone())?;
    pending(&mut a); pending(&mut b); effect(&w,0,3);
    if persist_prefix {
        let mut m=w.lock().unwrap(); m.durable[0]=m.volatile[0].clone();
        m.trace.push("spontaneous persistence n0 prefix3".into());
    }
    if kill {
        w.lock().unwrap().trace.push("kill n0 while import pending".into());
        drop(a);
        {
            let mut m=w.lock().unwrap();
            assert!(m.pending.contains_key(&0)); // issued operation outlives waiter
            assert_eq!(m.host_drops,1);
            let d=m.durable[0].clone(); m.volatile[0]=d;
            m.trace.push("power loss n0 discards volatile state".into());
        }
        effect(&w,1,7); complete(&w,1,7); pending(&mut b);
        effect(&w,2,0); complete(&w,2,0); finish(&mut b)?;
        assert!(!w.lock().unwrap().trace.iter().any(|s|s.starts_with("published n0")));
    } else {
        pending(&mut a); complete(&w,0,3); pending(&mut a);
        effect(&w,1,7); complete(&w,1,7); pending(&mut b);
        effect(&w,2,4); complete(&w,2,4); pending(&mut a);
        effect(&w,4,0); pending(&mut a);
        complete(&w,4,0); finish(&mut a)?;
        effect(&w,3,0); complete(&w,3,0); finish(&mut b)?;
        assert_eq!(w.lock().unwrap().durable[0],vec![10;7]);
    }
    let result=w.lock().unwrap().trace.clone(); Ok(result)
}
fn main()->Result<()> {
    let engine=Engine::default(); let component=Component::new(&engine,GUEST)?;
    for kill in [false,true] { for persist in [false,true] {
        let reference=schedule(&engine,&component,kill,persist)?;
        for _ in 0..100 { assert_eq!(reference,schedule(&engine,&component,kill,persist)?); }
        println!("PASS kill={kill} persist_prefix={persist}: 101 identical traces");
        if kill && persist { for line in &reference { println!("  {line}"); } }
    }}
    println!("Mechanism probe only: nested synchronous component calls, two stores, short writes, separate effect/persistence/completion, drop during import, host cleanup. No production port or performance claim.");
    Ok(())
}

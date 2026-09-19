//! Mechanism check: synchronous component calls suspended at host imports.
//!
//! This is a hand-selected schedule, not an exploration engine. Effects, durability
//! and completion are separate. A killed invocation never resumes; its already
//! submitted operations remain owned by World. See the experiment README for
//! the stronger guarantees this probe does not establish.

use std::{
    collections::BTreeMap,
    future::{Future, poll_fn},
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};
use wasmtime::Result;
use wasmtime::component::{Component, Linker};
use wasmtime::{Engine, Store};

// A component with synchronous canonical imports and live nested call frames.
const GUEST: &str = include_str!("../fixtures/commit.wat");

#[derive(Clone, Debug)]
enum OperationKind {
    Write { offset: usize, length: usize },
    Sync,
}
#[derive(Clone, Debug)]
struct Request {
    node: usize,
    kind: OperationKind,
}
/// Simulator-owned state survives dropping an invocation. No native I/O occurs here.
#[derive(Default)]
struct World {
    next: u64,
    pending_requests: BTreeMap<u64, Request>,
    ready: BTreeMap<u64, u32>,
    volatile: [Vec<u8>; 2],
    durable: [Vec<u8>; 2],
    trace: Vec<String>,
    host_drops: usize,
}
type Shared = Arc<Mutex<World>>;
#[derive(Clone)]
struct Node {
    id: usize,
    world: Shared,
}
/// Records host cleanup, which must not be mistaken for guest process cleanup.
struct HostGuard(Shared);
impl Drop for HostGuard {
    fn drop(&mut self) {
        self.0.lock().unwrap().host_drops += 1;
    }
}
async fn submit_and_wait(node: Node, kind: OperationKind) -> Result<(u32,)> {
    let _guard = HostGuard(node.world.clone());
    let id = {
        let mut world = node.world.lock().unwrap();
        let id = world.next;
        world.next += 1;
        world
            .trace
            .push(format!("submit n{} r{} {:?}", node.id, id, kind));
        world.pending_requests.insert(
            id,
            Request {
                node: node.id,
                kind,
            },
        );
        id
    };
    let result = poll_fn(|_cx| {
        let mut world = node.world.lock().unwrap();
        match world.ready.remove(&id) {
            Some(n) => {
                world.pending_requests.remove(&id);
                Poll::Ready(n)
            }
            None => Poll::Pending,
        }
    })
    .await;
    node.world
        .lock()
        .unwrap()
        .trace
        .push(format!("return r{id} {result}"));
    Ok((result,))
}
type NodeInvocation = Pin<Box<dyn Future<Output = Result<u32>>>>;
fn create_node_invocation(
    engine: Engine,
    component: Component,
    id: usize,
    world: Shared,
) -> Result<NodeInvocation> {
    let mut linker = Linker::<Node>::new(&engine);
    linker
        .root()
        .func_wrap_async("write", |cx, (offset, length): (u32, u32)| {
            Box::new(submit_and_wait(
                cx.data().clone(),
                OperationKind::Write {
                    offset: offset as usize,
                    length: length as usize,
                },
            ))
        })?;
    linker.root().func_wrap_async("sync", |cx, (): ()| {
        Box::new(submit_and_wait(cx.data().clone(), OperationKind::Sync))
    })?;
    Ok(Box::pin(async move {
        let mut store = Store::new(
            &engine,
            Node {
                id,
                world: world.clone(),
            },
        );
        let instance = linker.instantiate_async(&mut store, &component).await?;
        let run = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
        let (n,) = run.call_async(&mut store, ()).await?;
        world
            .lock()
            .unwrap()
            .trace
            .push(format!("published n{id} {n}"));
        Ok(n)
    }))
}
/// Poll only when the driver chooses; wakeups never schedule a node implicitly.
fn poll_invocation(f: &mut NodeInvocation) -> Poll<Result<u32>> {
    f.as_mut().poll(&mut Context::from_waker(Waker::noop()))
}
fn assert_pending(f: &mut NodeInvocation) {
    assert!(poll_invocation(f).is_pending());
}
fn assert_finished(f: &mut NodeInvocation) -> Result<()> {
    match poll_invocation(f) {
        Poll::Ready(r) => {
            assert_eq!(r?, 123463);
            Ok(())
        }
        Poll::Pending => Err(wasmtime::Error::msg("unexpected suspension")),
    }
}
/// Change volatile storage (or persist it for sync) without delivering completion.
fn apply_effect(world: &Shared, id: u64, bytes: usize) {
    let mut world = world.lock().unwrap();
    let r = world.pending_requests[&id].clone();
    match r.kind {
        OperationKind::Write { offset, length } => {
            assert!(bytes <= length);
            world.volatile[r.node].resize(7, 0);
            for i in offset..offset + bytes {
                world.volatile[r.node][i] = 10 + r.node as u8;
            }
        }
        OperationKind::Sync => {
            world.durable[r.node] = world.volatile[r.node].clone();
        }
    }
    world.trace.push(format!("effect r{id} bytes={bytes}"));
}
/// Mark a result available; the guest observes it on its next explicitly selected poll.
fn make_completion_ready(world: &Shared, id: u64, n: u32) {
    let mut world = world.lock().unwrap();
    world.ready.insert(id, n);
    world.trace.push(format!("ready r{id} {n}"));
}
/// A deliberately fixed two-node schedule. Request IDs encode submission order.
/// The assertions make changes to that order fail loudly rather than silently
/// turning this mechanism check into a different experiment.
fn run_schedule(
    engine: &Engine,
    component: &Component,
    kill: bool,
    persist_prefix: bool,
) -> Result<Vec<String>> {
    let world = Arc::new(Mutex::new(World::default()));
    let mut node_a = create_node_invocation(engine.clone(), component.clone(), 0, world.clone())?;
    let mut node_b = create_node_invocation(engine.clone(), component.clone(), 1, world.clone())?;
    assert_pending(&mut node_a);
    assert_pending(&mut node_b);
    apply_effect(&world, 0, 3);
    if persist_prefix {
        let mut state = world.lock().unwrap();
        state.durable[0] = state.volatile[0].clone();
        state
            .trace
            .push("spontaneous persistence n0 prefix3".into());
    }
    if kill {
        world
            .lock()
            .unwrap()
            .trace
            .push("kill n0 while import pending".into());
        drop(node_a);
        {
            let mut state = world.lock().unwrap();
            assert!(state.pending_requests.contains_key(&0)); // issued operation outlives waiter
            assert_eq!(state.host_drops, 1);
            let d = state.durable[0].clone();
            state.volatile[0] = d;
            state
                .trace
                .push("power loss n0 discards volatile state".into());
        }
        apply_effect(&world, 1, 7);
        make_completion_ready(&world, 1, 7);
        assert_pending(&mut node_b);
        apply_effect(&world, 2, 0);
        make_completion_ready(&world, 2, 0);
        assert_finished(&mut node_b)?;
        assert!(
            !world
                .lock()
                .unwrap()
                .trace
                .iter()
                .any(|s| s.starts_with("published n0"))
        );
    } else {
        assert_pending(&mut node_a);
        make_completion_ready(&world, 0, 3);
        assert_pending(&mut node_a);
        apply_effect(&world, 1, 7);
        make_completion_ready(&world, 1, 7);
        assert_pending(&mut node_b);
        apply_effect(&world, 2, 4);
        make_completion_ready(&world, 2, 4);
        assert_pending(&mut node_a);
        apply_effect(&world, 4, 0);
        assert_pending(&mut node_a);
        make_completion_ready(&world, 4, 0);
        assert_finished(&mut node_a)?;
        apply_effect(&world, 3, 0);
        make_completion_ready(&world, 3, 0);
        assert_finished(&mut node_b)?;
        assert_eq!(world.lock().unwrap().durable[0], vec![10; 7]);
    }
    let result = world.lock().unwrap().trace.clone();
    Ok(result)
}
fn main() -> Result<()> {
    let engine = Engine::default();
    let component = Component::new(&engine, GUEST)?;
    for kill in [false, true] {
        for persist in [false, true] {
            let reference = run_schedule(&engine, &component, kill, persist)?;
            for _ in 0..100 {
                assert_eq!(reference, run_schedule(&engine, &component, kill, persist)?);
            }
            println!("PASS kill={kill} persist_prefix={persist}: 101 identical traces");
            if kill && persist {
                for line in &reference {
                    println!("  {line}");
                }
            }
        }
    }
    println!(
        "Mechanism probe only: nested synchronous component calls, two stores, short writes, separate effect/persistence/completion, drop during import, host cleanup. No production port or performance claim."
    );
    Ok(())
}

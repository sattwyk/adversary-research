//! Run the unmodified redb engine through synchronous Wasm storage imports.
//!
//! This exercises a real commit/recovery path with one caller. It does not test
//! native filesystem semantics, concurrent readers, or general crash coverage.
//! Phase 2 identifies commit I/O; phases 1 and 3 are setup and post-commit reads.

use std::{
    collections::BTreeMap,
    future::{Future, poll_fn},
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};
use wasmtime::{Caller, Engine, Linker, Module, Result, Store};
#[derive(Clone, Debug)]
enum StorageOperation {
    Len,
    Read(u64, usize),
    Write(u64, Vec<u8>),
    Resize(u64),
    Sync,
}
/// Model state is external to the guest lifetime. This byte-vector disk is not
/// a qualified model of a real filesystem or storage device.
#[derive(Default)]
struct World {
    volatile: Vec<u8>,
    durable: Vec<u8>,
    phase: u32,
    next: u64,
    pending: BTreeMap<u64, StorageOperation>,
    ready: BTreeMap<u64, u64>,
    trace: Vec<String>,
    commit_writes: usize,
    guest_drops: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CrashPoint {
    CommitWrite(usize),
    SyncBeforeCompletion,
}

type Shared = Arc<Mutex<World>>;
/// The waiter owns no disk operation: abandoning it cannot undo a write.
async fn submit_and_wait(world: Shared, op: StorageOperation) -> Result<u64> {
    let id = {
        let mut world = world.lock().unwrap();
        let id = world.next;
        world.next += 1;
        let label = match &op {
            StorageOperation::Write(o, b) => format!("Write({o},{})", b.len()),
            x => format!("{x:?}"),
        };
        let phase = world.phase;
        world.trace.push(format!("p{phase} r{id} {label}"));
        world.pending.insert(id, op);
        id
    };
    Ok(poll_fn(|_| {
        let mut world = world.lock().unwrap();
        match world.ready.remove(&id) {
            Some(n) => {
                world.pending.remove(&id);
                Poll::Ready(n)
            }
            None => Poll::Pending,
        }
    })
    .await)
}
fn memory(caller: &mut Caller<'_, Shared>) -> wasmtime::Memory {
    caller.get_export("memory").unwrap().into_memory().unwrap()
}
type GuestInvocation = Pin<Box<dyn Future<Output = Result<u64>>>>;
/// Each invocation owns a fresh Store. Writes copy guest bytes before suspension;
/// reads copy results back only after the driver has delivered their completion.
fn start_guest(
    engine: &Engine,
    module: &Module,
    world: Shared,
    entry: &str,
) -> Result<GuestInvocation> {
    let mut linker = Linker::new(engine);
    linker.func_wrap_async(
        "sim",
        "storage_len",
        |caller: Caller<'_, Shared>, (): ()| {
            Box::new(submit_and_wait(
                caller.data().clone(),
                StorageOperation::Len,
            ))
        },
    )?;
    linker.func_wrap_async(
        "sim",
        "storage_set_len",
        |caller: Caller<'_, Shared>, (n,): (u64,)| {
            Box::new(async move {
                submit_and_wait(caller.data().clone(), StorageOperation::Resize(n)).await?;
                Ok(0i32)
            })
        },
    )?;
    linker.func_wrap_async(
        "sim",
        "storage_sync",
        |caller: Caller<'_, Shared>, (): ()| {
            Box::new(async move {
                submit_and_wait(caller.data().clone(), StorageOperation::Sync).await?;
                Ok(0i32)
            })
        },
    )?;
    linker.func_wrap_async(
        "sim",
        "storage_write",
        |mut caller: Caller<'_, Shared>, (offset, ptr, len): (u64, u32, u32)| {
            Box::new(async move {
                let mut bytes = vec![0; len as usize];
                memory(&mut caller).read(&caller, ptr as usize, &mut bytes)?;
                submit_and_wait(
                    caller.data().clone(),
                    StorageOperation::Write(offset, bytes),
                )
                .await?;
                Ok(0i32)
            })
        },
    )?;
    linker.func_wrap_async(
        "sim",
        "storage_read",
        |mut caller: Caller<'_, Shared>, (offset, ptr, len): (u64, u32, u32)| {
            Box::new(async move {
                submit_and_wait(
                    caller.data().clone(),
                    StorageOperation::Read(offset, len as usize),
                )
                .await?;
                let bytes = caller.data().lock().unwrap().volatile
                    [offset as usize..offset as usize + len as usize]
                    .to_vec();
                memory(&mut caller).write(&mut caller, ptr as usize, &bytes)?;
                Ok(0i32)
            })
        },
    )?;
    linker.func_wrap("sim", "phase", |caller: Caller<'_, Shared>, p: u32| {
        caller.data().lock().unwrap().phase = p;
    })?;
    linker.func_wrap("sim", "cleanup", |caller: Caller<'_, Shared>| {
        caller.data().lock().unwrap().guest_drops += 1;
    })?;
    let engine = engine.clone();
    let module = module.clone();
    let entry = entry.to_owned();
    Ok(Box::pin(async move {
        let mut store = Store::new(&engine, world);
        let instance = linker.instantiate_async(&mut store, &module).await?;
        instance
            .get_typed_func::<(), u64>(&mut store, &entry)?
            .call_async(&mut store, ())
            .await
    }))
}
/// Drive one invocation against the model, optionally killing it after an effect
/// but before returning completion. These crash cases also model power loss, so
/// pending model operations and non-durable bytes are discarded explicitly.
fn drive_guest(
    mut invocation: GuestInvocation,
    world: Shared,
    crash: Option<CrashPoint>,
    persist_partial: bool,
) -> Result<Option<u64>> {
    loop {
        match invocation
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
        {
            Poll::Ready(r) => return r.map(Some),
            Poll::Pending => {}
        }
        let mut state = world.lock().unwrap();
        let (&id, op) = state.pending.iter().next().expect("pending import");
        let op = op.clone();
        let should_kill = if matches!(&op, StorageOperation::Write(..)) && state.phase == 2 {
            state.commit_writes += 1;
            crash == Some(CrashPoint::CommitWrite(state.commit_writes))
        } else {
            matches!(&op, StorageOperation::Sync)
                && state.phase == 2
                && crash == Some(CrashPoint::SyncBeforeCompletion)
        };
        let n = match op {
            StorageOperation::Len => state.volatile.len() as u64,
            StorageOperation::Read(offset, len) => {
                assert!(offset as usize + len <= state.volatile.len());
                0
            }
            StorageOperation::Resize(n) => {
                state.volatile.resize(n as usize, 0);
                0
            }
            StorageOperation::Sync => {
                state.durable = state.volatile.clone();
                0
            }
            StorageOperation::Write(offset, bytes) => {
                let len = if should_kill {
                    bytes.len() / 2
                } else {
                    bytes.len()
                };
                state.volatile[offset as usize..offset as usize + len]
                    .copy_from_slice(&bytes[..len]);
                if should_kill && persist_partial {
                    // Persist only this changed prefix, not all earlier unsynced writes.
                    if state.durable.len() < offset as usize + len {
                        state.durable.resize(offset as usize + len, 0);
                    }
                    state.durable[offset as usize..offset as usize + len]
                        .copy_from_slice(&bytes[..len]);
                }
                0
            }
        };
        if should_kill {
            state.trace.push(format!(
                "CRASH during operation {id}; persisted_prefix={persist_partial}"
            ));
            drop(state);
            drop(invocation); // no application error return, no guest cleanup
            let mut state = world.lock().unwrap();
            state.pending.clear();
            state.ready.clear();
            state.volatile = state.durable.clone();
            state.phase = 0;
            return Ok(None);
        }
        state.ready.insert(id, n);
        drop(state);
    }
}
/// Reopen a new guest against surviving bytes. An absent table is the old state;
/// unexpected recovery errors trap in the guest and fail this experiment.
fn run_scenario(
    engine: &Engine,
    module: &Module,
    crash: Option<CrashPoint>,
    persist: bool,
) -> Result<(Vec<String>, usize)> {
    let world = Arc::new(Mutex::new(World::default()));
    let run = drive_guest(
        start_guest(engine, module, world.clone(), "run")?,
        world.clone(),
        crash,
        persist,
    )?;
    if crash.is_none() {
        assert_eq!(run, Some(42));
    } else {
        assert!(run.is_none());
    }
    assert_eq!(
        world.lock().unwrap().guest_drops,
        if crash.is_none() { 1 } else { 0 }
    );
    let writes = world.lock().unwrap().commit_writes;
    {
        let mut state = world.lock().unwrap();
        state.volatile = state.durable.clone();
        state.phase = 0;
    }
    let recovered = drive_guest(
        start_guest(engine, module, world.clone(), "recover")?,
        world.clone(),
        None,
        false,
    )?
    .unwrap();
    assert!(recovered == 0 || recovered == 42);
    if crash.is_none() {
        assert_eq!(recovered, 42);
    }
    if crash == Some(CrashPoint::SyncBeforeCompletion) {
        assert_eq!(recovered, 42);
    }
    let mut state = world.lock().unwrap();
    state.trace.push(format!("recovered={recovered}"));
    Ok((state.trace.clone(), writes))
}
fn main() -> Result<()> {
    let path = std::env::args().nth(1).expect("guest wasm path");
    let engine = Engine::default();
    let module = Module::from_file(&engine, path)?;
    let (baseline, writes) = run_scenario(&engine, &module, None, false)?;
    println!(
        "Unmodified redb 4.1.0: commit+read=42, power-loss reopen=42; {writes} backend writes during commit."
    );
    println!("Baseline import count: {}", baseline.len() - 1);
    for write_index in 1..=writes {
        for persist in [false, true] {
            let (reference, _) = run_scenario(
                &engine,
                &module,
                Some(CrashPoint::CommitWrite(write_index)),
                persist,
            )?;
            for _ in 0..9 {
                assert_eq!(
                    reference,
                    run_scenario(
                        &engine,
                        &module,
                        Some(CrashPoint::CommitWrite(write_index)),
                        persist
                    )?
                    .0
                );
            }
            println!(
                "PASS commit write {write_index}: half effect, persist={persist}, kill suspended guest; {}; 10 identical traces",
                reference.last().unwrap()
            );
        }
    }
    let (sync_trace, _) = run_scenario(
        &engine,
        &module,
        Some(CrashPoint::SyncBeforeCompletion),
        false,
    )?;
    for _ in 0..9 {
        assert_eq!(
            sync_trace,
            run_scenario(
                &engine,
                &module,
                Some(CrashPoint::SyncBeforeCompletion),
                false
            )?
            .0
        );
    }
    println!(
        "PASS: sync effect persisted, killed before completion returned, recovered=42; 10 identical traces."
    );
    println!("PASS: guest Drop marker ran on normal completion and never on suspended-call kill.");
    println!(
        "Scope: real synchronous redb engine, single caller, simple abstract disk. No native concurrency/OS conformance or bug-discovery claim."
    );
    Ok(())
}

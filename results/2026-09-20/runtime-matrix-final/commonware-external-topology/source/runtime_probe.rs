//! Minimal executor substitution around the same Databend storage workload.
//! No runtime owns the native WAL thread merely by polling this future.
#[path = "runtime_shared/mod.rs"]
mod shared;
#[path = "runtime_shared/sync_failure.rs"]
mod sync_failure;

async fn workload() -> anyhow::Result<()> {
    match std::env::var("ADVERSARY_SCENARIO").as_deref() {
        Ok("sync-failure") => sync_failure::run().await,
        Ok("topology") | Err(_) => shared::run().await,
        Ok(other) => anyhow::bail!("unknown scenario: {other}"),
    }
}

#[cfg(not(any(
    feature = "adversary-turmoil",
    feature = "adversary-commonware",
    feature = "adversary-madsim"
)))]
fn main() -> anyhow::Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(workload())
}

#[cfg(feature = "adversary-commonware")]
fn main() -> anyhow::Result<()> {
    use commonware_runtime::Runner;
    commonware_runtime::deterministic::Runner::seeded(0).start(|_context| workload())
}

#[cfg(feature = "adversary-madsim")]
fn main() -> anyhow::Result<()> {
    // cfg(madsim) is required by the runner script. Without it this is not a
    // simulator experiment. Opting into native threads does not control them.
    let mut runtime = madsim::runtime::Runtime::new();
    if std::env::var_os("ADVERSARY_ALLOW_SYSTEM_THREAD").is_some() {
        runtime.set_allow_system_thread(true);
    }
    runtime.block_on(workload())
}

#[cfg(feature = "adversary-turmoil")]
fn main() -> anyhow::Result<()> {
    let mut sim = turmoil::Builder::new().build();
    sim.client("databend-wal", async {
        workload().await.map_err(|error| error.into())
    });
    sim.run().map_err(|error| anyhow::anyhow!("{error}"))
}

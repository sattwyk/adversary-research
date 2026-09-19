//! Thin storage adapter around an unchanged, pinned redb dependency.
//!
//! The exported workload uses real transaction and recovery code. The host owns
//! storage effects; phase/cleanup imports are test instrumentation only.
//! Build for wasm32-unknown-unknown with panic=abort so unexpected errors trap.

use redb::{Database, ReadableDatabase, StorageBackend, TableDefinition};
use std::io;

// The host copies write buffers before suspension and validates all memory
// ranges. These pointers must never survive guest destruction in host state.
#[link(wasm_import_module = "sim")]
unsafe extern "C" {
    fn storage_len() -> u64;
    fn storage_read(offset: u64, ptr: *mut u8, len: usize) -> i32;
    fn storage_write(offset: u64, ptr: *const u8, len: usize) -> i32;
    fn storage_set_len(len: u64) -> i32;
    fn storage_sync() -> i32;
    fn phase(id: u32);
    fn cleanup();
}
#[derive(Debug)]
struct Backend;
fn result(n: i32) -> io::Result<()> {
    if n == 0 {
        Ok(())
    } else {
        Err(io::Error::other("injected I/O error"))
    }
}
// Each pointer below comes from a live Rust slice and remains valid until its
// synchronous import returns. The host controls suspension without unwinding it.
impl StorageBackend for Backend {
    fn len(&self) -> io::Result<u64> {
        Ok(unsafe { storage_len() })
    }
    fn read(&self, offset: u64, out: &mut [u8]) -> io::Result<()> {
        result(unsafe { storage_read(offset, out.as_mut_ptr(), out.len()) })
    }
    fn write(&self, offset: u64, data: &[u8]) -> io::Result<()> {
        result(unsafe { storage_write(offset, data.as_ptr(), data.len()) })
    }
    fn set_len(&self, len: u64) -> io::Result<()> {
        result(unsafe { storage_set_len(len) })
    }
    fn sync_data(&self) -> io::Result<()> {
        result(unsafe { storage_sync() })
    }
}
const TABLE: TableDefinition<u64, u64> = TableDefinition::new("items");
/// Distinguishes normal guest cleanup from abandoning a suspended invocation.
struct GuestDropMarker;
impl Drop for GuestDropMarker {
    fn drop(&mut self) {
        unsafe { cleanup() };
    }
}
// No redb source patches or asynchronous conversion.
#[unsafe(no_mangle)]
pub extern "C" fn run() -> u64 {
    let _marker = GuestDropMarker;
    let db = Database::builder()
        .set_cache_size(1024 * 1024)
        .create_with_backend(Backend)
        .unwrap();
    unsafe { phase(1) };
    let tx = db.begin_write().unwrap();
    {
        let mut t = tx.open_table(TABLE).unwrap();
        t.insert(1, 42).unwrap();
    }
    unsafe { phase(2) };
    tx.commit().unwrap();
    unsafe { phase(3) };
    let value = db
        .begin_read()
        .unwrap()
        .open_table(TABLE)
        .unwrap()
        .get(1)
        .unwrap()
        .unwrap()
        .value();
    value
}
#[unsafe(no_mangle)]
pub extern "C" fn recover() -> u64 {
    let db = Database::builder()
        .set_cache_size(1024 * 1024)
        .create_with_backend(Backend)
        .unwrap();
    let tx = db.begin_read().unwrap();
    match tx.open_table(TABLE) {
        Ok(t) => t.get(1).unwrap().map(|v| v.value()).unwrap_or(0),
        Err(redb::TableError::TableDoesNotExist(_)) => 0,
        Err(e) => panic!("unexpected recovery error: {e}"),
    }
}

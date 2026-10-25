//! Test-only macOS allocator accounting. This is not RSS or managed-heap size.
#[derive(Default)]
#[repr(C)]
pub(super) struct Snapshot {
    blocks_in_use: std::ffi::c_uint,
    size_in_use: usize,
    max_size_in_use: usize,
    size_allocated: usize,
}

unsafe extern "C" {
    fn malloc_zone_statistics(zone: *mut std::ffi::c_void, stats: *mut Snapshot);
}

impl Snapshot {
    pub(super) fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "blocksInUse": self.blocks_in_use,
            "sizeInUse": self.size_in_use,
            "maxSizeInUse": self.max_size_in_use,
            "sizeAllocated": self.size_allocated,
        })
    }
}

pub(super) fn snapshot() -> Snapshot {
    let mut result = Snapshot::default();
    // SAFETY: malloc/malloc.h defines this repr(C) layout; null sums all zones.
    // The OS writes into a valid, exclusively borrowed output structure.
    unsafe { malloc_zone_statistics(std::ptr::null_mut(), &mut result) };
    result
}

#[test]
#[ignore = "process-wide counters require an isolated exact test invocation"]
fn observes_live_native_allocation_and_release() {
    let before = snapshot();
    let allocation = vec![0xabu8; 8 * 1024 * 1024];
    std::hint::black_box(&allocation);
    let during = snapshot();
    assert!(during.size_in_use >= before.size_in_use + allocation.len());
    drop(allocation);
    let after = snapshot();
    assert!(during.size_in_use >= after.size_in_use + 8 * 1024 * 1024);
    assert!(during.blocks_in_use > before.blocks_in_use);
}

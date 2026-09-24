//! Exploration only: compare current Value clones with shared immutable UTF-8.
//! This does not change the VM representation, GC or guest String identity.
use neoclr::Value;
use std::{
    alloc::{GlobalAlloc, Layout, System},
    hint::black_box,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

struct CountingAllocator;
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static REQUESTED_BYTES: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        REQUESTED_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        REQUESTED_BYTES.fetch_add(size, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, size) }
    }
}
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

#[derive(Clone)]
struct SharedText(Arc<str>);
impl SharedText {
    fn new(text: &str) -> Self {
        Self(Arc::from(text))
    }
    fn same_identity(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    fn same_content(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

fn measure(work: impl FnOnce()) -> (usize, usize) {
    let allocations = ALLOCATIONS.load(Ordering::Relaxed);
    let bytes = REQUESTED_BYTES.load(Ordering::Relaxed);
    work();
    (
        ALLOCATIONS.load(Ordering::Relaxed) - allocations,
        REQUESTED_BYTES.load(Ordering::Relaxed) - bytes,
    )
}

fn main() {
    const COPIES: usize = 10_000;
    println!(
        "utf8_bytes,copies,owned_allocations,owned_requested_bytes,shared_allocations,shared_requested_bytes"
    );
    for size in [0, 32, 4096, 65536] {
        let text = "x".repeat(size);
        let owned = Value::String(text.clone());
        let shared = SharedText::new(&text);
        let (owned_count, owned_bytes) = measure(|| {
            for _ in 0..COPIES {
                black_box(owned.clone());
            }
        });
        let (shared_count, shared_bytes) = measure(|| {
            for _ in 0..COPIES {
                black_box(shared.clone());
            }
        });
        assert_eq!(owned_count, if size == 0 { 0 } else { COPIES });
        assert_eq!(owned_bytes, size * COPIES);
        assert_eq!((shared_count, shared_bytes), (0, 0));
        println!("{size},{COPIES},{owned_count},{owned_bytes},{shared_count},{shared_bytes}");
    }
    // Existing VM baseline: adapting the same local repeatedly allocates wrappers.
    let app = neoclr::assemble(
        r#"
.module StringStorageBaseline
.entry Main
.type class abstract System.Object
.end
.function Main() -> Int32
.local String text
.local Int32 n
ldstr "hello 👩‍💻"
stloc text
ldc.i4 0
stloc n
loop:
ldloc text
castclass System.Object
pop
ldloc n
ldc.i4 1
add
stloc n
ldloc n
ldc.i4 1000
blt loop
ldc.i4 0
ret
.end
"#,
    )
    .unwrap();
    let program = neoclr::LoadedProgram::new(&app).unwrap();
    program.verify().unwrap();
    let execution = program
        .run(neoclr::Limits {
            heap_objects: 8,
            ..Default::default()
        })
        .unwrap();
    let stats = execution.heap.statistics();
    assert_eq!(stats.allocated_objects, 1000);
    assert_eq!(stats.reclaimed_objects, 1000);
    assert_eq!(stats.live_objects, 0);
    println!(
        "current VM: casts=1000 wrappers={} peak={} collections={} live={}",
        stats.allocated_objects, stats.peak_objects, stats.collections, stats.live_objects
    );
    // Prototype ownership gates; these are Rust containers, not guest GC tests.
    let original = SharedText::new("hello 👩‍💻");
    let separate = SharedText::new("hello 👩‍💻");
    assert!(original.same_content(&separate));
    assert!(!original.same_identity(&separate));
    let field = Some(original.clone());
    let array = [original.clone()];
    let weak = Arc::downgrade(&original.0);
    assert!(original.same_identity(field.as_ref().unwrap()));
    assert!(original.same_identity(&array[0]));
    drop(original);
    assert!(weak.upgrade().is_some());
    drop(field);
    assert!(weak.upgrade().is_some());
    drop(array);
    assert!(weak.upgrade().is_none());
    // No empty-string interning is promised by this candidate.
    assert!(!SharedText::new("").same_identity(&SharedText::new("")));
    println!("prototype identity, retained ownership and release: passed");
}

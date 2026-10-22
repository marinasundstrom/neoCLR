use super::*;
use crate::{Limits, metadata::Type};

fn layout() -> Layout {
    Layout {
        size: 4,
        alignment: 4,
        fields: vec![],
    }
}
fn allocate(memory: &SharedMemory) -> Pointer {
    memory
        .allocate(Type::Int32, &layout(), 1, usize::MAX, usize::MAX)
        .unwrap()
}

#[test]
fn native_contexts_share_identity_lifetime_and_limits() {
    let memory = SharedMemory::new(Limits {
        pointer_bytes: 4,
        pointer_allocations: 2,
        ..Limits::default()
    });
    let pointer = allocate(&memory);
    memory.write(&pointer, &layout(), &Value::Int32(7)).unwrap();
    let worker = memory.clone();
    let captured = pointer.clone();
    std::thread::spawn(move || {
        assert_eq!(worker.read(&captured, &layout()).unwrap(), Value::Int32(7));
        assert!(
            worker
                .allocate(Type::Int32, &layout(), 1, usize::MAX, usize::MAX)
                .is_err()
        );
        worker
            .write(&captured, &layout(), &Value::Int32(42))
            .unwrap();
    })
    .join()
    .unwrap();
    assert_eq!(memory.read(&pointer, &layout()).unwrap(), Value::Int32(42));
    memory.free(&pointer).unwrap();
    let next = allocate(&memory);
    assert_ne!(pointer.allocation, next.allocation);
    assert!(memory.read(&pointer, &layout()).is_err());
    memory.free(&next).unwrap();
    assert!(
        memory
            .allocate(Type::Int32, &layout(), 1, usize::MAX, usize::MAX)
            .is_err()
    );
    assert_eq!(memory.into_heap().unwrap().live_allocations(), 0);
}

#[test]
fn foreign_borrow_excludes_conflicts_but_allows_unrelated_progress() {
    let memory = SharedMemory::default();
    let pointer = allocate(&memory);
    memory.write(&pointer, &layout(), &Value::Int32(1)).unwrap();
    let raw = Pointer {
        allocation: None,
        ..pointer.clone()
    };
    let borrow = memory
        .borrow_native(&[Value::Pointer(raw), Value::Pointer(pointer.clone())])
        .unwrap();
    assert!(memory.read(&pointer, &layout()).is_err());
    assert!(memory.write(&pointer, &layout(), &Value::Int32(2)).is_err());
    assert!(memory.free(&pointer).is_err());
    assert!(
        memory
            .borrow_native(&[Value::Pointer(pointer.clone())])
            .is_err()
    );
    assert!(memory.debug_allocations()[0].bytes.is_empty());
    let worker = memory.clone();
    std::thread::spawn(move || {
        let independent = allocate(&worker);
        worker
            .write(&independent, &layout(), &Value::Int32(3))
            .unwrap();
        assert_eq!(
            worker.read(&independent, &layout()).unwrap(),
            Value::Int32(3)
        );
        worker.free(&independent).unwrap();
    })
    .join()
    .unwrap();
    // SAFETY: the lease excludes every safe VM access to this tracked allocation.
    unsafe { (pointer.address as *mut i32).write(42) };
    drop(borrow);
    assert_eq!(memory.read(&pointer, &layout()).unwrap(), Value::Int32(42));
    memory.free(&pointer).unwrap();
    assert_eq!(memory.into_heap().unwrap().live_allocations(), 0);
}

#[test]
fn failed_admission_and_unwinding_release_borrows() {
    let memory = SharedMemory::default();
    let live = allocate(&memory);
    let stale = allocate(&memory);
    memory.free(&stale).unwrap();
    assert!(
        memory
            .borrow_native(&[Value::Pointer(live.clone()), Value::Pointer(stale)])
            .is_err()
    );
    memory.write(&live, &layout(), &Value::Int32(9)).unwrap();
    let result = std::panic::catch_unwind(|| {
        let _borrow = memory
            .borrow_native(&[Value::Pointer(live.clone())])
            .unwrap();
        panic!("simulate native wrapper unwind");
    });
    assert!(result.is_err());
    assert_eq!(memory.read(&live, &layout()).unwrap(), Value::Int32(9));
    memory.free(&live).unwrap();
}

#[test]
fn foreign_borrow_retains_storage_after_context_exit() {
    let memory = SharedMemory::default();
    let pointer = allocate(&memory);
    memory
        .write(&pointer, &layout(), &Value::Int32(42))
        .unwrap();
    let borrow = memory
        .borrow_native(&[Value::Pointer(pointer.clone())])
        .unwrap();
    assert!(memory.into_heap().is_err());
    // SAFETY: the outstanding lease owns and excludes access to this allocation.
    assert_eq!(unsafe { (pointer.address as *const i32).read() }, 42);
    drop(borrow);
}

fn byte_view(values: Vec<Value>) -> (crate::slots::Cell, crate::MemoryView) {
    let count = values.len();
    let cell = crate::slots::Slot::cell(
        Type::Array(Box::new(Type::Byte)),
        Some(Value::Array {
            element: Type::Byte,
            elements: values,
        }),
    );
    let view = crate::MemoryView::borrowed(
        crate::memory_views::Slice::new(crate::SlotReference::new(&cell), 0, count).unwrap(),
    );
    (cell, view)
}

#[test]
fn native_byte_copy_preflights_ranges_and_initialization() {
    let memory = SharedMemory::default();
    let pointer = allocate(&memory);
    let (_source, source) = byte_view(vec![
        Value::Byte(1),
        Value::Byte(2),
        Value::Byte(3),
        Value::Byte(4),
    ]);
    let (_destination, destination) = byte_view(vec![Value::Byte(99); 4]);
    // An uninitialized native source cannot partially update managed output.
    assert!(memory.copy_view(&pointer, 4, &destination, false).is_err());
    assert_eq!(destination.get(0).unwrap(), Value::Byte(99));
    assert!(
        memory
            .copy_view(&pointer, 3, &source.readonly(), true)
            .is_err()
    );
    memory
        .copy_view(&pointer, 4, &source.readonly(), true)
        .unwrap();
    // Validate the complete range before copying a single byte.
    let offset = memory.offset(&pointer, 1).unwrap();
    assert!(memory.copy_view(&offset, 4, &destination, false).is_err());
    assert!(
        memory
            .copy_view(&pointer, usize::MAX, &destination, false)
            .is_err()
    );
    assert_eq!(destination.get(0).unwrap(), Value::Byte(99));
    memory.copy_view(&pointer, 2, &destination, false).unwrap();
    assert_eq!(destination.get(0).unwrap(), Value::Byte(1));
    assert_eq!(destination.get(1).unwrap(), Value::Byte(2));
    assert_eq!(destination.get(2).unwrap(), Value::Byte(99));
    assert!(
        memory
            .copy_view(&pointer, 4, &destination.readonly(), false)
            .is_err()
    );
    let guard = memory
        .borrow_native(&[Value::Pointer(pointer.clone())])
        .unwrap();
    assert!(memory.copy_view(&pointer, 4, &destination, false).is_err());
    assert!(
        memory
            .copy_view(&pointer, 4, &source.readonly(), true)
            .is_err()
    );
    drop(guard);
    memory.free(&pointer).unwrap();
    assert!(memory.copy_view(&pointer, 4, &destination, false).is_err());
}

#[test]
fn native_byte_copy_empty_windows_allow_null_without_access() {
    let memory = SharedMemory::default();
    let (_owner, view) = byte_view(vec![]);
    let null = Pointer::null(Type::Void);
    memory.copy_view(&null, 0, &view.readonly(), true).unwrap();
    memory.copy_view(&null, 0, &view, false).unwrap();
    let (_owner, nonempty) = byte_view(vec![Value::Byte(1)]);
    assert!(
        memory
            .copy_view(&null, 1, &nonempty.readonly(), true)
            .is_err()
    );
    assert!(memory.copy_view(&null, 1, &nonempty, false).is_err());
}

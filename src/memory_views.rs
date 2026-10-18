//! Checked, zero-copy memory-view capabilities for the interpreter.
//!
//! These are internal building blocks, not yet guest System.Slice/Memory types.
//! A SlotReference is a logical managed address, not a host pointer. Its array
//! path and the start below jointly identify the first element, including an
//! empty window at the end. No payload is read/copied to construct a view.
//! Before guest exposure, metadata, escape checks and native lowering must agree
//! on the byref-like contract. In particular, do not encode Slice as an ordinary
//! Value::Object: existing heap-only reference checks are insufficient for that.
use crate::{Fault, FaultCode, ManagedHeap, SlotReference, Value, metadata::Type};

#[derive(Debug, Clone, Copy)]
struct Window {
    start: usize,
    length: usize,
}

impl Window {
    fn new(total: usize, start: usize, length: usize) -> Result<Self, Fault> {
        // Subtract before adding, so even hostile usize::MAX inputs cannot wrap.
        if start > total || length > total - start {
            return Err(bounds());
        }
        Ok(Self { start, length })
    }

    fn slice(self, start: usize, length: usize) -> Result<Self, Fault> {
        let relative = Self::new(self.length, start, length)?;
        Ok(Self {
            start: self.start + relative.start,
            length,
        })
    }

    fn index(self, index: usize) -> Result<usize, Fault> {
        if index >= self.length {
            return Err(bounds());
        }
        Ok(self.start + index)
    }
}

fn bounds() -> Fault {
    Fault::coded(
        FaultCode::IndexOutOfRange,
        "memory view bounds out of range",
    )
}

/// Borrowed managed array window. Copies alias storage; readonly is a capability
/// on the reference and cannot be widened through copying, slicing or indexing.
#[derive(Debug, Clone)]
pub(crate) struct Slice {
    array: SlotReference,
    window: Window,
}

impl Slice {
    pub(crate) fn new(array: SlotReference, start: usize, length: usize) -> Result<Self, Fault> {
        if !matches!(array.target(), Type::Array(_)) {
            return Err(Fault::new("memory view requires an exact array reference"));
        }
        let window = Window::new(array.array_length()?, start, length)?;
        Ok(Self { array, window })
    }

    pub(crate) fn length(&self) -> usize {
        self.window.length
    }

    pub(crate) fn slice(&self, start: usize, length: usize) -> Result<Self, Fault> {
        self.array.assigned()?;
        Ok(Self {
            array: self.array.clone(),
            window: self.window.slice(start, length)?,
        })
    }

    pub(crate) fn as_readonly(&self) -> Self {
        let mut result = self.clone();
        result.array.restrict_readonly();
        result
    }

    pub(crate) fn element(&self, index: usize) -> Result<SlotReference, Fault> {
        let Type::Array(element) = self.array.target() else {
            unreachable!("validated array view")
        };
        self.array.element(self.window.index(index)?, element)
    }

    pub(crate) fn get(&self, index: usize) -> Result<Value, Fault> {
        self.element(index)?.read()
    }

    pub(crate) fn set(&self, index: usize, value: Value) -> Result<(), Fault> {
        self.element(index)?.write(value)
    }

    /// Publish the actual owner, including for empty and interior array views.
    /// A frame-backed view's owner must be scanned through the live caller frame.
    /// The caller still owns the collection boundary and root publication.
    pub(crate) fn heap_root(&self, heap: &ManagedHeap) -> Result<Option<usize>, Fault> {
        self.array.assigned()?;
        if let Some(identity) = self.array.allocation_id() {
            heap.check_reference_owner(&self.array)?;
            Ok(Some(identity))
        } else {
            Ok(None)
        }
    }

    /// Used by the future guest return checker; a view may return into a caller's
    /// frame but must never keep the returning frame's own storage alive.
    pub(crate) fn check_return(&self, frame: &[crate::slots::Cell]) -> Result<(), Fault> {
        self.array.assigned()?;
        if frame.iter().any(|cell| self.array.addresses(cell)) {
            return Err(Fault::new("cannot return a slice of the current frame"));
        }
        Ok(())
    }
}

/// Persistent window over a GC-owned array. Construction requires the owning
/// execution heap; allocation IDs from another execution do not authorize use.
/// Like other managed capabilities, this does not root itself automatically.
#[derive(Debug, Clone)]
pub(crate) struct Memory {
    view: Slice,
}

impl Memory {
    pub(crate) fn new(
        heap: &ManagedHeap,
        array: SlotReference,
        start: usize,
        length: usize,
    ) -> Result<Self, Fault> {
        heap.check_reference_owner(&array)?;
        Ok(Self {
            view: Slice::new(array, start, length)?,
        })
    }

    pub(crate) fn length(&self) -> usize {
        self.view.length()
    }

    pub(crate) fn slice(&self, start: usize, length: usize) -> Result<Self, Fault> {
        Ok(Self {
            view: self.view.slice(start, length)?,
        })
    }

    pub(crate) fn as_slice(&self) -> Slice {
        self.view.clone()
    }

    pub(crate) fn as_readonly(&self) -> Self {
        Self {
            view: self.view.as_readonly(),
        }
    }

    pub(crate) fn heap_root(&self, heap: &ManagedHeap) -> Result<usize, Fault> {
        self.view
            .heap_root(heap)?
            .ok_or_else(|| Fault::new("memory requires a heap owner"))
    }
}

/// Separate immutable UTF-8 byte view. Byte boundaries deliberately need not be
/// grapheme or scalar boundaries. This never changes String/Character indexing.
#[derive(Debug, Clone)]
pub(crate) struct Utf8Slice {
    owner: crate::StringValue,
    window: Window,
}

impl Utf8Slice {
    pub(crate) fn new(owner: crate::StringValue) -> Self {
        let length = owner.len();
        Self {
            owner,
            window: Window { start: 0, length },
        }
    }

    pub(crate) fn slice(&self, start: usize, length: usize) -> Result<Self, Fault> {
        Ok(Self {
            owner: self.owner.clone(),
            window: self.window.slice(start, length)?,
        })
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.owner.as_bytes()[self.window.start..self.window.start + self.window.length]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CollectionReason, slots::Slot};

    fn array() -> Value {
        Value::Array {
            element: Type::Int32,
            elements: [10, 20, 30, 40, 50].into_iter().map(Value::Int32).collect(),
        }
    }

    fn allocated(heap: &mut ManagedHeap) -> SlotReference {
        let id = heap.allocate(array()).unwrap();
        heap.address(id).unwrap()
    }

    #[test]
    fn nested_slices_and_copies_share_original_storage() {
        let mut heap = ManagedHeap::default();
        let owner = allocated(&mut heap);
        let view = Slice::new(owner.clone(), 1, 3).unwrap();
        let nested = view.slice(1, 2).unwrap();
        nested.clone().set(0, Value::Int32(99)).unwrap();
        assert_eq!(view.get(1).unwrap(), Value::Int32(99));
        assert_eq!(
            owner.element(2, &Type::Int32).unwrap().read().unwrap(),
            Value::Int32(99)
        );
        assert_eq!(heap.statistics().allocated_objects, 1);
        assert!(nested.set(0, Value::Byte(1)).is_err());
        assert_eq!(nested.get(0).unwrap(), Value::Int32(99));
    }

    #[test]
    fn readonly_is_preserved_through_nested_views_and_element_addresses() {
        let mut heap = ManagedHeap::default();
        let view = Slice::new(allocated(&mut heap), 0, 5).unwrap();
        let readonly = view.as_readonly().slice(1, 2).unwrap();
        assert!(readonly.set(0, Value::Int32(1)).is_err());
        assert!(readonly.element(0).unwrap().write(Value::Int32(1)).is_err());
        view.set(1, Value::Int32(77)).unwrap();
        assert_eq!(readonly.get(0).unwrap(), Value::Int32(77));
        let mut reference = allocated(&mut heap);
        reference.restrict_readonly();
        assert!(
            Slice::new(reference, 0, 1)
                .unwrap()
                .set(0, Value::Int32(0))
                .is_err()
        );
    }

    #[test]
    fn bounds_include_empty_end_and_do_not_wrap() {
        let mut heap = ManagedHeap::default();
        let owner = allocated(&mut heap);
        let view = Slice::new(owner.clone(), 1, 3).unwrap();
        assert_eq!(view.slice(3, 0).unwrap().length(), 0);
        assert!(view.slice(3, 0).unwrap().get(0).is_err());
        for (start, length) in [(4, 0), (2, 2), (0, usize::MAX), (usize::MAX, 1)] {
            assert!(view.slice(start, length).is_err());
        }
        assert!(view.get(3).is_err());
        assert!(view.get(usize::MAX).is_err());
        assert!(Slice::new(owner.clone(), usize::MAX, 0).is_err());
        assert!(Slice::new(owner, 0, usize::MAX).is_err());
    }

    #[test]
    fn frame_views_cannot_return_own_storage_or_become_persistent() {
        let cell = Slot::cell(Type::Array(Box::new(Type::Int32)), Some(array()));
        let reference = SlotReference::new(&cell);
        let view = Slice::new(reference.clone(), 0, 5).unwrap();
        assert!(view.check_return(std::slice::from_ref(&cell)).is_err());
        assert!(
            view.as_readonly()
                .slice(5, 0)
                .unwrap()
                .check_return(&[cell])
                .is_err()
        );
        assert!(view.check_return(&[]).is_ok());
        let heap = ManagedHeap::default();
        assert_eq!(view.heap_root(&heap).unwrap(), None);
        assert!(Memory::new(&heap, reference, 0, 5).is_err());
    }

    #[test]
    fn persistent_views_retain_owner_when_published_and_expire_after_collection() {
        let mut heap = ManagedHeap::default();
        let owner = allocated(&mut heap);
        let memory = Memory::new(&heap, owner, 1, 3).unwrap();
        let saved = memory.slice(1, 2).unwrap().as_readonly();
        assert_eq!(saved.length(), 2);
        let roots = vec![saved.heap_root(&heap).unwrap()];
        heap.collect(roots, CollectionReason::ExplicitRequest)
            .unwrap();
        assert_eq!(saved.as_slice().get(0).unwrap(), Value::Int32(30));
        assert!(saved.as_slice().set(0, Value::Int32(0)).is_err());
        memory.as_slice().set(1, Value::Int32(31)).unwrap();
        assert_eq!(saved.as_slice().get(0).unwrap(), Value::Int32(31));
        heap.collect(vec![], CollectionReason::ExplicitRequest)
            .unwrap();
        assert!(saved.as_slice().get(0).is_err());
        assert!(saved.heap_root(&heap).is_err());
        assert!(heap.is_empty());
    }

    #[test]
    fn empty_interior_view_roots_containing_object_and_reachable_elements() {
        let mut heap = ManagedHeap::default();
        let child = allocated(&mut heap);
        let child_id = child.allocation_id().unwrap();
        let element = Type::ByRef(Box::new(Type::Array(Box::new(Type::Int32))));
        let payload = Value::Array {
            element: element.clone(),
            elements: vec![Value::SlotReference(child)],
        };
        let owner = heap
            .allocate(Value::Object {
                ty: Type::Named("Container".into()),
                fields: vec![payload],
            })
            .unwrap();
        let interior = heap
            .address(owner)
            .unwrap()
            .field(0, Type::Array(Box::new(element)))
            .unwrap();
        let empty = Slice::new(interior, 1, 0).unwrap();
        heap.collect(
            vec![empty.heap_root(&heap).unwrap().unwrap()],
            CollectionReason::ExplicitRequest,
        )
        .unwrap();
        assert!(heap.contains(owner));
        assert!(heap.contains(child_id));
        assert!(empty.get(0).is_err());
        heap.collect(vec![], CollectionReason::ExplicitRequest)
            .unwrap();
        assert!(heap.is_empty());
    }

    #[test]
    fn same_allocation_id_in_foreign_heap_does_not_authorize_memory() {
        let mut first = ManagedHeap::default();
        let mut second = ManagedHeap::default();
        let owner = allocated(&mut first);
        let other = allocated(&mut second);
        assert_eq!(owner.allocation_id(), other.allocation_id());
        assert!(Memory::new(&second, owner.clone(), 0, 5).is_err());
        let view = Slice::new(owner, 0, 5).unwrap();
        assert!(view.heap_root(&second).is_err());
    }

    #[test]
    fn construction_does_not_read_uninitialized_elements() {
        let cell = Slot::cell(
            Type::Array(Box::new(Type::Int32)),
            Some(Value::Array {
                element: Type::Int32,
                elements: vec![Value::Uninitialized(Type::Int32)],
            }),
        );
        let view = Slice::new(SlotReference::new(&cell), 0, 1).unwrap();
        assert!(view.get(0).is_err());
        view.set(0, Value::Int32(42)).unwrap();
        assert_eq!(view.get(0).unwrap(), Value::Int32(42));
    }

    #[test]
    fn utf8_bytes_share_owner_and_allow_non_text_boundaries() {
        let text: crate::StringValue = "é👩‍💻".into();
        let pointer = text.as_ptr();
        let bytes = Utf8Slice::new(text.clone());
        let continuation = bytes.slice(1, 1).unwrap();
        assert_eq!(bytes.bytes().as_ptr(), pointer);
        assert_eq!(continuation.bytes().as_ptr(), text.as_bytes()[1..].as_ptr());
        assert_eq!(continuation.bytes(), &[0xa9]);
        drop(text);
        drop(bytes);
        assert_eq!(continuation.bytes(), &[0xa9]);
        assert!(continuation.slice(1, 1).is_err());
        assert!(continuation.slice(usize::MAX, 0).is_err());
        assert!(continuation.slice(1, 0).unwrap().bytes().is_empty());
    }
}

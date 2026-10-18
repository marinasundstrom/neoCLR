//! Checked, zero-copy memory-view capabilities for the interpreter.
//!
//! These back explicitly admitted guest System.Slice/Memory types.
//! A SlotReference is a logical managed address, not a host pointer. Its array
//! path and the start below jointly identify the first element, including an
//! empty window at the end. No payload is read/copied to construct a view.
//! Value::MemoryView supplies tracing and storage/return checks. Metadata admission
//! and suspension checks enforce interpreter lifetimes; native lowering remains
//! unsupported. Do not encode Slice as an ordinary Value::Object.
use crate::{
    Fault, FaultCode, ManagedHeap, SlotReference, Value,
    metadata::{MemoryViewKind as Kind, Type},
};

/// Opaque interpreter memory-view payload, constructed through checked runtime
/// services and explicit nominal metadata. Host code cannot fabricate a view.
#[derive(Debug, Clone)]
pub struct MemoryView(View);

#[derive(Debug, Clone)]
enum View {
    Empty(Kind, Type),
    Slice(Slice),
    Memory(Memory),
    Utf8(Utf8Slice),
}

impl MemoryView {
    pub(crate) fn empty(kind: Kind, element: Type) -> Self {
        Self(View::Empty(kind, element))
    }

    pub(crate) fn kind(&self) -> Kind {
        match &self.0 {
            View::Empty(kind, _) => *kind,
            View::Slice(slice) => {
                if slice.array.is_readonly() {
                    Kind::ReadOnlySlice
                } else {
                    Kind::Slice
                }
            }
            View::Memory(memory) => {
                if memory.view.array.is_readonly() {
                    Kind::ReadOnlyMemory
                } else {
                    Kind::Memory
                }
            }
            View::Utf8(_) => Kind::ReadOnlySlice,
        }
    }

    pub(crate) fn length(&self) -> usize {
        match &self.0 {
            View::Empty(..) => 0,
            View::Slice(slice) => slice.length(),
            View::Memory(memory) => memory.length(),
            View::Utf8(slice) => slice.window.length,
        }
    }

    pub(crate) fn sliced(&self, start: usize, length: usize) -> Result<Self, Fault> {
        self.assigned()?;
        Window::new(self.length(), start, length)?;
        Ok(match &self.0 {
            View::Empty(..) => self.clone(),
            View::Slice(slice) => Self::borrowed(slice.slice(start, length)?),
            View::Memory(memory) => Self::persistent(memory.slice(start, length)?),
            View::Utf8(slice) => Self::utf8(slice.slice(start, length)?),
        })
    }

    pub(crate) fn readonly(&self) -> Self {
        match &self.0 {
            View::Empty(kind, element) => Self::empty(
                if kind.is_borrowed() {
                    Kind::ReadOnlySlice
                } else {
                    Kind::ReadOnlyMemory
                },
                element.clone(),
            ),
            View::Slice(slice) => Self::borrowed(slice.as_readonly()),
            View::Memory(memory) => Self::persistent(memory.as_readonly()),
            View::Utf8(_) => self.clone(),
        }
    }

    pub(crate) fn as_slice(&self) -> Self {
        match &self.0 {
            View::Empty(kind, element) => Self::empty(
                match kind {
                    Kind::Memory => Kind::Slice,
                    Kind::ReadOnlyMemory => Kind::ReadOnlySlice,
                    other => *other,
                },
                element.clone(),
            ),
            View::Memory(memory) => Self::borrowed(memory.as_slice()),
            _ => self.clone(),
        }
    }

    pub(crate) fn get(&self, index: usize) -> Result<Value, Fault> {
        match &self.0 {
            View::Slice(slice) => slice.get(index),
            View::Utf8(slice) => slice
                .bytes()
                .get(index)
                .copied()
                .map(Value::Byte)
                .ok_or_else(bounds),
            _ => Err(bounds()),
        }
    }

    pub(crate) fn set(&self, index: usize, value: Value) -> Result<(), Fault> {
        match &self.0 {
            View::Slice(slice) if self.kind() == Kind::Slice => slice.set(index, value),
            _ => Err(Fault::new(
                "memory-view write requires a writable slice element",
            )),
        }
    }
    pub(crate) fn borrowed(slice: Slice) -> Self {
        Self(View::Slice(slice))
    }

    pub(crate) fn persistent(memory: Memory) -> Self {
        Self(View::Memory(memory))
    }

    pub(crate) fn utf8(slice: Utf8Slice) -> Self {
        Self(View::Utf8(slice))
    }

    pub(crate) fn is_borrowed(&self) -> bool {
        self.kind().is_borrowed()
    }

    pub(crate) fn reference(&self) -> Option<&SlotReference> {
        match &self.0 {
            View::Empty(..) => None,
            View::Slice(slice) => Some(&slice.array),
            View::Memory(memory) => Some(&memory.view.array),
            // String has its own immutable shared owner outside the object heap.
            View::Utf8(_) => None,
        }
    }

    pub(crate) fn assigned(&self) -> Result<(), Fault> {
        match self.reference() {
            Some(reference) => reference.assigned(),
            None => Ok(()),
        }
    }

    pub(crate) fn ty(&self) -> Type {
        let (name, element) = match &self.0 {
            View::Empty(kind, element) => return view_type(kind.name(), element.clone()),
            View::Slice(slice) => (
                if slice.array.is_readonly() {
                    "System.ReadOnlySlice"
                } else {
                    "System.Slice"
                },
                slice.array.target(),
            ),
            View::Memory(memory) => (
                if memory.view.array.is_readonly() {
                    "System.ReadOnlyMemory"
                } else {
                    "System.Memory"
                },
                memory.view.array.target(),
            ),
            View::Utf8(_) => return view_type("System.ReadOnlySlice", Type::Byte),
        };
        let Type::Array(element) = element else {
            unreachable!("validated memory view")
        };
        view_type(name, (**element).clone())
    }
}

fn view_type(name: &str, element: Type) -> Type {
    Type::Constructed {
        definition: name.into(),
        arguments: vec![element],
    }
}

impl PartialEq for MemoryView {
    fn eq(&self, other: &Self) -> bool {
        // Host diagnostic equality only; no guest equality operator is added.
        match (&self.0, &other.0) {
            (View::Empty(a, at), View::Empty(b, bt)) => a == b && at == bt,
            (View::Slice(a), View::Slice(b)) => same_window(a, b),
            (View::Memory(a), View::Memory(b)) => same_window(&a.view, &b.view),
            (View::Utf8(a), View::Utf8(b)) => a.owner.same_owner(&b.owner) && a.window == b.window,
            _ => false,
        }
    }
}

fn same_window(a: &Slice, b: &Slice) -> bool {
    a.array == b.array && a.array.is_readonly() == b.array.is_readonly() && a.window == b.window
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        let value = Value::MemoryView(MemoryView::borrowed(view.clone()));
        assert!(
            value
                .check_frame_return(|reference| reference.addresses(&cell))
                .is_err()
        );
        let empty = Value::MemoryView(MemoryView::borrowed(
            view.as_readonly().slice(5, 0).unwrap(),
        ));
        assert!(
            empty
                .check_frame_return(|reference| reference.addresses(&cell))
                .is_err()
        );
        assert!(value.check_frame_return(|_| false).is_ok());
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

    #[test]
    fn borrowed_values_reject_storage_and_erasure_even_with_heap_owners() {
        let mut heap = ManagedHeap::default();
        let slice = Slice::new(allocated(&mut heap), 0, 5).unwrap();
        let borrowed = [
            MemoryView::borrowed(slice.clone()),
            MemoryView::borrowed(slice.as_readonly()),
            MemoryView::borrowed(slice.slice(5, 0).unwrap()),
            MemoryView::utf8(Utf8Slice::new("é".into())),
        ];
        for view in borrowed {
            let value = Value::MemoryView(view);
            let ty = value.ty();
            assert!(
                value.clone().for_storage(&ty).is_ok(),
                "local value copies are permitted"
            );
            assert!(value.clone().erase(&ty).is_err());
            assert!(heap.allocate(value.clone()).is_err());
            assert!(
                crate::arrays::create(ty.clone(), 1, value.clone(), &crate::Limits::default())
                    .is_err()
            );
            let nested = Value::Object {
                ty: Type::Named("Holder".into()),
                fields: vec![value.clone()],
            };
            assert!(nested.ensure_heap_references().is_err());
            assert!(nested.clone().erase(&nested.ty()).is_err());
            assert!(heap.allocate(nested).is_err());
            let field_owner = heap
                .allocate(Value::Object {
                    ty: Type::Named("Holder".into()),
                    fields: vec![Value::Uninitialized(ty.clone())],
                })
                .unwrap();
            assert!(
                heap.address(field_owner)
                    .unwrap()
                    .write_field(0, ty, value)
                    .is_err()
            );
        }
    }

    #[test]
    fn ordinary_frame_tracing_roots_borrowed_views() {
        let mut heap = ManagedHeap::default();
        let owner = allocated(&mut heap);
        let id = owner.allocation_id().unwrap();
        let value = Value::MemoryView(MemoryView::borrowed(Slice::new(owner, 5, 0).unwrap()));
        let local = Slot::cell(value.ty(), Some(value));
        let mut roots = vec![];
        local.borrow().trace_heap(&mut roots);
        assert_eq!(roots, vec![id]);
        heap.collect(roots, CollectionReason::ExplicitRequest)
            .unwrap();
        assert!(heap.contains(id));
        drop(local);
        heap.collect(vec![], CollectionReason::ExplicitRequest)
            .unwrap();
        assert!(!heap.contains(id));
    }

    #[test]
    fn persistent_value_in_heap_field_traces_owner_automatically() {
        let mut heap = ManagedHeap::default();
        let owner = allocated(&mut heap);
        let owner_id = owner.allocation_id().unwrap();
        let memory = Memory::new(&heap, owner, 1, 3).unwrap().as_readonly();
        let value = Value::MemoryView(MemoryView::persistent(memory.clone()));
        assert_eq!(heap.value_roots(&value).unwrap(), vec![owner_id]);
        let erased = value.clone().erase(&value.ty()).unwrap();
        let holder = heap
            .allocate(Value::Object {
                ty: Type::Named("Holder".into()),
                fields: vec![erased],
            })
            .unwrap();
        heap.collect(vec![holder], CollectionReason::ExplicitRequest)
            .unwrap();
        assert_eq!(memory.as_slice().get(0).unwrap(), Value::Int32(20));
        heap.collect(vec![], CollectionReason::ExplicitRequest)
            .unwrap();
        assert!(value.initialized().is_err());
        assert!(memory.as_slice().get(0).is_err());
    }

    #[test]
    fn persistent_value_cannot_publish_another_heaps_owner() {
        let mut first = ManagedHeap::default();
        let mut second = ManagedHeap::default();
        let owner = allocated(&mut first);
        allocated(&mut second);
        let value = Value::MemoryView(MemoryView::persistent(
            Memory::new(&first, owner, 0, 5).unwrap(),
        ));
        assert!(first.value_roots(&value).is_ok());
        assert!(second.value_roots(&value).is_err());
    }

    #[test]
    fn nested_frame_slice_is_rejected_by_the_vm_return_check() {
        let cell = Slot::cell(Type::Array(Box::new(Type::Int32)), Some(array()));
        let view = Slice::new(SlotReference::new(&cell), 0, 5).unwrap();
        let value = Value::MemoryView(MemoryView::borrowed(view));
        // Deliberately fabricated inline nesting exercises defense in depth even
        // though normal stored-value construction rejects this payload earlier.
        let nested = Value::Erased(Box::new(Value::Object {
            ty: Type::Named("Holder".into()),
            fields: vec![value],
        }));
        assert!(
            nested
                .check_frame_return(|reference| reference.addresses(&cell))
                .is_err()
        );
        assert!(
            nested.check_frame_return(|_| false).is_ok(),
            "caller storage can return to its owner"
        );
    }

    #[test]
    fn value_identity_preserves_kind_element_and_readonly_capability() {
        let mut heap = ManagedHeap::default();
        let owner = allocated(&mut heap);
        let slice = Slice::new(owner.clone(), 0, 5).unwrap();
        let value = Value::MemoryView(MemoryView::borrowed(slice.clone()));
        let readonly = Value::MemoryView(MemoryView::borrowed(slice.as_readonly()));
        assert_eq!(value, value.clone());
        assert_ne!(value, readonly);
        assert_eq!(value.ty(), view_type("System.Slice", Type::Int32));
        assert_eq!(
            readonly.ty(),
            view_type("System.ReadOnlySlice", Type::Int32)
        );
        assert!(readonly.for_storage(&value.ty()).is_err());
        let memory = Value::MemoryView(MemoryView::persistent(
            Memory::new(&heap, owner, 0, 5).unwrap(),
        ));
        assert_eq!(memory.ty(), view_type("System.Memory", Type::Int32));
        assert_ne!(memory, value);
    }

    #[test]
    fn fabricated_capture_cannot_hide_borrowed_view_from_escape_checks() {
        let cell = Slot::cell(Type::Array(Box::new(Type::Int32)), Some(array()));
        let view = Value::MemoryView(MemoryView::borrowed(
            Slice::new(SlotReference::new(&cell), 0, 5).unwrap(),
        ));
        // Binding normally rejects this capture. Exercise the value guards even
        // for an internally fabricated binding, including erasure traversal.
        let captured = Value::Function(crate::Function {
            ty: Type::Function(Box::new(crate::metadata::FunctionType {
                parameters: vec![],
                returns: Type::Void,
                no_result: true,
                out_parameters: vec![],
                out_when_true: vec![],
            })),
            view: None,
            identity: std::sync::Arc::new(()),
            target: None,
            native: None,
            receiver: Some(Box::new(view)),
        });
        assert!(captured.ensure_heap_references().is_err());
        assert!(captured.clone().erase(&captured.ty()).is_err());
        assert!(
            captured
                .check_frame_return(|reference| reference.addresses(&cell))
                .is_err()
        );
    }
}

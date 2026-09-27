//! Retaining typed capabilities to interpreter slots. References preserve one
//! stable location without retaining frames. Guest returns independently enforce
//! the lifetime of the owning frame, including for interior field references.
use crate::{Fault, Value, metadata::Type};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard, Weak},
};

#[derive(Debug)]
pub(crate) struct Slot {
    ty: Type,
    value: Option<Value>,
    // Invalidate on every successful value mutation, including interior aliases.
    // Numeric summaries contain no references and never cache quota permission.
    array_usage: std::cell::Cell<Option<crate::arrays::Usage>>,
    writes: u64,
    construction: Option<Vec<Type>>,
    replacements: HashMap<Vec<usize>, u64>,
}
/// One synchronized storage location. Guards must never span a traversal into
/// another managed location (including debugger inspection of a cyclic graph).
/// The lock protects storage, not an entire guest read/modify/write expression.
#[derive(Debug)]
pub(crate) struct SlotStorage(Mutex<Slot>);
impl SlotStorage {
    fn new(slot: Slot) -> Self {
        Self(Mutex::new(slot))
    }
    pub(crate) fn borrow(&self) -> MutexGuard<'_, Slot> {
        self.0.lock().expect("managed slot lock poisoned")
    }
    pub(crate) fn borrow_mut(&self) -> MutexGuard<'_, Slot> {
        self.borrow()
    }
}
pub(crate) type Cell = Arc<SlotStorage>;

impl Slot {
    pub(crate) fn new(ty: Type, value: Option<Value>) -> Cell {
        Arc::new(SlotStorage::new(Self {
            ty,
            value,
            array_usage: std::cell::Cell::new(None),
            writes: 0,
            construction: None,
            replacements: HashMap::new(),
        }))
    }
    pub(crate) fn receiver(cell: &Cell, readonly_value: bool) -> Result<Value, Fault> {
        let mut reference = crate::SlotReference::new(cell);
        reference.assigned()?;
        if matches!(
            cell.borrow().inspect_type(),
            Type::ByRef(_) | Type::ReadOnlyByRef(_)
        ) {
            let value = cell.borrow().get()?.on_stack();
            match &value {
                Value::SlotReference(target)
                | Value::SlotInterface {
                    receiver: target, ..
                } => target.assigned()?,
                _ => {
                    return Err(Fault::new(
                        "managed receiver slot does not contain a managed reference",
                    ));
                }
            }
            return Ok(value);
        }
        if readonly_value {
            reference.restrict_readonly();
        }
        Ok(Value::SlotReference(reference))
    }
    pub(crate) fn construction(ty: Type, fields: Vec<Value>) -> Cell {
        let cell = Self::new(ty.clone(), Some(Value::Object { ty, fields }));
        cell.borrow_mut().construction = Some(vec![]);
        cell
    }
    pub(crate) fn publish(&mut self) -> Result<Value, Fault> {
        let value = self.get()?;
        if let Value::Object { fields, .. } = &value {
            for field in fields {
                field.initialized()?;
            }
        }
        self.construction = None;
        Ok(value)
    }
    pub(crate) fn reset(cell: &Cell) -> Result<(), Fault> {
        if Arc::strong_count(cell) != 1 {
            return Err(Fault::new(
                "cannot reset local while managed references are live",
            ));
        }
        let mut slot = cell.borrow_mut();
        slot.value = None;
        slot.array_usage.set(None);
        slot.replacements.clear();
        Ok(())
    }
    pub(crate) fn trace_heap(&self, roots: &mut Vec<usize>) {
        if let Some(value) = &self.value {
            crate::gc::trace(value, roots);
        }
    }
    pub(crate) fn array_usage(
        &self,
        usage: &mut crate::arrays::Usage,
        limits: &crate::Limits,
    ) -> Result<(), Fault> {
        let measured = if let Some(measured) = self.array_usage.get() {
            measured
        } else {
            let mut measured = crate::arrays::Usage::default();
            if let Some(value) = &self.value {
                // Only successful, complete measurements are reusable. Counts
                // are independent of limits, but failures must never be cached.
                crate::arrays::measure(value, &mut measured, limits)?;
            }
            self.array_usage.set(Some(measured));
            measured
        };
        usage.add(measured, limits)
    }
    pub(crate) fn inspect_type(&self) -> &Type {
        &self.ty
    }
    pub(crate) fn inspect(&self) -> Option<&Value> {
        self.value.as_ref()
    }
    pub(crate) fn get(&self) -> Result<Value, Fault> {
        self.value
            .clone()
            .ok_or_else(|| Fault::new("read of uninitialized slot"))
    }
    pub(crate) fn set(&mut self, value: Value) -> Result<(), Fault> {
        let value = value.for_storage(&self.ty)?;
        if let Some(old) = &self.value {
            crate::arrays::check_replacement(old, &value)?;
        }
        self.writes = self
            .writes
            .checked_add(1)
            .ok_or_else(|| Fault::new("slot write counter exhausted"))?;
        self.value = Some(value);
        self.array_usage.set(None);
        self.replacements.clear();
        self.replacements.insert(vec![], self.writes);
        Ok(())
    }
}

#[derive(Debug, Clone)]
enum Root {
    Frame(Cell),
    Heap {
        identity: usize,
        cell: Weak<SlotStorage>,
    },
}

/// An execution-local managed reference, with no public fabrication API.
#[derive(Debug, Clone)]
pub struct SlotReference {
    target: Type,
    root: Root,
    path: Vec<usize>,
    after_write: Option<u64>,
    readonly: bool,
    construction: Option<ConstructionView>,
}
#[derive(Debug, Clone)]
struct ConstructionView {
    owner: Type,
    base: Option<Type>,
    start: usize,
    end: usize,
}
impl PartialEq for SlotReference {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target
            && self.path == other.path
            && match (&self.root, &other.root) {
                (Root::Frame(a), Root::Frame(b)) => Arc::ptr_eq(a, b),
                (Root::Heap { cell: a, .. }, Root::Heap { cell: b, .. }) => Weak::ptr_eq(a, b),
                _ => false,
            }
    }
}
impl SlotReference {
    pub(crate) fn new(cell: &Cell) -> Self {
        Self {
            target: cell.borrow().ty.clone(),
            root: Root::Frame(Arc::clone(cell)),
            path: vec![],
            after_write: None,
            readonly: false,
            construction: None,
        }
    }
    pub(crate) fn heap(cell: &Cell, identity: usize) -> Self {
        let mut reference = Self::new(cell);
        reference.root = Root::Heap {
            identity,
            cell: Arc::downgrade(cell),
        };
        reference
    }
    /// Allocation identity for diagnostics; frame-backed references have none.
    /// Identities are local to one execution and do not authorize host invocation.
    pub fn allocation_id(&self) -> Option<usize> {
        match self.root {
            Root::Heap { identity, .. } => Some(identity),
            Root::Frame(_) => None,
        }
    }
    pub(crate) fn belongs_to_heap_cell(&self, cell: &Cell) -> bool {
        matches!(&self.root, Root::Heap { cell: root, .. } if Weak::ptr_eq(root, &Arc::downgrade(cell)))
    }
    fn cell(&self) -> Result<Cell, Fault> {
        match &self.root {
            Root::Frame(cell) => Ok(Arc::clone(cell)),
            Root::Heap { cell, .. } => cell
                .upgrade()
                .ok_or_else(|| Fault::new("managed heap reference has expired")),
        }
    }
    pub(crate) fn addresses(&self, cell: &Cell) -> bool {
        matches!(&self.root, Root::Frame(root) if Arc::ptr_eq(root, cell))
    }
    pub(crate) fn debug_path(&self) -> &[usize] {
        &self.path
    }
    pub(crate) fn same_location(&self, other: &Self) -> bool {
        self.path == other.path
            && match (&self.root, &other.root) {
                (Root::Frame(a), Root::Frame(b)) => Arc::ptr_eq(a, b),
                (Root::Heap { cell: a, .. }, Root::Heap { cell: b, .. }) => Weak::ptr_eq(a, b),
                _ => false,
            }
    }
    pub(crate) fn target(&self) -> &Type {
        &self.target
    }
    /// Whether this access view forbids writes to its addressed storage.
    pub fn is_readonly(&self) -> bool {
        self.readonly
    }
    pub(crate) fn restrict_readonly(&mut self) {
        self.readonly = true;
    }
    pub(crate) fn require_writable(&self) -> Result<(), Fault> {
        if self.readonly {
            Err(Fault::new(
                "readonly managed reference cannot be used for writable access",
            ))
        } else {
            Ok(())
        }
    }
    pub(crate) fn constructor_view(
        &self,
        module: &crate::Module,
        owner: &Type,
    ) -> Result<Self, Fault> {
        self.require_writable()?;
        let cell = self.cell()?;
        let slot = cell.borrow();
        if !self.path.is_empty() || slot.construction.is_none() {
            return Err(Fault::new(
                "constructor requires unpublished construction storage",
            ));
        }
        crate::inheritance::require_base(module, &slot.ty, owner)?;
        let base = crate::inheritance::base(module, owner)?;
        let start = base
            .as_ref()
            .map(|t| module.instantiated_fields(t).map(|f| f.len()))
            .transpose()?
            .unwrap_or(0);
        let mut view = self.clone();
        view.target = owner.clone();
        view.construction = Some(ConstructionView {
            owner: owner.clone(),
            base,
            start,
            end: module.instantiated_fields(owner)?.len(),
        });
        Ok(view)
    }
    fn construction_fields(&self) -> Result<Option<ConstructionView>, Fault> {
        let Some(view) = &self.construction else {
            return Ok(None);
        };
        let cell = self.cell()?;
        let slot = cell.borrow();
        let completed = slot
            .construction
            .as_ref()
            .ok_or_else(|| Fault::new("construction capability has expired"))?;
        if view
            .base
            .as_ref()
            .is_some_and(|base| !completed.contains(base))
        {
            return Err(Fault::new(
                "base constructor must complete before field access",
            ));
        }
        Ok(Some(view.clone()))
    }
    pub(crate) fn complete_constructor(&self) -> Result<(), Fault> {
        let view = self
            .construction_fields()?
            .ok_or_else(|| Fault::new("missing construction capability"))?;
        let cell = self.cell()?;
        let mut slot = cell.borrow_mut();
        let Some(Value::Object { fields, .. }) = &slot.value else {
            return Err(Fault::new("missing construction storage"));
        };
        for field in &fields[..view.end] {
            field.initialized()?;
        }
        let completed = slot.construction.as_mut().unwrap();
        if !completed.contains(&view.owner) {
            completed.push(view.owner);
        }
        Ok(())
    }
    pub(crate) fn constructor_completed(&self, owner: &Type) -> Result<bool, Fault> {
        Ok(self
            .cell()?
            .borrow()
            .construction
            .as_ref()
            .is_some_and(|done| done.contains(owner)))
    }
    pub(crate) fn field(&self, index: usize, target: Type) -> Result<Self, Fault> {
        if let Some(view) = self.construction_fields()? {
            if !self.path.is_empty() || index < view.start || index >= view.end {
                return Err(Fault::new("constructor can address only its own fields"));
            }
        } else {
            self.assigned()?;
        }
        if matches!(&target, Type::ByRef(_) | Type::ReadOnlyByRef(_)) {
            return Err(Fault::new("nested managed references are not supported"));
        }
        let mut result = self.clone();
        result.path.push(index);
        result.target = target;
        result.after_write = None;
        {
            let cell = result.cell()?;
            let slot = cell.borrow();
            let value = slot
                .value
                .as_ref()
                .ok_or_else(|| Fault::new("read of uninitialized slot"))?;
            if at_path(value, &result.path)?.ty() != result.target {
                return Err(Fault::new("managed field reference type mismatch"));
            }
        }
        Ok(result)
    }
    pub(crate) fn array_length(&self) -> Result<usize, Fault> {
        self.assigned()?;
        let cell = self.cell()?;
        let slot = cell.borrow();
        let value = at_path(
            slot.value
                .as_ref()
                .ok_or_else(|| Fault::new("uninitialized array"))?,
            &self.path,
        )?;
        let Value::Array { elements, .. } = value else {
            return Err(Fault::new("array operation requires array"));
        };
        Ok(elements.len())
    }
    pub(crate) fn element(&self, index: usize, target: &Type) -> Result<Self, Fault> {
        if self.target != Type::Array(Box::new(target.clone())) {
            return Err(Fault::new("array element type mismatch"));
        }
        if index >= self.array_length()? {
            return Err(Fault::coded(
                crate::FaultCode::IndexOutOfRange,
                "array index out of range",
            ));
        }
        let mut result = self.clone();
        result.path.push(index);
        result.target = target.clone();
        result.after_write = None;
        Ok(result)
    }
    pub(crate) fn output(&self) -> Result<Self, Fault> {
        if self.construction.is_some() {
            return Err(Fault::new(
                "construction storage cannot be an output argument",
            ));
        }
        self.require_writable()?;
        self.require_complete_view()?;
        let mut result = self.clone();
        result.after_write = Some(self.cell()?.borrow().writes);
        Ok(result)
    }
    pub(crate) fn assigned(&self) -> Result<(), Fault> {
        let cell = self.cell()?;
        let slot = cell.borrow();
        if slot.construction.is_some() {
            return Err(Fault::new(
                "cannot publish or call through a receiver during construction",
            ));
        }
        if self.after_write.is_some_and(|baseline| {
            !slot
                .replacements
                .iter()
                .any(|(path, write)| self.path.starts_with(path) && *write > baseline)
        }) {
            return Err(Fault::new("out parameter has not been assigned"));
        }
        if slot.value.is_none() {
            return Err(Fault::new("read of uninitialized slot"));
        }
        at_path(slot.value.as_ref().unwrap(), &self.path)?.initialized()?;
        Ok(())
    }
    pub(crate) fn stored_type(&self) -> Result<Type, Fault> {
        self.assigned()?;
        let cell = self.cell()?;
        let slot = cell.borrow();
        Ok(at_path(slot.value.as_ref().unwrap(), &self.path)?.ty())
    }
    fn require_complete_view(&self) -> Result<(), Fault> {
        // Uninitialized exact slots must remain writable, including out parameters.
        let cell = self.cell()?;
        let slot = cell.borrow();
        if let Some(value) = &slot.value {
            if at_path(value, &self.path)?.ty() != self.target {
                return Err(Fault::new(
                    "whole-value access through a base view would slice derived storage",
                ));
            }
        }
        Ok(())
    }
    pub(crate) fn base_view(&self, module: &crate::Module, target: &Type) -> Result<Self, Fault> {
        self.assigned()?;
        crate::inheritance::require_base(module, &self.target, target)?;
        let mut view = self.clone();
        view.target = target.clone();
        Ok(view)
    }
    /// Runtime-only receiver projection after validated virtual target selection.
    pub(crate) fn dispatch_view(
        &self,
        module: &crate::Module,
        target: &Type,
    ) -> Result<Self, Fault> {
        crate::inheritance::require_base(module, &self.stored_type()?, target)?;
        let mut view = self.clone();
        view.target = target.clone();
        Ok(view)
    }
    pub(crate) fn write_field(
        &self,
        index: usize,
        target: Type,
        value: Value,
    ) -> Result<(), Fault> {
        self.require_writable()?;
        if let Some(view) = self.construction_fields()? {
            if !self.path.is_empty() || index < view.start || index >= view.end {
                return Err(Fault::new("constructor can write only its own fields"));
            }
        } else {
            self.assigned()?;
        }
        let mut field = self.clone();
        field.path.push(index);
        field.target = target;
        field.after_write = None;
        field.write(value)
    }
    pub(crate) fn read_field(&self, index: usize) -> Result<Value, Fault> {
        if let Some(view) = self.construction_fields()? {
            if !self.path.is_empty() || index >= view.end {
                return Err(Fault::new("constructor field read outside owner"));
            }
        } else {
            self.assigned()?;
        }
        let cell = self.cell()?;
        let slot = cell.borrow();
        let Value::Object { fields, .. } = at_path(slot.value.as_ref().unwrap(), &self.path)?
        else {
            return Err(Fault::new("field read requires a record"));
        };
        Ok(fields
            .get(index)
            .ok_or_else(|| Fault::new("field index out of range"))?
            .initialized()?
            .clone())
    }
    pub(crate) fn read(&self) -> Result<Value, Fault> {
        self.assigned()?;
        self.require_complete_view()?;
        let cell = self.cell()?;
        let slot = cell.borrow();
        Ok(at_path(
            slot.value
                .as_ref()
                .ok_or_else(|| Fault::new("read of uninitialized slot"))?,
            &self.path,
        )?
        .clone())
    }
    pub(crate) fn write(&self, value: Value) -> Result<(), Fault> {
        self.require_writable()?;
        if self.construction.is_some() {
            self.construction_fields()?;
            if self.path.len() != 1 {
                return Err(Fault::new(
                    "constructor must initialize individual own fields",
                ));
            }
        }
        self.require_complete_view()?;
        if !self.path.is_empty() || self.allocation_id().is_some() {
            value.ensure_heap_references()?;
        }
        let cell = self.cell()?;
        let mut slot = cell.borrow_mut();
        if self.path.is_empty() {
            return slot.set(value);
        }
        let value = value.for_storage(&self.target)?;
        let next_write = slot
            .writes
            .checked_add(1)
            .ok_or_else(|| Fault::new("slot write counter exhausted"))?;
        let mut field = slot
            .value
            .as_mut()
            .ok_or_else(|| Fault::new("read of uninitialized slot"))?;
        for index in &self.path {
            let fields = match field {
                Value::Object { fields, .. }
                | Value::Array {
                    elements: fields, ..
                } => fields,
                _ => return Err(Fault::new("managed path requires record or array")),
            };
            field = fields
                .get_mut(*index)
                .ok_or_else(|| Fault::new("field index out of range"))?;
        }
        if field.ty() != self.target {
            return Err(Fault::new("managed field reference type mismatch"));
        }
        crate::arrays::check_replacement(field, &value)?;
        *field = value;
        slot.array_usage.set(None);
        slot.writes = next_write;
        slot.replacements
            .retain(|path, _| !path.starts_with(&self.path));
        slot.replacements.insert(self.path.clone(), next_write);
        Ok(())
    }
}

fn at_path<'a>(mut value: &'a Value, path: &[usize]) -> Result<&'a Value, Fault> {
    for index in path {
        let fields = match value {
            Value::Object { fields, .. }
            | Value::Array {
                elements: fields, ..
            } => fields,
            _ => return Err(Fault::new("managed path requires record or array")),
        };
        value = fields
            .get(*index)
            .ok_or_else(|| Fault::new("field index out of range"))?;
    }
    Ok(value)
}

pub(crate) fn contains(ty: &Type) -> bool {
    match ty {
        Type::ByRef(_) | Type::ReadOnlyByRef(_) => true,
        Type::Array(t) | Type::ArrayRef(t) | Type::Ptr(t) | Type::InterfaceRef(t) => contains(t),
        Type::Constructed { arguments, .. } | Type::Scoped { arguments, .. } => {
            arguments.iter().any(contains)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn managed_values_are_send_and_sync_without_unsafe_markers() {
        fn require<T: Send + Sync>() {}
        require::<crate::Value>();
        require::<crate::SlotReference>();
        require::<crate::ManagedHeap>();
    }

    #[test]
    fn native_threads_mutate_the_same_heap_object_through_interior_aliases() {
        let mut heap = crate::ManagedHeap::default();
        let id = heap
            .allocate(Value::Object {
                ty: Type::from_name("Pair"),
                fields: vec![Value::Int32(0), Value::Int32(0)],
            })
            .unwrap();
        let object = heap.address(id).unwrap();
        let first = object.field(0, Type::Int32).unwrap();
        let second = object.field(1, Type::Int32).unwrap();
        let start = std::sync::Barrier::new(3);
        std::thread::scope(|scope| {
            for field in [first, second] {
                let start = &start;
                scope.spawn(move || {
                    start.wait();
                    for n in 1..=2_000 {
                        field.write(Value::Int32(n)).unwrap();
                        assert_eq!(field.read().unwrap(), Value::Int32(n));
                    }
                    assert_eq!(field.allocation_id(), Some(id));
                });
            }
            start.wait();
        });
        assert_eq!(object.read_field(0).unwrap(), Value::Int32(2_000));
        assert_eq!(object.read_field(1).unwrap(), Value::Int32(2_000));
        // Native sharing does not turn a weak heap handle into a GC root.
        heap.collect(vec![], crate::CollectionReason::ExplicitRequest)
            .unwrap();
        assert!(object.read().unwrap_err().message.contains("expired"));
    }

    #[test]
    fn concurrent_record_replacement_and_reads_do_not_observe_torn_values() {
        let ty = Type::from_name("Pair");
        let pair = |n| Value::Object {
            ty: ty.clone(),
            fields: vec![Value::Int32(n), Value::Int32(n)],
        };
        let cell = Slot::new(ty.clone(), Some(pair(0)));
        let reference = crate::SlotReference::new(&cell);
        let start = std::sync::Barrier::new(2);
        std::thread::scope(|scope| {
            scope.spawn(|| {
                start.wait();
                for n in 1..=2_000 {
                    reference.write(pair(n)).unwrap();
                }
            });
            start.wait();
            for _ in 0..2_000 {
                let Value::Object { fields, .. } = reference.read().unwrap() else {
                    panic!("expected record");
                };
                assert_eq!(fields[0], fields[1]);
            }
        });
        assert_eq!(reference.read().unwrap(), pair(2_000));
    }

    #[test]
    fn cross_thread_array_mutation_invalidates_payload_summary() {
        let cell = Slot::new(Type::Array(Box::new(Type::String)), Some(text_array("a")));
        budget(&cell, 1, std::mem::size_of::<Value>() + 1).unwrap();
        let element = crate::SlotReference::new(&cell)
            .element(0, &Type::String)
            .unwrap();
        std::thread::spawn(move || {
            element.write(Value::String("longer".into())).unwrap();
        })
        .join()
        .unwrap();
        assert!(budget(&cell, 1, std::mem::size_of::<Value>() + 1).is_err());
        budget(&cell, 1, std::mem::size_of::<Value>() + 6).unwrap();
    }

    fn text_array(text: &str) -> Value {
        Value::Array {
            element: Type::String,
            elements: vec![Value::String(text.into())],
        }
    }

    fn budget(cell: &Cell, elements: usize, bytes: usize) -> Result<(), Fault> {
        cell.borrow().array_usage(
            &mut crate::arrays::Usage::default(),
            &crate::Limits {
                array_elements: elements,
                array_bytes: bytes,
                ..Default::default()
            },
        )
    }

    #[test]
    fn payload_summary_tracks_replacement_failed_stores_and_reset() {
        let size = std::mem::size_of::<Value>();
        let cell = Slot::new(Type::Array(Box::new(Type::String)), Some(text_array("a")));
        budget(&cell, 1, size + 1).unwrap();
        cell.borrow_mut().set(text_array("longer")).unwrap();
        assert_eq!(
            budget(&cell, 1, size + 1).unwrap_err().code,
            crate::FaultCode::ArrayLimitExceeded
        );
        budget(&cell, 1, size + 6).unwrap();
        assert!(cell.borrow_mut().set(Value::Int32(0)).is_err());
        assert!(budget(&cell, 1, size + 1).is_err());
        budget(&cell, 1, size + 6).unwrap();
        Slot::reset(&cell).unwrap();
        budget(&cell, 0, 0).unwrap();
        cell.borrow_mut().set(text_array("x")).unwrap();
        assert!(budget(&cell, 0, 0).is_err());
        budget(&cell, 1, size + 1).unwrap();
    }

    #[test]
    fn payload_summary_tracks_aliased_element_and_nested_field_writes() {
        let size = std::mem::size_of::<Value>();
        let array_type = Type::Array(Box::new(Type::String));
        let owner = Type::from_name("Container");
        let cell = Slot::new(
            owner.clone(),
            Some(Value::Object {
                ty: owner,
                fields: vec![text_array("a")],
            }),
        );
        let field = SlotReference::new(&cell).field(0, array_type).unwrap();
        let element = field.element(0, &Type::String).unwrap();
        let alias = element.clone();
        budget(&cell, 1, size + 1).unwrap();
        alias.write(Value::String("Café".into())).unwrap();
        assert!(budget(&cell, 1, size + 1).is_err());
        budget(&cell, 1, size + 5).unwrap();
        assert!(element.write(Value::Int32(0)).is_err());
        budget(&cell, 1, size + 5).unwrap();
        field.write(text_array("z")).unwrap();
        budget(&cell, 1, size + 1).unwrap();
    }

    #[test]
    fn payload_summary_rechecks_limits_and_accumulates_all_roots() {
        let size = std::mem::size_of::<Value>();
        let cell = Slot::new(Type::Array(Box::new(Type::String)), Some(text_array("abc")));
        // An initial failed measurement must not publish incomplete counts.
        assert!(budget(&cell, 0, 0).is_err());
        budget(&cell, 1, size + 3).unwrap();
        // A successful cache fill must not remember permission from a larger limit.
        assert!(budget(&cell, 1, size + 2).is_err());
        let limits = crate::Limits {
            array_elements: 1,
            ..Default::default()
        };
        let mut usage = crate::arrays::Usage::default();
        cell.borrow().array_usage(&mut usage, &limits).unwrap();
        assert_eq!(
            cell.borrow()
                .array_usage(&mut usage, &limits)
                .unwrap_err()
                .code,
            crate::FaultCode::ArrayLimitExceeded
        );
    }

    #[test]
    fn heap_payload_summary_tracks_native_style_replacement_and_does_not_root_storage() {
        let size = std::mem::size_of::<Value>();
        let mut heap = crate::ManagedHeap::default();
        let id = heap.allocate(text_array("a")).unwrap();
        let address = heap.address(id).unwrap();
        let limits = crate::Limits {
            array_elements: 1,
            array_bytes: size + 1,
            ..Default::default()
        };
        heap.array_usage(&mut crate::arrays::Usage::default(), &limits)
            .unwrap();
        // Native receive replaces the whole destination array through this path.
        address.write(text_array("abc")).unwrap();
        assert!(
            heap.array_usage(&mut crate::arrays::Usage::default(), &limits)
                .is_err()
        );
        let limits = crate::Limits {
            array_bytes: size + 3,
            ..limits
        };
        heap.array_usage(&mut crate::arrays::Usage::default(), &limits)
            .unwrap();
        address
            .element(0, &Type::String)
            .unwrap()
            .write(Value::String("longer".into()))
            .unwrap();
        assert!(
            heap.array_usage(&mut crate::arrays::Usage::default(), &limits)
                .is_err()
        );
        heap.collect(vec![], crate::CollectionReason::AllocationPressure)
            .unwrap();
        let limits = crate::Limits {
            array_elements: 0,
            array_bytes: 0,
            ..limits
        };
        heap.array_usage(&mut crate::arrays::Usage::default(), &limits)
            .unwrap();
        assert_eq!(heap.len(), 0);
        assert!(address.read().is_err());
    }

    #[test]
    fn references_retain_storage_and_failed_stores_do_not_fulfill_outputs() {
        let cell = Slot::new(Type::Int32, Some(Value::Int32(7)));
        let reference = SlotReference::new(&cell);
        let output = reference.output().unwrap();
        assert!(output.write(Value::String("wrong".into())).is_err());
        assert!(output.assigned().is_err());
        assert_eq!(reference.read().unwrap(), Value::Int32(7));
        reference.write(Value::Int32(42)).unwrap();
        assert_eq!(output.read().unwrap(), Value::Int32(42));
        let observer = Arc::downgrade(&cell);
        drop(cell);
        assert_eq!(reference.read().unwrap(), Value::Int32(42));
        reference.write(Value::Int32(0)).unwrap();
        assert_eq!(output.read().unwrap(), Value::Int32(0));
        drop(reference);
        assert!(observer.upgrade().is_some());
        drop(output);
        assert!(observer.upgrade().is_none());
    }

    #[test]
    fn reference_replacement_releases_only_the_replaced_retention() {
        let first = Slot::new(Type::String, Some(Value::String("first".into())));
        let second = Slot::new(Type::String, Some(Value::String("second".into())));
        let first_lifetime = Arc::downgrade(&first);
        let second_lifetime = Arc::downgrade(&second);
        let alias = SlotReference::new(&first);
        let holder = Slot::new(
            Type::ByRef(Box::new(Type::String)),
            Some(Value::SlotReference(alias.clone())),
        );
        drop(first);
        let copy = holder.borrow().get().unwrap();
        holder.borrow_mut().set(copy).unwrap();
        assert!(holder.borrow_mut().set(Value::Int32(0)).is_err());
        assert_eq!(alias.read().unwrap(), Value::String("first".into()));
        holder
            .borrow_mut()
            .set(Value::SlotReference(SlotReference::new(&second)))
            .unwrap();
        drop(second);
        assert!(first_lifetime.upgrade().is_some());
        drop(alias);
        assert!(first_lifetime.upgrade().is_none());
        assert!(second_lifetime.upgrade().is_some());
        drop(holder);
        assert!(second_lifetime.upgrade().is_none());
    }
}

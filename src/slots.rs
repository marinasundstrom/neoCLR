//! Retaining typed capabilities to interpreter slots. References preserve one
//! stable location without retaining frames. Guest returns independently enforce
//! the lifetime of the owning frame, including for interior field references.
use crate::{Fault, Value, metadata::Type};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

#[derive(Debug)]
pub(crate) struct Slot {
    ty: Type,
    value: Option<Value>,
    writes: u64,
    construction: Option<Vec<Type>>,
    replacements: HashMap<Vec<usize>, u64>,
}
pub(crate) type Cell = Rc<RefCell<Slot>>;

impl Slot {
    pub(crate) fn new(ty: Type, value: Option<Value>) -> Cell {
        Rc::new(RefCell::new(Self {
            ty,
            value,
            writes: 0,
            construction: None,
            replacements: HashMap::new(),
        }))
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
        if Rc::strong_count(cell) != 1 {
            return Err(Fault::new(
                "cannot reset local while managed references are live",
            ));
        }
        let mut slot = cell.borrow_mut();
        slot.value = None;
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
        if let Some(value) = &self.value {
            crate::arrays::measure(value, usage, limits)?;
        }
        Ok(())
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
        cell: Weak<RefCell<Slot>>,
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
                (Root::Frame(a), Root::Frame(b)) => Rc::ptr_eq(a, b),
                (Root::Heap { cell: a, .. }, Root::Heap { cell: b, .. }) => Weak::ptr_eq(a, b),
                _ => false,
            }
    }
}
impl SlotReference {
    pub(crate) fn new(cell: &Cell) -> Self {
        Self {
            target: cell.borrow().ty.clone(),
            root: Root::Frame(Rc::clone(cell)),
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
            cell: Rc::downgrade(cell),
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
        matches!(&self.root, Root::Heap { cell: root, .. } if Weak::ptr_eq(root, &Rc::downgrade(cell)))
    }
    fn cell(&self) -> Result<Cell, Fault> {
        match &self.root {
            Root::Frame(cell) => Ok(Rc::clone(cell)),
            Root::Heap { cell, .. } => cell
                .upgrade()
                .ok_or_else(|| Fault::new("managed heap reference has expired")),
        }
    }
    pub(crate) fn addresses(&self, cell: &Cell) -> bool {
        matches!(&self.root, Root::Frame(root) if Rc::ptr_eq(root, cell))
    }
    pub(crate) fn debug_path(&self) -> &[usize] {
        &self.path
    }
    pub(crate) fn same_location(&self, other: &Self) -> bool {
        self.path == other.path
            && match (&self.root, &other.root) {
                (Root::Frame(a), Root::Frame(b)) => Rc::ptr_eq(a, b),
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
            return Err(Fault::new("array index out of range"));
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
        Type::Array(t) | Type::Ptr(t) | Type::InterfaceRef(t) => contains(t),
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
    fn references_retain_storage_and_failed_stores_do_not_fulfill_outputs() {
        let cell = Slot::new(Type::Int32, Some(Value::Int32(7)));
        let reference = SlotReference::new(&cell);
        let output = reference.output().unwrap();
        assert!(output.write(Value::String("wrong".into())).is_err());
        assert!(output.assigned().is_err());
        assert_eq!(reference.read().unwrap(), Value::Int32(7));
        reference.write(Value::Int32(42)).unwrap();
        assert_eq!(output.read().unwrap(), Value::Int32(42));
        let observer = Rc::downgrade(&cell);
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
        let first_lifetime = Rc::downgrade(&first);
        let second_lifetime = Rc::downgrade(&second);
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

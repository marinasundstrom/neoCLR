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
    replacements: HashMap<Vec<usize>, u64>,
}
pub(crate) type Cell = Rc<RefCell<Slot>>;

impl Slot {
    pub(crate) fn new(ty: Type, value: Option<Value>) -> Cell {
        Rc::new(RefCell::new(Self {
            ty,
            value,
            writes: 0,
            replacements: HashMap::new(),
        }))
    }
    pub(crate) fn trace_heap(&self, roots: &mut Vec<usize>) {
        if let Some(value) = &self.value {
            crate::gc::trace(value, roots);
        }
    }
    pub(crate) fn get(&self) -> Result<Value, Fault> {
        self.value
            .clone()
            .ok_or_else(|| Fault::new("read of uninitialized slot"))
    }
    pub(crate) fn set(&mut self, value: Value) -> Result<(), Fault> {
        let value = value.for_storage(&self.ty)?;
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
    pub(crate) fn target(&self) -> &Type {
        &self.target
    }
    pub(crate) fn field(&self, index: usize, target: Type) -> Result<Self, Fault> {
        self.assigned()?;
        if contains(&target) {
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
    pub(crate) fn output(&self) -> Result<Self, Fault> {
        let mut result = self.clone();
        result.after_write = Some(self.cell()?.borrow().writes);
        Ok(result)
    }
    pub(crate) fn assigned(&self) -> Result<(), Fault> {
        let cell = self.cell()?;
        let slot = cell.borrow();
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
        Ok(())
    }
    pub(crate) fn read(&self) -> Result<Value, Fault> {
        self.assigned()?;
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
            let Value::Object { fields, .. } = field else {
                return Err(Fault::new("managed field reference requires record"));
            };
            field = fields
                .get_mut(*index)
                .ok_or_else(|| Fault::new("field index out of range"))?;
        }
        if field.ty() != self.target {
            return Err(Fault::new("managed field reference type mismatch"));
        }
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
        let Value::Object { fields, .. } = value else {
            return Err(Fault::new("managed field reference requires record"));
        };
        value = fields
            .get(*index)
            .ok_or_else(|| Fault::new("field index out of range"))?;
    }
    Ok(value)
}

pub(crate) fn contains(ty: &Type) -> bool {
    match ty {
        Type::ByRef(_) => true,
        Type::Ptr(t) | Type::InterfaceRef(t) => contains(t),
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

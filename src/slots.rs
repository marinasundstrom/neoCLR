//! Non-owning typed capabilities to interpreter slots. Weak identity prevents
//! keeping a frame alive or accidentally aliasing a subsequently reused slot.
use crate::{Fault, Value, metadata::Type};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

#[derive(Debug)]
pub(crate) struct Slot {
    ty: Type,
    value: Option<Value>,
    writes: u64,
}
pub(crate) type Cell = Rc<RefCell<Slot>>;

impl Slot {
    pub(crate) fn new(ty: Type, value: Option<Value>) -> Cell {
        Rc::new(RefCell::new(Self {
            ty,
            value,
            writes: 0,
        }))
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
        Ok(())
    }
}

/// An execution-local managed reference, with no public fabrication API.
#[derive(Debug, Clone)]
pub struct SlotReference {
    target: Type,
    slot: Weak<RefCell<Slot>>,
    after_write: Option<u64>,
}
impl PartialEq for SlotReference {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target && Weak::ptr_eq(&self.slot, &other.slot)
    }
}
impl SlotReference {
    pub(crate) fn new(cell: &Cell) -> Self {
        Self {
            target: cell.borrow().ty.clone(),
            slot: Rc::downgrade(cell),
            after_write: None,
        }
    }
    pub(crate) fn target(&self) -> &Type {
        &self.target
    }
    fn cell(&self) -> Result<Cell, Fault> {
        self.slot
            .upgrade()
            .ok_or_else(|| Fault::new("expired managed slot reference"))
    }
    pub(crate) fn output(&self) -> Result<Self, Fault> {
        let mut result = self.clone();
        result.after_write = Some(self.cell()?.borrow().writes);
        Ok(result)
    }
    pub(crate) fn assigned(&self) -> Result<(), Fault> {
        let cell = self.cell()?;
        let slot = cell.borrow();
        if self
            .after_write
            .is_some_and(|baseline| slot.writes <= baseline)
        {
            return Err(Fault::new("out parameter has not been assigned"));
        }
        if slot.value.is_none() {
            return Err(Fault::new("read of uninitialized slot"));
        }
        Ok(())
    }
    pub(crate) fn read(&self) -> Result<Value, Fault> {
        self.assigned()?;
        self.cell()?.borrow().get()
    }
    pub(crate) fn write(&self, value: Value) -> Result<(), Fault> {
        self.cell()?.borrow_mut().set(value)
    }
}

pub(crate) fn contains(ty: &Type) -> bool {
    match ty {
        Type::ByRef(_) => true,
        Type::Ptr(t) | Type::Ref(t) | Type::InterfaceRef(t) => contains(t),
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
    fn references_do_not_retain_storage_and_failed_stores_do_not_fulfill_outputs() {
        let cell = Slot::new(Type::Int32, Some(Value::Int32(7)));
        let reference = SlotReference::new(&cell);
        let output = reference.output().unwrap();
        assert!(output.write(Value::String("wrong".into())).is_err());
        assert!(output.assigned().is_err());
        assert_eq!(reference.read().unwrap(), Value::Int32(7));
        reference.write(Value::Int32(42)).unwrap();
        assert_eq!(output.read().unwrap(), Value::Int32(42));
        drop(cell);
        assert!(reference.read().is_err());
        assert!(reference.write(Value::Int32(0)).is_err());
        assert!(output.assigned().is_err());
    }
}

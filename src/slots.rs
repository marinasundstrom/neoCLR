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
}
pub(crate) type Cell = Rc<RefCell<Slot>>;

impl Slot {
    pub(crate) fn new(ty: Type, value: Option<Value>) -> Cell {
        Rc::new(RefCell::new(Self { ty, value }))
    }
    pub(crate) fn get(&self) -> Result<Value, Fault> {
        self.value
            .clone()
            .ok_or_else(|| Fault::new("read of uninitialized slot"))
    }
    pub(crate) fn set(&mut self, value: Value) -> Result<(), Fault> {
        self.value = Some(value.for_storage(&self.ty)?);
        Ok(())
    }
}

/// An execution-local managed reference, with no public fabrication API.
#[derive(Debug, Clone)]
pub struct SlotReference {
    target: Type,
    slot: Weak<RefCell<Slot>>,
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
    pub(crate) fn read(&self) -> Result<Value, Fault> {
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

use crate::metadata::Type;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Internal reserved array slot; guest element reads fault until initialized.
    Uninitialized(Type),
    Void,
    Single(f32),
    Double(f64),
    Int32(i32),
    SByte(i8),
    Byte(u8),
    Int16(i16),
    UInt16(u16),
    Char(u16),
    UInt32(u32),
    Int64(i64),
    UInt64(u64),

    IntPtr(isize),
    UIntPtr(usize),
    Boolean(bool),
    String(String),
    Error(String),
    /// Interpreter storage for explicit erasure, not a guest heap reference.
    Erased(Box<Value>),
    /// Owned metadata snapshot, not an arbitrary-value container or native pointer.
    RuntimeTypeHandle(Box<crate::TypeDescriptor>),
    Array {
        element: Type,
        elements: Vec<Value>,
    },
    Object {
        ty: Type,
        fields: Vec<Value>,
    },
    SlotReference(crate::SlotReference),
    Pointer(crate::memory::Pointer),
    /// An interface projection retaining its managed concrete slot.
    SlotInterface {
        interface: Type,
        receiver: crate::SlotReference,
    },
    InterfaceRef {
        interface: Type,
        receiver: crate::memory::Pointer,
    },
}

impl Value {
    pub(crate) fn initialized(&self) -> Result<&Self, crate::Fault> {
        if matches!(self, Self::Uninitialized(_)) {
            Err(crate::Fault::new("read of uninitialized array element"))
        } else {
            Ok(self)
        }
    }
    /// Values embedded in fields, erased payloads or heap storage may only carry
    /// heap-backed references. A scoped reference cannot acquire a longer lifetime.
    pub(crate) fn ensure_heap_references(&self) -> Result<(), crate::Fault> {
        let mut pending = vec![self];
        while let Some(value) = pending.pop() {
            match value {
                Self::SlotReference(reference)
                | Self::SlotInterface {
                    receiver: reference,
                    ..
                } => {
                    if reference.allocation_id().is_none() {
                        return Err(crate::Fault::new(
                            "frame-backed references cannot escape into stored values",
                        ));
                    }
                    reference.assigned()?;
                }
                Self::Object { fields, .. }
                | Self::Array {
                    elements: fields, ..
                } => pending.extend(fields),
                Self::Erased(value) => pending.push(value),
                _ => (),
            }
        }
        Ok(())
    }

    pub(crate) fn erase(self, ty: &Type) -> Result<Self, crate::Fault> {
        let value = self.for_storage(ty)?;
        // Erasure permits recursive value shapes. Bound their copy/drop depth before
        // installing another wrapper, without following native pointers or Ref handles.
        let mut pending = vec![(&value, 1usize)];
        let mut remaining = 16_384usize;
        while let Some((item, depth)) = pending.pop() {
            if depth > 64 || remaining == 0 {
                return Err(crate::Fault::new(
                    "erased value exceeds depth or complexity limit",
                ));
            }
            remaining -= 1;
            match item {
                Self::SlotReference(reference)
                | Self::SlotInterface {
                    receiver: reference,
                    ..
                } if reference.allocation_id().is_none() => {
                    return Err(crate::Fault::new(
                        "frame-backed references cannot be erased",
                    ));
                }
                Self::Erased(payload) => {
                    pending.push((payload, depth + 1));
                }
                Self::Object { fields, .. }
                | Self::Array {
                    elements: fields, ..
                } => {
                    pending.extend(fields.iter().map(|field| (field, depth + 1)));
                }
                _ => {}
            }
        }
        Ok(Self::Erased(Box::new(value)))
    }

    pub fn ty(&self) -> Type {
        match self {
            Self::Uninitialized(ty) => ty.clone(),
            Self::Void => Type::Void,
            Self::Single(_) => Type::Single,
            Self::Double(_) => Type::Double,
            Self::Int32(_) => Type::Int32,
            Self::SByte(_) => Type::SByte,
            Self::Byte(_) => Type::Byte,
            Self::Int16(_) => Type::Int16,
            Self::UInt16(_) => Type::UInt16,
            Self::Char(_) => Type::Char,
            Self::UInt32(_) => Type::UInt32,
            Self::Int64(_) => Type::Int64,
            Self::UInt64(_) => Type::UInt64,
            Self::IntPtr(_) => Type::IntPtr,
            Self::UIntPtr(_) => Type::UIntPtr,
            Self::Boolean(_) => Type::Boolean,
            Self::String(_) => Type::String,
            Self::Error(_) => Type::Error,
            Self::Erased(_) => Type::Value,
            Self::RuntimeTypeHandle(_) => Type::RuntimeTypeHandle,
            Self::Object { ty, .. } => ty.clone(),
            Self::Array { element, .. } => Type::Array(Box::new(element.clone())),
            Self::SlotInterface { interface, .. } => Type::ByRef(Box::new(interface.clone())),
            Self::InterfaceRef { interface, .. } => Type::InterfaceRef(Box::new(interface.clone())),
            Self::SlotReference(reference) => Type::ByRef(Box::new(reference.target().clone())),
            Self::Pointer(pointer) => Type::Ptr(Box::new(pointer.target.clone())),
        }
    }

    /// Storage signatures remain precise; small integers load as Int32.
    pub(crate) fn on_stack(self) -> Self {
        match self {
            Self::Single(n) => Self::Double(n as f64),
            Self::SByte(n) => Self::Int32(n as _),
            Self::Byte(n) => Self::Int32(n as _),
            Self::Int16(n) => Self::Int32(n as _),
            Self::UInt16(n) => Self::Int32(n as _),
            Self::Char(n) => Self::Int32(n as _),
            Self::UInt32(n) => Self::Int32(n as _),
            Self::UInt64(n) => Self::Int64(n as _),
            value => value,
        }
    }

    /// CLI integer storage truncates a stack integer to the destination width.
    pub(crate) fn for_storage(self, ty: &Type) -> Result<Self, crate::Fault> {
        self.initialized()?;
        let value = match (&self, ty) {
            (Self::Double(n), Type::Single) => Self::Single(*n as f32),
            (Self::Int32(n), Type::SByte) => Self::SByte(*n as i8),
            (Self::Int32(n), Type::Byte) => Self::Byte(*n as u8),
            (Self::Int32(n), Type::Int16) => Self::Int16(*n as i16),
            (Self::Int32(n), Type::UInt16) => Self::UInt16(*n as u16),
            (Self::Int32(n), Type::Char) => Self::Char(*n as u16),
            (Self::Int32(n), Type::UInt32) => Self::UInt32(*n as u32),
            (Self::Int64(n), Type::UInt64) => Self::UInt64(*n as u64),
            _ => self,
        };
        if value.ty() == *ty {
            Ok(value)
        } else {
            Err(crate::Fault::new(format!(
                "expected {ty:?}, got {:?}",
                value.ty()
            )))
        }
    }
}

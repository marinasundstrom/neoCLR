use crate::metadata::{Case, Type};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Void,
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
    Object {
        name: String,
        fields: Vec<Value>,
    },
    Union {
        ty: Type,
        case: Case,
        payload: Box<Value>,
    },
    Pointer(crate::memory::Pointer),
    Reference {
        index: usize,
        target: Type,
    },
}

impl Value {
    pub fn ty(&self) -> Type {
        match self {
            Self::Void => Type::Void,
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
            Self::Object { name, .. } => Type::Named(name.clone()),
            Self::Union { ty, .. } => ty.clone(),
            Self::Pointer(pointer) => Type::Ptr(Box::new(pointer.target.clone())),
            Self::Reference { target, .. } => Type::Ref(Box::new(target.clone())),
        }
    }

    /// Storage signatures remain precise; small integers load as Int32.
    pub(crate) fn on_stack(self) -> Self {
        match self {
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
        let value = match (&self, ty) {
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

    pub fn result(value: Value, success: Type, error: Type, case: Case) -> Self {
        Self::Union {
            ty: Type::Result(Box::new(success), Box::new(error)),
            case,
            payload: Box::new(value),
        }
    }
}

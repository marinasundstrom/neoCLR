use crate::metadata::{Case, Type};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Void,
    Int32(i32),
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

    pub fn result(value: Value, success: Type, error: Type, case: Case) -> Self {
        Self::Union {
            ty: Type::Result(Box::new(success), Box::new(error)),
            case,
            payload: Box::new(value),
        }
    }
}

use serde::{Deserialize, Serialize};

/// No value-type/reference-type bit: Ref is an explicit storage capability.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    Void,
    Single,
    Double,
    Int32,
    SByte,
    Byte,
    Int16,
    UInt16,
    Char,
    UInt32,
    Int64,
    UInt64,

    IntPtr,
    UIntPtr,
    Boolean,
    String,
    Error,
    Named(String),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Ref(Box<Type>),
    /// Fundamental unmanaged pointer signature; no ownership policy is implied.
    Ptr(Box<Type>),
}

impl Type {
    /// Primitive spellings are signature aliases for canonical System type identities.
    pub fn from_name(name: &str) -> Self {
        match name {
            "Void" | "void" | "System.Void" => Self::Void,
            "Single" | "single" | "float32" | "System.Single" => Self::Single,
            "Double" | "double" | "float64" | "System.Double" => Self::Double,
            "Int32" | "int32" | "int" | "System.Int32" => Self::Int32,
            "SByte" | "int8" | "System.SByte" => Self::SByte,
            "Byte" | "uint8" | "System.Byte" => Self::Byte,
            "Int16" | "int16" | "System.Int16" => Self::Int16,
            "UInt16" | "uint16" | "System.UInt16" => Self::UInt16,
            "Char" | "char" | "System.Char" => Self::Char,
            "UInt32" | "uint32" | "System.UInt32" => Self::UInt32,
            "Int64" | "int64" | "System.Int64" => Self::Int64,
            "UInt64" | "uint64" | "System.UInt64" => Self::UInt64,
            "IntPtr" | "nint" | "System.IntPtr" => Self::IntPtr,
            "UIntPtr" | "nuint" | "System.UIntPtr" => Self::UIntPtr,
            "Boolean" | "boolean" | "bool" | "System.Boolean" => Self::Boolean,
            "String" | "string" | "System.String" => Self::String,
            "Error" | "System.Error" => Self::Error,
            _ => Self::Named(name.into()),
        }
    }

    pub fn definition_name(&self) -> Option<&str> {
        match self {
            Self::Void => Some("System.Void"),
            Self::Single => Some("System.Single"),
            Self::Double => Some("System.Double"),
            Self::Int32 => Some("System.Int32"),
            Self::SByte => Some("System.SByte"),
            Self::Byte => Some("System.Byte"),
            Self::Int16 => Some("System.Int16"),
            Self::UInt16 => Some("System.UInt16"),
            Self::Char => Some("System.Char"),
            Self::UInt32 => Some("System.UInt32"),
            Self::Int64 => Some("System.Int64"),
            Self::UInt64 => Some("System.UInt64"),
            Self::IntPtr => Some("System.IntPtr"),
            Self::UIntPtr => Some("System.UIntPtr"),
            Self::Boolean => Some("System.Boolean"),
            Self::String => Some("System.String"),
            Self::Error => Some("System.Error"),
            Self::Named(name) => Some(name),
            _ => None,
        }
    }

    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            Self::Void
                | Self::Single
                | Self::Double
                | Self::Int32
                | Self::SByte
                | Self::Byte
                | Self::Int16
                | Self::UInt16
                | Self::Char
                | Self::UInt32
                | Self::Int64
                | Self::UInt64
                | Self::IntPtr
                | Self::UIntPtr
                | Self::Boolean
                | Self::String
                | Self::Error
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Module {
    pub format: u32,
    pub name: String,
    #[serde(default)]
    pub entry: String,
    #[serde(default)]
    pub types: Vec<TypeDef>,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeDef {
    pub name: String,
    pub fields: Vec<Field>,
    #[serde(default)]
    pub representation: Representation,
}

/// Representation is independent of ownership and reference identity.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Representation {
    #[default]
    Record,
    Runtime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Function {
    pub name: String,
    #[serde(default)]
    pub owner: Option<Type>,
    #[serde(default)]
    pub instance: bool,
    #[serde(default)]
    pub parameters: Vec<Type>,
    /// Optional names aligned with declared parameters (excluding the receiver).
    #[serde(default)]
    pub parameter_names: Vec<Option<String>>,
    pub returns: Type,
    #[serde(default)]
    pub locals: Vec<Type>,
    #[serde(default)]
    pub local_names: Vec<Option<String>>,
    /// CLR MethodImplAttributes values: IL = 0, InternalCall = 0x1000.
    #[serde(default)]
    pub impl_flags: u16,
    #[serde(default)]
    pub body: Vec<Instruction>,
}

pub const INTERNAL_CALL: u16 = 0x1000;

impl Function {
    pub fn argument_types(&self) -> Vec<Type> {
        let mut types = Vec::new();
        if let (true, Some(owner)) = (self.instance, &self.owner) {
            types.push(owner.clone());
        }
        types.extend(self.parameters.iter().cloned());
        types
    }

    pub fn is_internal_call(&self) -> bool {
        self.impl_flags == INTERNAL_CALL
    }
}

/// A call identifies an overload by name and ordered parameter types.
/// Return types remain on definitions and cannot distinguish overloads.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionRef {
    pub name: String,
    #[serde(default)]
    pub owner: Option<Type>,
    #[serde(default)]
    pub instance: bool,
    pub parameters: Vec<Type>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "arg", deny_unknown_fields)]
pub enum Instruction {
    #[serde(rename = "ldc.i4")]
    Int(i32),
    #[serde(rename = "ldc.i8")]
    Int64(i64),
    #[serde(rename = "conv.ovf.i1")]
    CheckedInt8,
    #[serde(rename = "conv.ovf.u1")]
    CheckedUInt8,
    #[serde(rename = "conv.ovf.i2")]
    CheckedInt16,
    #[serde(rename = "conv.ovf.u2")]
    CheckedUInt16,
    #[serde(rename = "conv.ovf.i4")]
    CheckedInt32,
    #[serde(rename = "conv.ovf.u4")]
    CheckedUInt32,
    #[serde(rename = "conv.ovf.i8")]
    CheckedInt64,
    #[serde(rename = "conv.ovf.u8")]
    CheckedUInt64,
    #[serde(rename = "conv.ovf.i")]
    CheckedNativeInt,
    #[serde(rename = "conv.ovf.u")]
    CheckedNativeUInt,
    #[serde(rename = "conv.ovf.i1.un")]
    CheckedInt8Unsigned,
    #[serde(rename = "conv.ovf.u1.un")]
    CheckedUInt8Unsigned,
    #[serde(rename = "conv.ovf.i2.un")]
    CheckedInt16Unsigned,
    #[serde(rename = "conv.ovf.u2.un")]
    CheckedUInt16Unsigned,
    #[serde(rename = "conv.ovf.i4.un")]
    CheckedInt32Unsigned,
    #[serde(rename = "conv.ovf.u4.un")]
    CheckedUInt32Unsigned,
    #[serde(rename = "conv.ovf.i8.un")]
    CheckedInt64Unsigned,
    #[serde(rename = "conv.ovf.u8.un")]
    CheckedUInt64Unsigned,
    #[serde(rename = "conv.ovf.i.un")]
    CheckedNativeIntUnsigned,
    #[serde(rename = "conv.ovf.u.un")]
    CheckedNativeUIntUnsigned,
    #[serde(rename = "conv.i1")]
    ConvertInt8,
    #[serde(rename = "conv.u1")]
    ConvertUInt8,
    #[serde(rename = "conv.i2")]
    ConvertInt16,
    #[serde(rename = "conv.u2")]
    ConvertUInt16,
    #[serde(rename = "conv.u4")]
    ConvertUInt32,
    #[serde(rename = "conv.i8")]
    ConvertInt64,
    #[serde(rename = "conv.u8")]
    ConvertUInt64,
    #[serde(rename = "ldind.i1")]
    LoadIndirectInt8,
    #[serde(rename = "ldind.u1")]
    LoadIndirectUInt8,
    #[serde(rename = "ldind.i2")]
    LoadIndirectInt16,
    #[serde(rename = "ldind.u2")]
    LoadIndirectUInt16,
    #[serde(rename = "ldind.u4")]
    LoadIndirectUInt32,
    #[serde(rename = "ldind.i8")]
    LoadIndirectInt64,
    #[serde(rename = "ldind.i")]
    LoadIndirectNative,
    #[serde(rename = "stind.i1")]
    StoreIndirectInt8,
    #[serde(rename = "stind.i2")]
    StoreIndirectInt16,
    #[serde(rename = "stind.i8")]
    StoreIndirectInt64,
    #[serde(rename = "stind.i")]
    StoreIndirectNative,
    #[serde(rename = "ldc.r4")]
    Float32 { bits: u32 },
    #[serde(rename = "ldc.r8")]
    Float64 { bits: u64 },
    #[serde(rename = "conv.r4")]
    ConvertFloat32,
    #[serde(rename = "conv.r8")]
    ConvertFloat64,
    #[serde(rename = "conv.r.un")]
    ConvertFloatUnsigned,
    #[serde(rename = "ckfinite")]
    CheckFinite,
    #[serde(rename = "ldind.r4")]
    LoadIndirectFloat32,
    #[serde(rename = "ldind.r8")]
    LoadIndirectFloat64,
    #[serde(rename = "stind.r4")]
    StoreIndirectFloat32,
    #[serde(rename = "stind.r8")]
    StoreIndirectFloat64,
    #[serde(rename = "ldc.bool")]
    Bool(bool),
    #[serde(rename = "ldstr")]
    String(String),
    #[serde(rename = "ldvoid")]
    Void,
    #[serde(rename = "ldarg")]
    Arg(usize),
    #[serde(rename = "ldloc")]
    Load(usize),
    #[serde(rename = "stloc")]
    Store(usize),
    #[serde(rename = "dup")]
    Dup,
    #[serde(rename = "pop")]
    Pop,
    #[serde(rename = "and")]
    BitAnd,
    #[serde(rename = "or")]
    BitOr,
    #[serde(rename = "xor")]
    BitXor,
    #[serde(rename = "not")]
    BitNot,
    #[serde(rename = "neg")]
    Negate,
    #[serde(rename = "shl")]
    ShiftLeft,
    #[serde(rename = "shr")]
    ShiftRight,
    #[serde(rename = "shr.un")]
    ShiftRightUnsigned,
    #[serde(rename = "rem")]
    Remainder,
    #[serde(rename = "rem.un")]
    RemainderUnsigned,
    #[serde(rename = "add")]
    Add,
    #[serde(rename = "sub")]
    Sub,
    #[serde(rename = "mul")]
    Mul,
    #[serde(rename = "add.ovf")]
    AddChecked,
    #[serde(rename = "sub.ovf")]
    SubChecked,
    #[serde(rename = "mul.ovf")]
    MulChecked,
    #[serde(rename = "div")]
    Divide,
    #[serde(rename = "add.ovf.un")]
    AddCheckedUnsigned,
    #[serde(rename = "sub.ovf.un")]
    SubCheckedUnsigned,
    #[serde(rename = "mul.ovf.un")]
    MulCheckedUnsigned,
    #[serde(rename = "div.un")]
    DivideUnsigned,
    #[serde(rename = "clt.un")]
    LessUnsigned,
    #[serde(rename = "conv.i")]
    ConvertNativeInt,
    #[serde(rename = "conv.u")]
    ConvertNativeUInt,
    #[serde(rename = "conv.i4")]
    ConvertInt32,
    #[serde(rename = "ptr.fromint")]
    PointerFromInt(Type),
    #[serde(rename = "ceq")]
    Equal,
    #[serde(rename = "cgt")]
    Greater,
    #[serde(rename = "cgt.un")]
    GreaterUnsigned,
    #[serde(rename = "clt")]
    Less,
    #[serde(rename = "br")]
    Branch(usize),
    #[serde(rename = "brtrue")]
    BranchTrue(usize),
    #[serde(rename = "call")]
    Call(FunctionRef),
    #[serde(rename = "ret")]
    Return,
    #[serde(rename = "newobj")]
    New(String),
    #[serde(rename = "ldfld")]
    Field(usize),
    #[serde(rename = "stfld")]
    SetField(usize),
    #[serde(rename = "sizeof")]
    SizeOf(Type),
    #[serde(rename = "alignof")]
    AlignOf(Type),
    #[serde(rename = "heap.alloc")]
    Allocate(Type),
    #[serde(rename = "heap.free")]
    Free,
    #[serde(rename = "ptr.null")]
    NullPointer(Type),
    #[serde(rename = "ptr.cast")]
    PointerCast(Type),
    #[serde(rename = "ptr.add")]
    PointerAdd,
    #[serde(rename = "ldflda")]
    FieldAddress(usize),
    #[serde(rename = "ldobj")]
    LoadObject(Type),
    #[serde(rename = "stobj")]
    StoreObject(Type),
    #[serde(rename = "ldind.i4")]
    LoadIndirectInt32,
    #[serde(rename = "stind.i4")]
    StoreIndirectInt32,
    #[serde(rename = "heap.new")]
    HeapNew,
    #[serde(rename = "heap.load")]
    HeapLoad,
    #[serde(rename = "heap.store")]
    HeapStore,
    #[serde(rename = "some")]
    Some,
    #[serde(rename = "none")]
    None(Type),
    #[serde(rename = "ok")]
    Ok(Type),
    #[serde(rename = "err")]
    Err(Type),
    #[serde(rename = "is.case")]
    IsCase(Case),
    #[serde(rename = "ldcase")]
    LoadCase(Case),
    #[serde(rename = "error")]
    Error(String),
    #[serde(rename = "fault")]
    Fault(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Case {
    Some,
    None,
    Ok,
    Err,
}

impl Module {
    pub fn type_definition(&self, ty: &Type) -> Option<&TypeDef> {
        let name = ty.definition_name()?;
        self.types.iter().find(|def| def.name == name)
    }
}

pub(crate) fn valid_slot_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub(crate) fn validate_slot_names(
    names: &[Option<String>],
    count: usize,
    reserve_this: bool,
) -> Result<(), crate::Fault> {
    if !names.is_empty() && names.len() != count {
        return Err(crate::Fault::new(
            "slot names must align with their type signatures",
        ));
    }
    let mut seen = std::collections::HashSet::new();
    for name in names.iter().flatten() {
        if !valid_slot_name(name) || (reserve_this && name == "this") || !seen.insert(name) {
            return Err(crate::Fault::new(format!(
                "invalid, duplicate or reserved slot name {name:?}"
            )));
        }
    }
    Ok(())
}

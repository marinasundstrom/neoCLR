use serde::{Deserialize, Serialize};

/// No value-type/reference-type bit: ByRef selects explicit managed reference access.
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
    /// One explicitly erased, complete value; no reference identity or allocation policy.
    Value,
    /// Opaque read-only metadata descriptor, with no native address or payload storage.
    RuntimeTypeHandle,
    Named(String),
    /// An explicit source-module scope, checked and bound during preparation.
    Scoped {
        module: String,
        name: String,
        arguments: Vec<Type>,
    },
    /// Indexed parameter of the declaring type (CLI VAR-like signature).
    TypeParameter(u16),
    /// Indexed method parameter (CLI MVAR), independent of its owner.
    MethodTypeParameter(u16),
    Constructed {
        definition: String,
        arguments: Vec<Type>,
    },
    /// Retaining managed slot reference; independent of native layout.
    ByRef(Box<Type>),
    /// Managed reference with a declared readonly access contract.
    ReadOnlyByRef(Box<Type>),
    /// Owned fixed-length array value; allocation mode is separate.
    Array(Box<Type>),
    /// Explicit borrowed interface receiver, separate from the interface declaration.
    InterfaceRef(Box<Type>),
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
            "Value" | "System.Value" => Self::Value,
            "RuntimeTypeHandle" | "System.RuntimeTypeHandle" => Self::RuntimeTypeHandle,
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
            Self::Value => Some("System.Value"),
            Self::RuntimeTypeHandle => Some("System.RuntimeTypeHandle"),
            Self::Named(name)
            | Self::Constructed {
                definition: name, ..
            } => Some(name),
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
                | Self::Value
                | Self::RuntimeTypeHandle
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Module {
    pub format: u32,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    /// None retains legacy load-set visibility; Some lists direct references.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub references: Option<Vec<ModuleReference>>,
    #[serde(default)]
    pub entry: String,
    #[serde(default)]
    pub types: Vec<TypeDef>,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeDef {
    #[serde(default, skip_serializing_if = "Visibility::is_public")]
    pub visibility: Visibility,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definition: Option<TypeDefId>,
    /// Explicit lexical owner; dotted names alone do not imply nesting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declaring_type: Option<TypeDefId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_attributes: Vec<CustomAttribute>,
    pub name: String,
    pub fields: Vec<Field>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub implements: Vec<Type>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<Type>,
    #[serde(default)]
    pub is_abstract: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<Property>,
    #[serde(default)]
    pub representation: Representation,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub generic_parameters: Vec<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packing: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum_size: Option<u32>,
}

impl TypeDef {
    pub(crate) fn open_type(&self) -> Type {
        if self.generic_parameters.is_empty() {
            Type::from_name(&self.name)
        } else {
            Type::Constructed {
                definition: self.name.clone(),
                arguments: (0..self.generic_parameters.len())
                    .map(|i| Type::TypeParameter(i as u16))
                    .collect(),
            }
        }
    }
}

/// An ordinary property signature and explicit method-semantics associations.
/// Accessors execute through calls; this metadata adds no storage or dispatch.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Property {
    pub name: String,
    pub instance: bool,
    pub parameters: Vec<Type>,
    pub ty: Type,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub getter: Option<FunctionRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setter: Option<FunctionRef>,
}

impl Property {
    pub(crate) fn map_types(
        &mut self,
        mut map: impl FnMut(&Type) -> Result<Type, crate::Fault>,
    ) -> Result<(), crate::Fault> {
        self.ty = map(&self.ty)?;
        for ty in &mut self.parameters {
            *ty = map(ty)?;
        }
        for target in self.getter.iter_mut().chain(self.setter.iter_mut()) {
            if let Some(owner) = &mut target.owner {
                *owner = map(owner)?;
            }
            for ty in target
                .parameters
                .iter_mut()
                .chain(&mut target.generic_arguments)
            {
                *ty = map(ty)?;
            }
        }
        Ok(())
    }
}

/// Representation is independent of ownership and reference identity.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Representation {
    #[default]
    Record,
    Runtime,
    Interface,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    #[serde(default, skip_serializing_if = "Visibility::is_public")]
    pub visibility: Visibility,
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Function {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sequence_points: Vec<SequencePoint>,
    #[serde(default, skip_serializing_if = "Visibility::is_public")]
    pub visibility: Visibility,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definition: Option<MemberId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_attributes: Vec<CustomAttribute>,
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
    /// Instance argument zero addresses the caller slot rather than containing a value copy.
    #[serde(default)]
    pub receiver_byref: bool,
    /// Narrow the managed instance receiver to readonly access on entry.
    #[serde(default)]
    pub receiver_readonly: bool,
    #[serde(default)]
    pub is_virtual: bool,
    #[serde(default)]
    pub is_override: bool,
    #[serde(default)]
    pub is_abstract: bool,
    /// Explicit interface declarations implemented by this body (MethodImpl analogue).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interface_implementations: Vec<FunctionRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub generic_parameters: Vec<Option<String>>,
    /// Instantiation carried by resolved bodies, never serialized on a definition.
    #[serde(skip)]
    pub generic_arguments: Vec<Type>,
    /// Declared parameter indices whose slots must be assigned before normal return.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub out_parameters: Vec<usize>,
    /// Output slots assigned on Boolean true; false provides no initialization guarantee.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub out_when_true: Vec<usize>,
    /// Input parameters narrowed to readonly managed-reference capabilities on entry.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub readonly_parameters: Vec<usize>,
    pub returns: Type,
    #[serde(default)]
    pub locals: Vec<Type>,
    #[serde(default)]
    pub local_names: Vec<Option<String>>,
    /// CLR MethodImplAttributes values: IL = 0, InternalCall = 0x1000.
    #[serde(default)]
    pub impl_flags: u16,
    #[serde(default)]
    pub pinvoke: Option<NativeImport>,
    #[serde(default)]
    pub body: Vec<Instruction>,
}

/// A one-based source location for an IL instruction boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SequencePoint {
    pub instruction: usize,
    pub document: String,
    pub line: usize,
    pub column: usize,
}

/// Initial member access levels, independent of type representation and allocation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    #[default]
    Public,
    Internal,
    Private,
}

impl Visibility {
    fn is_public(&self) -> bool {
        *self == Self::Public
    }
}

/// Prototype equivalent of an ImplMap entry; does not imply managed ownership.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeImport {
    pub library: String,
    pub entry_point: String,
    pub calling_convention: CallingConvention,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CallingConvention {
    Cdecl,
}

pub const INTERNAL_CALL: u16 = 0x1000;

impl Function {
    pub fn argument_types(&self) -> Vec<Type> {
        let mut types = Vec::new();
        if let (true, Some(owner)) = (self.instance, &self.owner) {
            types.push(if self.receiver_byref {
                Type::ByRef(Box::new(owner.clone()))
            } else {
                owner.clone()
            });
        }
        types.extend(self.parameters.iter().enumerate().map(|(index, ty)| {
            if self.readonly_parameters.contains(&index) {
                if let Type::ByRef(target) = ty {
                    return Type::ReadOnlyByRef(target.clone());
                }
            }
            ty.clone()
        }));
        if self.receiver_readonly {
            if let Some(Type::ByRef(target)) = types.first() {
                types[0] = Type::ReadOnlyByRef(target.clone());
            }
        }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definition: Option<MemberId>,
    pub name: String,
    #[serde(default)]
    pub owner: Option<Type>,
    #[serde(default)]
    pub instance: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub generic_arguments: Vec<Type>,
    pub parameters: Vec<Type>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "arg", deny_unknown_fields)]
pub enum Instruction {
    #[serde(rename = "unaligned.")]
    Unaligned(u8),
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
    #[serde(rename = "ldloca")]
    LocalAddress(usize),
    #[serde(rename = "ldarga")]
    ArgumentAddress(usize),
    #[serde(rename = "ldarg")]
    Arg(usize),
    #[serde(rename = "starg")]
    StoreArg(usize),
    #[serde(rename = "ldloc")]
    Load(usize),
    #[serde(rename = "stloc")]
    Store(usize),
    #[serde(rename = "local.reset")]
    ResetLocal(usize),
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
    #[serde(rename = "brfalse")]
    BranchFalse(usize),
    #[serde(rename = "beq")]
    BranchEqual(usize),
    #[serde(rename = "bne.un")]
    BranchNotEqual(usize),
    #[serde(rename = "bgt")]
    BranchGreater(usize),
    #[serde(rename = "bgt.un")]
    BranchGreaterUnsigned(usize),
    #[serde(rename = "blt")]
    BranchLess(usize),
    #[serde(rename = "blt.un")]
    BranchLessUnsigned(usize),
    #[serde(rename = "bge")]
    BranchGreaterEqual(usize),
    #[serde(rename = "bge.un")]
    BranchGreaterEqualUnsigned(usize),
    #[serde(rename = "ble")]
    BranchLessEqual(usize),
    #[serde(rename = "ble.un")]
    BranchLessEqualUnsigned(usize),
    #[serde(rename = "switch")]
    Switch(Vec<usize>),
    #[serde(rename = "call")]
    Call(FunctionRef),
    #[serde(rename = "newobj.ctor")]
    Construct(FunctionRef),
    #[serde(rename = "value.pack")]
    PackValue(Type),
    /// Fixed-length managed heap array with supported default initialization.
    #[serde(rename = "array.alloc")]
    AllocateArray(Type),
    #[serde(rename = "newarr")]
    NewArray(Type),
    #[serde(rename = "array.create")]
    CreateArray(Type),
    #[serde(rename = "ldlen")]
    ArrayLength,
    #[serde(rename = "ldelem")]
    ArrayElement(Type),
    #[serde(rename = "stelem")]
    StoreArrayElement(Type),
    #[serde(rename = "ldelema")]
    ArrayAddress(Type),
    /// CLI-shaped type-token acquisition; method and field tokens are not supported.
    #[serde(rename = "ldtoken")]
    LoadTypeToken(Type),
    #[serde(rename = "ref.type")]
    ReferenceType,
    #[serde(rename = "castclass")]
    CastClass(Type),
    #[serde(rename = "ref.eq")]
    ReferenceEqual,
    #[serde(rename = "interface.borrow")]
    BorrowInterface(Type),
    #[serde(rename = "callvirt")]
    CallVirtual(FunctionRef),
    #[serde(rename = "value.is")]
    IsValue(Type),
    #[serde(rename = "value.unpack")]
    UnpackValue(Type),
    #[serde(rename = "ret")]
    Return,
    #[serde(rename = "newobj")]
    New(#[serde(with = "construction_type")] Type),
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
    #[serde(rename = "localloc")]
    AllocateLocal,
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
    #[serde(rename = "cpobj")]
    CopyObject(Type),
    #[serde(rename = "initobj")]
    InitializeObject(Type),
    #[serde(rename = "cpblk")]
    CopyBlock,
    #[serde(rename = "initblk")]
    InitializeBlock,
    #[serde(rename = "ldind.i4")]
    LoadIndirectInt32,
    #[serde(rename = "stind.i4")]
    StoreIndirectInt32,
    #[serde(rename = "heap.new")]
    HeapNew,
    #[serde(rename = "error")]
    Error(String),
    #[serde(rename = "fault")]
    Fault(String),
}

impl Module {
    pub fn type_definition(&self, ty: &Type) -> Option<&TypeDef> {
        let name = ty.definition_name()?;
        let arity = match ty {
            Type::Constructed { arguments, .. } => arguments.len(),
            _ => 0,
        };
        self.types
            .iter()
            .find(|def| def.name == name && def.generic_parameters.len() == arity)
    }

    /// Enumerate ordinary nested definitions owned by a type definition.
    /// Ownership is resolved by definition identity, never by name prefixes.
    pub fn nested_type_definitions(&self, owner: &TypeDefId) -> Vec<&TypeDef> {
        self.types
            .iter()
            .filter(|definition| definition.declaring_type.as_ref() == Some(owner))
            .collect()
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

impl Instruction {
    pub(crate) fn accepts_unaligned(&self) -> bool {
        matches!(
            self,
            Self::LoadObject(_)
                | Self::StoreObject(_)
                | Self::LoadIndirectInt8
                | Self::LoadIndirectUInt8
                | Self::LoadIndirectInt16
                | Self::LoadIndirectUInt16
                | Self::LoadIndirectInt32
                | Self::LoadIndirectUInt32
                | Self::LoadIndirectInt64
                | Self::LoadIndirectNative
                | Self::LoadIndirectFloat32
                | Self::LoadIndirectFloat64
                | Self::StoreIndirectInt8
                | Self::StoreIndirectInt16
                | Self::StoreIndirectInt32
                | Self::StoreIndirectInt64
                | Self::StoreIndirectNative
                | Self::StoreIndirectFloat32
                | Self::StoreIndirectFloat64
                | Self::CopyBlock
                | Self::InitializeBlock
        )
    }
}

impl Type {
    pub fn substitute_type_parameters(&self, arguments: &[Type]) -> Result<Type, crate::Fault> {
        self.substitute_parameters(Some(arguments), None)
    }
    pub fn substitute_method_parameters(&self, arguments: &[Type]) -> Result<Type, crate::Fault> {
        self.substitute_parameters(None, Some(arguments))
    }
    pub(crate) fn substitute_parameters(
        &self,
        types: Option<&[Type]>,
        methods: Option<&[Type]>,
    ) -> Result<Type, crate::Fault> {
        fn substitute(
            ty: &Type,
            types: Option<&[Type]>,
            methods: Option<&[Type]>,
            depth: usize,
        ) -> Result<Type, crate::Fault> {
            if depth > 32 {
                return Err(crate::Fault::new("type substitution nesting exceeds 32"));
            }
            let nested = |ty: &Type| substitute(ty, types, methods, depth + 1);
            Ok(match ty {
                Type::TypeParameter(index) if types.is_some() => types
                    .unwrap()
                    .get(*index as usize)
                    .cloned()
                    .ok_or_else(|| crate::Fault::new("type parameter index outside arguments"))?,
                Type::MethodTypeParameter(index) if methods.is_some() => methods
                    .unwrap()
                    .get(*index as usize)
                    .cloned()
                    .ok_or_else(|| crate::Fault::new("method parameter index outside arguments"))?,
                Type::Constructed {
                    definition,
                    arguments: types,
                } => Type::Constructed {
                    definition: definition.clone(),
                    arguments: types.iter().map(nested).collect::<Result<_, _>>()?,
                },
                Type::Scoped {
                    module,
                    name,
                    arguments: types,
                } => Type::Scoped {
                    module: module.clone(),
                    name: name.clone(),
                    arguments: types.iter().map(nested).collect::<Result<_, _>>()?,
                },
                Type::ByRef(t) => Type::ByRef(Box::new(nested(t)?)),
                Type::ReadOnlyByRef(t) => Type::ReadOnlyByRef(Box::new(nested(t)?)),
                Type::Array(t) => Type::Array(Box::new(nested(t)?)),
                Type::Ptr(t) => Type::Ptr(Box::new(nested(t)?)),
                Type::InterfaceRef(t) => Type::InterfaceRef(Box::new(nested(t)?)),
                other => other.clone(),
            })
        }
        substitute(self, types, methods, 0)
    }
}

impl Module {
    /// Resolve closed record field signatures without allocating a runtime value.
    pub fn instantiated_fields(&self, ty: &Type) -> Result<Vec<Field>, crate::Fault> {
        crate::vm::record_fields(self, ty, 0)
    }
}

// Preserve the prototype's non-generic newobj string operand on disk, while
// constructed operands use the same structural signatures as other instructions.
mod construction_type {
    use super::Type;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(ty: &Type, serializer: S) -> Result<S::Ok, S::Error> {
        match ty {
            Type::Named(name) => name.serialize(serializer),
            _ => ty.serialize(serializer),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Type, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Operand {
            Signature(Type),
            LegacyName(String),
        }
        Ok(match Operand::deserialize(deserializer)? {
            Operand::Signature(ty) => ty,
            Operand::LegacyName(name) => Type::from_name(&name),
        })
    }
}

impl Function {
    /// Transform every signature in a method, including instruction operands.
    pub(crate) fn map_types(
        &self,
        mut map: impl FnMut(&Type) -> Result<Type, crate::Fault>,
    ) -> Result<Self, crate::Fault> {
        let mut result = self.clone();
        if let Some(owner) = &mut result.owner {
            *owner = map(owner)?;
        }
        for ty in result
            .parameters
            .iter_mut()
            .chain(&mut result.locals)
            .chain([&mut result.returns])
        {
            *ty = map(ty)?;
        }
        for target in &mut result.interface_implementations {
            if let Some(owner) = &mut target.owner {
                *owner = map(owner)?;
            }
            for ty in target
                .parameters
                .iter_mut()
                .chain(&mut target.generic_arguments)
            {
                *ty = map(ty)?;
            }
        }
        for op in &mut result.body {
            match op {
                Instruction::Call(target)
                | Instruction::CallVirtual(target)
                | Instruction::Construct(target) => {
                    if let Some(owner) = &mut target.owner {
                        *owner = map(owner)?;
                    }
                    for ty in target
                        .parameters
                        .iter_mut()
                        .chain(&mut target.generic_arguments)
                    {
                        *ty = map(ty)?;
                    }
                }
                Instruction::New(ty)
                | Instruction::AllocateArray(ty)
                | Instruction::NewArray(ty)
                | Instruction::CreateArray(ty)
                | Instruction::ArrayElement(ty)
                | Instruction::StoreArrayElement(ty)
                | Instruction::ArrayAddress(ty)
                | Instruction::CastClass(ty)
                | Instruction::BorrowInterface(ty)
                | Instruction::LoadTypeToken(ty)
                | Instruction::PackValue(ty)
                | Instruction::IsValue(ty)
                | Instruction::UnpackValue(ty)
                | Instruction::SizeOf(ty)
                | Instruction::AlignOf(ty)
                | Instruction::Allocate(ty)
                | Instruction::LoadObject(ty)
                | Instruction::StoreObject(ty)
                | Instruction::CopyObject(ty)
                | Instruction::InitializeObject(ty)
                | Instruction::NullPointer(ty)
                | Instruction::PointerCast(ty)
                | Instruction::PointerFromInt(ty) => *ty = map(ty)?,
                _ => (),
            }
        }
        Ok(result)
    }
}

/// Marker-only subset of CLI custom attributes. The constructor is metadata,
/// not an instruction to execute when loading the annotated definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomAttribute {
    pub constructor: FunctionRef,
}

/// Module-local function-definition row, not a durable identifier across rebuilds.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemberId {
    pub module: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    pub index: u32,
}

/// Module-local type-definition row; separate from the function-definition table.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeDefId {
    pub module: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    pub index: u32,
}

impl Module {
    pub(crate) fn normalize_definition_ids(&mut self) -> Result<(), crate::Fault> {
        if self
            .revision
            .as_ref()
            .is_some_and(|revision| !valid_revision(revision))
        {
            return Err(crate::Fault::new("invalid module revision"));
        }
        for (index, definition) in self.types.iter_mut().enumerate() {
            let identity = TypeDefId {
                module: self.name.clone(),
                revision: self.revision.clone(),
                index: u32::try_from(index)
                    .map_err(|_| crate::Fault::new("too many type definitions"))?,
            };
            if definition
                .definition
                .as_ref()
                .is_some_and(|existing| existing != &identity)
            {
                return Err(crate::Fault::new("noncanonical type definition identity"));
            }
            definition.definition = Some(identity);
        }
        for (index, function) in self.functions.iter_mut().enumerate() {
            let identity = MemberId {
                module: self.name.clone(),
                revision: self.revision.clone(),
                index: u32::try_from(index)
                    .map_err(|_| crate::Fault::new("too many function definitions"))?,
            };
            if function
                .definition
                .as_ref()
                .is_some_and(|existing| existing != &identity)
            {
                return Err(crate::Fault::new(
                    "noncanonical function definition identity",
                ));
            }
            function.definition = Some(identity);
        }
        Ok(())
    }
}

/// A name-only dependency or an exact revision requirement (no version ranges).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum ModuleReference {
    Name(String),
    Exact { name: String, revision: String },
}

impl ModuleReference {
    pub fn name(&self) -> &str {
        match self {
            Self::Name(name) | Self::Exact { name, .. } => name,
        }
    }
    pub fn revision(&self) -> Option<&str> {
        match self {
            Self::Name(_) => None,
            Self::Exact { revision, .. } => Some(revision),
        }
    }
}

impl From<String> for ModuleReference {
    fn from(name: String) -> Self {
        Self::Name(name)
    }
}
impl From<&str> for ModuleReference {
    fn from(name: &str) -> Self {
        Self::Name(name.into())
    }
}

pub(crate) fn valid_revision(revision: &str) -> bool {
    !revision.is_empty()
        && revision
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

use serde::{Deserialize, Serialize};

/// No value-type/reference-type bit: Ref is an explicit storage capability.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    Void,
    Int32,
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
            "Int32" | "int32" | "int" | "System.Int32" => Self::Int32,
            "Boolean" | "boolean" | "bool" | "System.Boolean" => Self::Boolean,
            "String" | "string" | "System.String" => Self::String,
            "Error" | "System.Error" => Self::Error,
            _ => Self::Named(name.into()),
        }
    }

    pub fn definition_name(&self) -> Option<&str> {
        match self {
            Self::Void => Some("System.Void"),
            Self::Int32 => Some("System.Int32"),
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
            Self::Void | Self::Int32 | Self::Boolean | Self::String | Self::Error
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
    pub returns: Type,
    #[serde(default)]
    pub locals: Vec<Type>,
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
    #[serde(rename = "ceq")]
    Equal,
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

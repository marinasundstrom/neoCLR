//! Queries over the loaded descriptive assembly catalog; never loads external files.
use crate::metadata::{Type, TypeDef};
use crate::metadata_origin::AssemblyMetadata;
use crate::{Fault, Limits, Module, Value};

#[derive(Clone, Copy)]
pub(crate) enum Query {
    Name,
    Token,
    References,
    Modules,
    Types,
    ModuleAssembly,
    ModuleToken,
    ModuleTypes,
}
impl Query {
    pub(crate) fn binding(name: &str) -> Option<(Self, usize, &'static str)> {
        Some(match name {
            "neoCLR.Runtime.AssemblyName" => (Self::Name, 1, "String"),
            "neoCLR.Runtime.AssemblyMetadataToken" => (Self::Token, 1, "Int32"),
            "neoCLR.Runtime.AssemblyReferences" => {
                (Self::References, 1, "System.Introspection.AssemblyInfo[]")
            }
            "neoCLR.Runtime.AssemblyModules" => {
                (Self::Modules, 1, "System.Introspection.ModuleInfo[]")
            }
            "neoCLR.Runtime.AssemblyTypes" => (Self::Types, 1, "System.Introspection.TypeInfo[]"),
            "neoCLR.Runtime.ModuleAssembly" => {
                (Self::ModuleAssembly, 2, "System.Introspection.AssemblyInfo")
            }
            "neoCLR.Runtime.ModuleMetadataToken" => (Self::ModuleToken, 2, "Int32"),
            "neoCLR.Runtime.ModuleTypes" => {
                (Self::ModuleTypes, 2, "System.Introspection.TypeInfo[]")
            }
            _ => return None,
        })
    }
    pub(crate) fn invoke(
        self,
        module: &Module,
        args: &[Value],
        limits: &Limits,
    ) -> Result<Value, Fault> {
        let Some(Value::String(identity)) = args.first() else {
            return Err(Fault::new("assembly identity requires String"));
        };
        let assembly = lookup(module, identity)?;
        let selected_module = if let Some(Value::String(name)) = args.get(1) {
            if !assembly.modules.contains(name) {
                return Err(Fault::new("module does not belong to assembly"));
            }
            Some(name.as_str())
        } else {
            None
        };
        match self {
            Self::Name => Ok(Value::String(assembly.name.clone())),
            Self::Token => Ok(Value::Int32(0x20000001)),
            Self::ModuleToken => Ok(Value::Int32(1)),
            Self::ModuleAssembly => assembly_value(module, identity),
            Self::References => crate::reflection::array(
                "System.Introspection.AssemblyInfo",
                assembly
                    .references
                    .iter()
                    .map(|id| assembly_value(module, id)),
                limits,
            ),
            Self::Modules => crate::reflection::array(
                "System.Introspection.ModuleInfo",
                assembly
                    .modules
                    .iter()
                    .map(|name| Ok(module_value(identity, name))),
                limits,
            ),
            Self::Types | Self::ModuleTypes => crate::reflection::array(
                "System.Introspection.TypeInfo",
                module
                    .types
                    .iter()
                    .filter(|d| belongs(d, assembly, selected_module))
                    .map(|d| {
                        crate::type_identity::describe_definition(module, d)
                            .map(|descriptor| crate::reflection::wrap_type(module, descriptor))
                    }),
                limits,
            ),
        }
    }
}
pub(crate) fn lookup<'a>(
    module: &'a Module,
    identity: &str,
) -> Result<&'a AssemblyMetadata, Fault> {
    module
        .assemblies
        .iter()
        .find(|a| a.full_name == identity)
        .ok_or_else(|| Fault::new(format!("assembly metadata is not loaded: {identity}")))
}
pub(crate) fn assembly_value(module: &Module, identity: &str) -> Result<Value, Fault> {
    lookup(module, identity)?;
    Ok(Value::Object {
        ty: Type::from_name("System.Introspection.AssemblyInfo"),
        fields: vec![Value::String(identity.into())],
    })
}
pub(crate) fn module_value(identity: &str, name: &str) -> Value {
    Value::Object {
        ty: Type::from_name("System.Introspection.ModuleInfo"),
        fields: vec![Value::String(identity.into()), Value::String(name.into())],
    }
}
fn belongs(
    definition: &TypeDef,
    assembly: &AssemblyMetadata,
    selected_module: Option<&str>,
) -> bool {
    if let Some(origin) = &definition.origin {
        origin.assembly == assembly.full_name && selected_module.is_none_or(|m| m == origin.module)
    } else {
        definition.definition.as_ref().is_some_and(|id| {
            assembly.modules.contains(&id.module) && selected_module.is_none_or(|m| m == id.module)
        })
    }
}

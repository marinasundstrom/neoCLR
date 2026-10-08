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
            if !assembly
                .modules
                .iter()
                .any(|module| module == name.as_str())
            {
                return Err(Fault::new("module does not belong to assembly"));
            }
            Some(name.as_str())
        } else {
            None
        };
        match self {
            Self::Name => Ok(Value::String(assembly.name.clone().into())),
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
// Source-built runtime facades have real origins, unlike the legacy System adapter.
// Skip only the known contiguous query wrappers in the same source module; ordinary
// runtime-library callers and similarly named methods in other assemblies still count.
pub(crate) fn executing_assembly<'a>(
    module: &Module,
    frames: impl Iterator<Item = &'a crate::metadata::Function>,
) -> Option<String> {
    let mut facade_scope: Option<(&str, &str)> = None;
    for function in frames {
        if let Some(origin) = &function.origin {
            let owner = function
                .owner
                .as_ref()
                .and_then(|ty| module.type_definition(ty))
                .and_then(|ty| ty.origin.as_ref());
            let facade = owner.is_some_and(|owner| {
                owner.assembly == origin.assembly
                    && owner.module == origin.module
                    && function.parameters.is_empty()
                    && matches!(
                        (owner.name.as_str(), origin.name.as_str()),
                        (
                            "System.Runtime.CompilerServices.IntrospectionRuntimeServices",
                            "ExecutingAssembly"
                        ) | ("System.Runtime.RuntimeContext", "get_ExecutingAssembly")
                    )
            });
            let scope = (origin.assembly.as_str(), origin.module.as_str());
            if facade && facade_scope.is_none_or(|expected| expected == scope) {
                facade_scope = Some(scope);
                continue;
            }
            return Some(origin.assembly.clone());
        }
        let Some(id) = &function.definition else {
            continue;
        };
        if id.module == "System" {
            continue; // Legacy library facade and generated adapters lack source origins.
        }
        if let Some(assembly) = module
            .assemblies
            .iter()
            .find(|a| a.modules.contains(&id.module))
        {
            return Some(assembly.full_name.clone());
        }
    }
    None
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::Function;

    fn origin(assembly: &str, name: &str) -> crate::metadata_origin::MetadataOrigin {
        serde_json::from_value(serde_json::json!({
            "assembly": assembly, "module": "Shared.dll", "name": name, "token": 1
        }))
        .unwrap()
    }

    #[test]
    fn executing_assembly_skips_scoped_source_facades_not_real_library_callers() {
        let mut module = crate::assemble(".module App\n.type Service\n.end\n.type Context\n.end\n.type ForeignContext\n.end\n.function Stub() -> Int32\nldc.i4 0\nret\n.end").unwrap();
        module.types[0].origin = Some(origin(
            "Runtime",
            "System.Runtime.CompilerServices.IntrospectionRuntimeServices",
        ));
        module.types[1].origin = Some(origin("Runtime", "System.Runtime.RuntimeContext"));
        module.types[2].origin = Some(origin("Other", "System.Runtime.RuntimeContext"));
        let make = |owner: Option<&str>, assembly: &str, name: &str| {
            let mut f: Function = module.functions[0].clone();
            f.owner = owner.map(Type::from_name);
            f.origin = Some(origin(assembly, name));
            f
        };
        let service = make(Some("Service"), "Runtime", "ExecutingAssembly");
        let getter = make(Some("Context"), "Runtime", "get_ExecutingAssembly");
        let app = make(None, "Application", "Main");
        let dependency = make(None, "Dependency", "ReadContext");
        let runtime_caller = make(None, "Runtime", "OrdinaryLibraryFunction");
        for (caller, expected) in [
            (&app, "Application"),
            (&dependency, "Dependency"),
            (&runtime_caller, "Runtime"),
        ] {
            assert_eq!(
                executing_assembly(&module, [&service, &getter, caller, &app].into_iter())
                    .as_deref(),
                Some(expected)
            );
        }
        // A similarly named caller in another assembly cannot be swallowed by the facade scan.
        let foreign = make(Some("ForeignContext"), "Other", "get_ExecutingAssembly");
        assert_eq!(
            executing_assembly(&module, [&service, &foreign, &app].into_iter()).as_deref(),
            Some("Other")
        );
        assert_eq!(
            executing_assembly(&module, [&app].into_iter()).as_deref(),
            Some("Application")
        );
        assert_eq!(
            executing_assembly(&module, [&service, &getter].into_iter()),
            None
        );
    }
}

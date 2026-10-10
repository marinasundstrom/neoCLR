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
            if !module_names(module, assembly)?
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
                module_names(module, assembly)?
                    .iter()
                    .map(|name| Ok(module_value(identity, name))),
                limits,
            ),
            Self::Types | Self::ModuleTypes => {
                let mut types = Vec::new();
                for definition in &module.types {
                    if !belongs(definition, assembly) {
                        continue;
                    }
                    if let Some(name) = selected_module {
                        if type_module_name(module, definition)? != name {
                            continue;
                        }
                    }
                    types.push(
                        crate::type_identity::describe_definition(module, definition)
                            .map(|descriptor| crate::reflection::wrap_type(module, descriptor))?,
                    );
                }
                crate::reflection::array(
                    "System.Introspection.TypeInfo",
                    types.into_iter().map(Ok),
                    limits,
                )
            }
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
fn belongs(definition: &TypeDef, assembly: &AssemblyMetadata) -> bool {
    if let Some(origin) = &definition.origin {
        origin.assembly == assembly.full_name
    } else {
        definition
            .definition
            .as_ref()
            .is_some_and(|id| assembly.modules.contains(&id.module))
    }
}

/// Flat logical names; legacy images project retained declarations without inventing parents.
pub(crate) fn module_names(
    module: &Module,
    assembly: &AssemblyMetadata,
) -> Result<Vec<String>, Fault> {
    if let Some(table) = &assembly.declaration_modules {
        let mut names = table.names.clone();
        names.sort_by(|left, right| left.encode_utf16().cmp(right.encode_utf16()));
        return Ok(names);
    }
    let mut names = std::collections::BTreeSet::new();
    for definition in &module.types {
        if belongs(definition, assembly) {
            names.insert(type_module_name(module, definition)?);
        }
    }
    for function in &module.functions {
        if function.owner.is_none()
            && function.origin.as_ref().map_or_else(
                || {
                    function
                        .definition
                        .as_ref()
                        .is_some_and(|id| assembly.modules.contains(&id.module))
                },
                |origin| origin.assembly == assembly.full_name,
            )
        {
            names.insert(function.namespace.clone());
        }
    }
    names.extend(
        assembly
            .constants
            .iter()
            .map(|constant| constant.namespace.clone()),
    );
    let mut names: Vec<_> = names.into_iter().collect();
    // Match the host metadata reader's StringComparer.Ordinal ordering.
    names.sort_by(|left, right| left.encode_utf16().cmp(right.encode_utf16()));
    Ok(names)
}

pub(crate) fn type_module_name(module: &Module, definition: &TypeDef) -> Result<String, Fault> {
    let mut root = definition;
    // Metadata admission rejects cycles; retain a bound for descriptor queries on unverified graphs.
    for _ in 0..=module.types.len() {
        let Some(parent) = &root.declaring_type else {
            let name = root
                .origin
                .as_ref()
                .map_or(root.name.as_str(), |origin| origin.name.as_str());
            return Ok(name
                .rsplit_once('.')
                .map_or("", |(namespace, _)| namespace)
                .to_owned());
        };
        root = module
            .types
            .iter()
            .find(|ty| ty.definition.as_ref() == Some(parent))
            .ok_or_else(|| Fault::new("missing logical module type owner"))?;
    }
    Err(Fault::new("cyclic logical module type owner"))
}

pub(crate) fn type_module_value(module: &Module, definition: &TypeDef) -> Result<Value, Fault> {
    let name = type_module_name(module, definition)?;
    declaration_module_value(
        module,
        definition.origin.as_ref(),
        definition.definition.as_ref().map(|id| id.module.as_str()),
        &name,
    )
}

pub(crate) fn function_module_value(
    module: &Module,
    function: &crate::metadata::Function,
) -> Result<Value, Fault> {
    if let Some(owner) = &function.owner {
        let definition = module
            .type_definition(owner)
            .ok_or_else(|| Fault::new("missing function module owner"))?;
        return type_module_value(module, definition);
    }
    declaration_module_value(
        module,
        function.origin.as_ref(),
        function.definition.as_ref().map(|id| id.module.as_str()),
        &function.namespace,
    )
}

fn declaration_module_value(
    module: &Module,
    origin: Option<&crate::metadata_origin::MetadataOrigin>,
    physical: Option<&str>,
    name: &str,
) -> Result<Value, Fault> {
    let assembly = if let Some(origin) = origin {
        lookup(module, &origin.assembly)?
    } else {
        let physical = physical.ok_or_else(|| Fault::new("missing module identity"))?;
        let mut matches = module
            .assemblies
            .iter()
            .filter(|assembly| assembly.modules.iter().any(|scope| scope == physical));
        let assembly = matches
            .next()
            .ok_or_else(|| Fault::new("missing module catalog entry"))?;
        if matches.next().is_some() {
            return Err(Fault::new("ambiguous physical module scope"));
        }
        assembly
    };
    if !module_names(module, assembly)?
        .iter()
        .any(|module| module == name)
    {
        return Err(Fault::new("logical module does not belong to assembly"));
    }
    Ok(module_value(&assembly.full_name, name))
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
    fn logical_modules_are_flat_scoped_and_preserve_empty_declarations() {
        let mut module = crate::assemble(".module Image\n.type Acme.CoffeeMaker.Machine\n.end\n.type Nested\n.end\n.type Acme.CoffeeMaker.Factories.Factory\n.end\n.function Make() -> Int32\nldc.i4 0\nret\n.end").unwrap();
        let assembly: AssemblyMetadata = serde_json::from_value(serde_json::json!({
            "name": "Package", "full_name": "Package", "modules": ["Shared.dll"], "references": [],
            "declaration_modules": {"version": 1, "names": ["Acme.CoffeeMaker.Factories", "Acme.CoffeeMaker", "Acme.Empty"]}
        })).unwrap();
        for ty in &mut module.types {
            ty.origin = Some(origin("Package", &ty.name));
        }
        module.types[1].declaring_type = module.types[0].definition.clone();
        module.functions[0].origin = Some(origin("Package", "Make"));
        module.functions[0].namespace = "Acme.CoffeeMaker.Factories".into();
        module.assemblies = vec![assembly.clone()];
        let names = module_names(&module, &assembly).unwrap();
        assert_eq!(
            names,
            [
                "Acme.CoffeeMaker",
                "Acme.CoffeeMaker.Factories",
                "Acme.Empty"
            ]
        );
        let expected = module_value("Package", "Acme.CoffeeMaker");
        assert_eq!(
            type_module_value(&module, &module.types[0]).unwrap(),
            expected
        );
        assert_eq!(
            type_module_value(&module, &module.types[1]).unwrap(),
            expected
        );
        assert_eq!(
            function_module_value(&module, &module.functions[0]).unwrap(),
            module_value("Package", "Acme.CoffeeMaker.Factories")
        );
        module.functions[0].owner = Some(Type::from_name("Acme.CoffeeMaker.Machine"));
        assert_eq!(
            function_module_value(&module, &module.functions[0]).unwrap(),
            expected
        );
        let args = [
            Value::String("Package".into()),
            Value::String("Acme".into()),
        ];
        assert!(Query::ModuleTypes
            .invoke(&module, &args, &Limits::default())
            .is_err());
        let args = [
            Value::String("Package".into()),
            Value::String("Acme.Empty".into()),
        ];
        let Value::Array { elements, .. } = Query::ModuleTypes
            .invoke(&module, &args, &Limits::default())
            .unwrap()
        else {
            panic!("expected array");
        };
        assert!(elements.is_empty());
        let mut other = assembly.clone();
        other.full_name = "Other".into();
        other.name = "Other".into();
        module.assemblies.push(other);
        assert_ne!(
            module_value("Package", &names[0]),
            module_value("Other", &names[0])
        );
        let root_args = [
            Value::String("Package".into()),
            Value::String("Acme.CoffeeMaker".into()),
        ];
        let Value::Array { elements, .. } = Query::ModuleTypes
            .invoke(&module, &root_args, &Limits::default())
            .unwrap()
        else {
            panic!("expected array");
        };
        assert_eq!(elements.len(), 2); // Includes the nested type, not the dotted sibling module.
        let mut unicode = assembly.clone();
        unicode.declaration_modules.as_mut().unwrap().names =
            vec!["\u{e000}".into(), "\u{10000}".into()];
        assert_eq!(
            module_names(&module, &unicode).unwrap(),
            ["\u{10000}", "\u{e000}"]
        );
        // A legacy projection derives retained logical owners, not physical filenames or parents.
        module.assemblies[0].declaration_modules = None;
        assert_eq!(
            module_names(&module, &module.assemblies[0]).unwrap(),
            ["Acme.CoffeeMaker", "Acme.CoffeeMaker.Factories"]
        );
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

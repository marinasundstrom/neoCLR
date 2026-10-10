//! Unstable build-time bridge for the in-repository native compiler experiment.
//! Produces trusted snapshot recipes through the same metadata queries as the VM.
//! It does not invoke user code or grant executable reflection rights.
use crate::{Fault, Limits, Module, Value, metadata::Type};

/// Return the logical property snapshot recipe, before runtime allocation.
pub fn properties(module: &Module, owner: &Type) -> Result<Value, Fault> {
    crate::reflection::Query::Properties.invoke_profile(
        module,
        &[
            Value::RuntimeTypeHandle(Box::new(crate::type_identity::describe_loaded(
                module, owner,
            )?)),
            Value::Int32(60),
        ],
        &Limits::default(),
        true,
    )
}

/// Return the optional logical element descriptor recipe.
pub fn element_type(module: &Module, owner: &Type) -> Result<Value, Fault> {
    crate::reflection::Query::ElementType.invoke_profile(
        module,
        &[Value::RuntimeTypeHandle(Box::new(
            crate::type_identity::describe_loaded(module, owner)?,
        ))],
        &Limits::default(),
        true,
    )
}

/// Return a retained nominal type's logical module descriptor recipe.
/// This unstable backend bridge adds no loading or callable execution capability.
pub fn type_module(module: &Module, owner: &Type) -> Result<Value, Fault> {
    crate::reflection::Query::Module.invoke_profile(
        module,
        &[Value::RuntimeTypeHandle(Box::new(crate::type_identity::describe_loaded(module, owner)?))],
        &Limits::default(),
        true,
    )
}

/// Return the owning assembly recipe after validating exact logical module ownership.
pub fn module_assembly(module: &Module, identity: &str, name: &str) -> Result<Value, Fault> {
    crate::assembly_info::Query::ModuleAssembly.invoke(
        module,
        &[Value::String(identity.into()), Value::String(name.into())],
        &Limits::default(),
    )
}

/// Return the simple name of an assembly in the loaded descriptive catalog.
pub fn assembly_name(module: &Module, identity: &str) -> Result<Value, Fault> {
    crate::assembly_info::Query::Name.invoke(
        module,
        &[Value::String(identity.into())],
        &Limits::default(),
    )
}

/// Return a complete explicit logical module catalog, including empty declarations.
/// Legacy namespace projections cannot establish completeness and are rejected.
pub fn assembly_modules(module: &Module, identity: &str) -> Result<Value, Fault> {
    if crate::assembly_info::lookup(module, identity)?.declaration_modules.is_none() {
        return Err(Fault::new("native module catalog requires explicit declaration metadata"));
    }
    crate::assembly_info::Query::Modules.invoke(
        module,
        &[Value::String(identity.into())],
        &Limits::default(),
    )
}

/// Select free-function definition identities in metadata order, without invocation.
/// Requires an explicit assembly-local module catalog, not a legacy projection.
/// This unstable backend bridge grants no AOT body or attribute retention rights.
pub fn module_function_definitions(
    module: &Module,
    identity: &str,
    name: &str,
) -> Result<Vec<crate::metadata::MemberId>, Fault> {
    if crate::assembly_info::lookup(module, identity)?.declaration_modules.is_none() {
        return Err(Fault::new("native module function discovery requires explicit declaration metadata"));
    }
    crate::assembly_info::module_functions(module, identity, name)?
        .into_iter()
        .map(|function| function.definition.clone()
            .ok_or_else(|| Fault::new("module function requires definition identity")))
        .collect()
}

fn module_function<'a>(
    module: &'a Module,
    identity: &str,
    name: &str,
    definition: &crate::metadata::MemberId,
) -> Result<&'a crate::metadata::Function, Fault> {
    // Require the same explicit catalog and ambiguity checks as enumeration.
    if !module_function_definitions(module, identity, name)?.contains(definition) {
        return Err(Fault::new("function does not belong to assembly module"));
    }
    crate::assembly_info::module_functions(module, identity, name)?
        .into_iter().find(|function| function.definition.as_ref() == Some(definition))
        .ok_or_else(|| Fault::new("missing module function definition"))
}

/// Describe an admitted module function without binding or executing it.
/// Open generic method signatures currently fault rather than being omitted.
pub fn module_function_snapshot(
    module: &Module,
    identity: &str,
    name: &str,
    definition: &crate::metadata::MemberId,
) -> Result<Value, Fault> {
    crate::reflection::module_function_snapshot(module, module_function(module, identity, name, definition)?, true)
}

/// Read method-level attribute data, without invoking constructors or test bodies.
pub fn module_function_attributes(
    module: &Module,
    identity: &str,
    name: &str,
    definition: &crate::metadata::MemberId,
) -> Result<Value, Fault> {
    crate::reflection::module_function_attributes(module, module_function(module, identity, name, definition)?, true)
}

/// Describe the assembly of a verified lexical caller, using the VM facade rules.
pub fn executing_assembly(
    module: &Module,
    facade: &crate::metadata::Function,
    caller: &crate::metadata::Function,
) -> Result<Value, Fault> {
    let identity = crate::assembly_info::executing_assembly(module, [facade, caller].into_iter())
        .ok_or_else(|| Fault::new("ExecutingAssembly requires source caller metadata"))?;
    crate::assembly_info::assembly_value(module, &identity)
}

/// Resolve a recipe handle to its source signature, without publishing native ordinals.
pub fn handle_type(module: &Module, handle: &crate::TypeDescriptor) -> Result<Type, Fault> {
    crate::reflection::from_identity(module, &handle.identity)
}

/// Resolve the exact scoped runtime provider used by interpreter materialization.
pub fn provider(module: &Module, contract: &Type, fields: &[Value]) -> Result<Type, Fault> {
    crate::reflection_source::provider(module, contract, fields)
}

/// Resolve an inherited interface target using the VM's conformance anchor rules.
pub fn interface_implementation(
    module: &Module,
    concrete: &Type,
    contract: &crate::metadata::Function,
) -> Result<crate::metadata::Function, Fault> {
    let interface = contract
        .owner
        .as_ref()
        .ok_or_else(|| Fault::new("missing interface owner"))?;
    crate::interfaces::implementation(module, concrete, interface, contract)
}

/// Resolve closed generic argument handles using the interpreter's type identity.
pub fn generic_arguments(module: &Module, owner: &Type) -> Result<Vec<Type>, Fault> {
    crate::type_identity::describe_loaded(module, owner)?
        .generic_arguments
        .iter()
        .map(|handle| handle_type(module, handle))
        .collect()
}

/// Classify a scoped built-in collection and its ordinary concrete implementation.
pub fn collection_shape(
    module: &Module,
    service: &crate::metadata::Function,
    owner: &Type,
) -> Result<Option<(i32, Type, Type)>, Fault> {
    crate::reflection_collections::shape(module, service, owner)
}

/// Plan a call to source-owned generic collection code without executing it.
pub fn collection_plan(
    module: &Module,
    service: &crate::metadata::Function,
    owner: &Type,
    operation: u8,
) -> Result<crate::metadata::Function, Fault> {
    crate::reflection_collections::plan(module, service, owner, operation)
}

/// Validate the private collection service before generating any native dispatch.
pub fn validate_collection_service(
    module: &crate::Module,
    service: &crate::metadata::Function,
    operation: u8,
) -> Result<(), crate::Fault> {
    crate::reflection_collections::validate_service(module, service, operation)
}

/// Return declared custom-attribute data without constructing attribute instances.
pub fn custom_attributes(module: &Module, owner: &Type, token: i32) -> Result<Value, Fault> {
    crate::reflection::custom_attribute_snapshot(module, owner, token)
}

/// Return the original module-scoped metadata token, not a native layout ordinal.
pub fn type_token(module: &Module, owner: &Type) -> Result<i32, Fault> {
    let handle = crate::type_identity::describe_loaded(module, owner)?;
    crate::metadata_tokens::type_token(module, &handle.identity)
}

/// Return the same logical display name as TypeInfo.DisplayName for native lowering.
pub fn display_name(module: &Module, owner: &Type) -> Result<Value, Fault> {
    crate::reflection::Query::DisplayName.invoke_profile(
        module,
        &[Value::RuntimeTypeHandle(Box::new(crate::type_identity::describe_loaded(module, owner)?)), Value::Int32(0)],
        &Limits::default(),
        true,
    )
}

#[cfg(test)]
mod module_function_tests {
    use super::*;

    fn fixture() -> Module {
        let mut module = crate::assemble(r#"
.module Functions
.type IntegerMetadata
.end
.type StringMetadata
.end
.type UnitMetadata
.end
.type System.Introspection.CustomAttributeNamedArgument
.end
.type Marker
.field public Tag String
.method instance .ctor(String description) -> Void
fault "attribute constructor must not execute"
.end
.end
.function Probe(Int32 input) -> Int32
.custom instance Marker::.ctor(String) = [{"String":"a discovered test"}]
fault "test body must not execute"
.end
"#).unwrap();
        for (definition, name) in module.types.iter_mut().zip(["System.Int32", "System.String", "System.Void"]) {
            definition.name = name.into();
        }
        module.functions.iter_mut().find(|f| f.owner.is_none()).unwrap().namespace = "Example.Tests".into();
        module.assemblies = vec![serde_json::from_value(serde_json::json!({
            "name":"Package", "full_name":"Package", "modules":["Functions"], "references":[],
            "declaration_modules":{"version":1,"names":["", "Example.Tests", "Other"]}
        })).unwrap()];
        module
    }

    fn fields(value: &Value) -> &[Value] {
        let Value::Object { fields, .. } = value else { panic!("expected descriptor"); };
        fields
    }

    #[test]
    fn ownerless_snapshot_preserves_signature_and_module_without_a_declaring_type() {
        let module = fixture();
        let ids = module_function_definitions(&module, "Package", "Example.Tests").unwrap();
        assert_eq!(ids.len(), 1);
        let snapshot = module_function_snapshot(&module, "Package", "Example.Tests", &ids[0]).unwrap();
        let values = fields(&snapshot);
        assert_eq!(values[0], Value::String("Probe".into()));
        assert!(matches!(values[1], Value::NullObjectReference(_)));
        assert_eq!(fields(&values[3])[0], Value::Boolean(true));
        assert_eq!(fields(&fields(&values[3])[1]), [Value::String("Package".into()), Value::String("Example.Tests".into())]);
        assert_eq!(values[5], Value::Boolean(true)); // Static.
        let Value::Array { elements, .. } = &values[11] else { panic!("expected parameters"); };
        assert_eq!(elements.len(), 1);
        let parameter = fields(&elements[0]);
        assert_eq!(parameter[0], Value::String("input".into()));
        assert_eq!(parameter[1], Value::Int32(0));
        assert_eq!(parameter[9], Value::Int32(4)); // Ownerless member identity.
        assert_eq!(fields(&parameter[7]), fields(&values[3]));
        assert!(module_function_snapshot(&module, "Package", "Other", &ids[0]).is_err());
    }

    #[test]
    fn module_attribute_recipes_preserve_descriptions_and_exclude_parameter_annotations() {
        let mut module = fixture();
        let index = module.functions.iter().position(|f| f.owner.is_none()).unwrap();
        let id = module.functions[index].definition.clone().unwrap();
        module.functions[index].custom_attributes[0].named_arguments.push(crate::metadata::CustomAttributeNamedArgument {
            name: "Tag".into(), is_field: true,
            value: crate::metadata::AttributeArgument::String(Some("named description".into())),
        });
        let mut parameter_attribute = module.functions[index].custom_attributes[0].clone();
        parameter_attribute.target_token = Some(0x08000001);
        module.functions[index].custom_attributes.push(parameter_attribute);
        let result = module_function_attributes(&module, "Package", "Example.Tests", &id).unwrap();
        let Value::Array { elements, .. } = result else { panic!("expected attributes"); };
        assert_eq!(elements.len(), 1);
        let Value::Array { elements: arguments, .. } = &fields(&elements[0])[2] else { panic!("expected arguments"); };
        assert_eq!(fields(&arguments[0])[1], Value::Erased(Box::new(Value::String("a discovered test".into()))));
        let Value::Array { elements: named, .. } = &fields(&elements[0])[3] else { panic!("expected named arguments"); };
        assert_eq!(named.len(), 1);
        assert_eq!(fields(&named[0])[0], Value::String("Tag".into()));
        module.functions[index].generic_parameters = vec![Some("T".into())];
        assert!(module_function_snapshot(&module, "Package", "Example.Tests", &id).unwrap_err().to_string().contains("generic method definition"));
        // Attributes remain available even when signature materialization is unsupported.
        assert!(module_function_attributes(&module, "Package", "Example.Tests", &id).is_ok());
    }
}

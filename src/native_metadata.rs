//! Unstable build-time bridge for the in-repository native compiler experiment.
//! Produces trusted snapshot recipes through the same metadata queries as the VM.
//! It does not invoke user code or grant executable reflection rights.
use crate::{Fault, Limits, Module, Value, metadata::Type};

/// Use the runtime's relationship rules before native identity lowering.
pub fn assignable(module: &Module, source: &Type, target: &Type) -> Result<bool, Fault> {
    crate::type_relationships::assignable(module, source, target)
}

/// Strict ancestry shares the interpreter's definition and implicit Object root.
pub fn subclass(module: &Module, source: &Type, target: &Type) -> Result<bool, Fault> {
    crate::type_relationships::subclass(module, source, target)
}

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

/// Return the optional declaring-type recipe using the interpreter's identity rules.
/// Native callers must explicitly retain the queried declaration; this grants no invocation.
pub fn declaring_type(module: &Module, owner: &Type) -> Result<Value, Fault> {
    crate::reflection::Query::DeclaringType.invoke_profile(
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
        &[Value::RuntimeTypeHandle(Box::new(
            crate::type_identity::describe_loaded(module, owner)?,
        ))],
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
    if crate::assembly_info::lookup(module, identity)?
        .declaration_modules
        .is_none()
    {
        return Err(Fault::new(
            "native module catalog requires explicit declaration metadata",
        ));
    }
    crate::assembly_info::Query::Modules.invoke(
        module,
        &[Value::String(identity.into())],
        &Limits::default(),
    )
}

/// Describe all type definitions in an explicitly retained assembly catalog.
/// Includes open generic definitions; does not retain executable layouts or bodies.
pub fn assembly_types(module: &Module, identity: &str) -> Result<Value, Fault> {
    assembly_modules(module, identity)?;
    crate::assembly_info::Query::Types.invoke_profile(
        module,
        &[Value::String(identity.into())],
        &Limits::default(),
        true,
    )
}

/// Describe definitions in an exact assembly-local logical module, including empty ones.
/// Requires explicit module metadata and shares the interpreter's ownership checks.
pub fn module_types(module: &Module, identity: &str, name: &str) -> Result<Value, Fault> {
    assembly_modules(module, identity)?;
    crate::assembly_info::Query::ModuleTypes.invoke_profile(
        module,
        &[Value::String(identity.into()), Value::String(name.into())],
        &Limits::default(),
        true,
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
    if crate::assembly_info::lookup(module, identity)?
        .declaration_modules
        .is_none()
    {
        return Err(Fault::new(
            "native module function discovery requires explicit declaration metadata",
        ));
    }
    crate::assembly_info::module_functions(module, identity, name)?
        .into_iter()
        .map(|function| {
            function
                .definition
                .clone()
                .ok_or_else(|| Fault::new("module function requires definition identity"))
        })
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
        .into_iter()
        .find(|function| function.definition.as_ref() == Some(definition))
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
    crate::reflection::module_function_snapshot(
        module,
        module_function(module, identity, name, definition)?,
        true,
    )
}

/// Read method-level attribute data, without invoking constructors or test bodies.
pub fn module_function_attributes(
    module: &Module,
    identity: &str,
    name: &str,
    definition: &crate::metadata::MemberId,
) -> Result<Value, Fault> {
    crate::reflection::module_function_attributes(
        module,
        module_function(module, identity, name, definition)?,
        true,
    )
}

/// Describe the assembly of a verified lexical caller, using the VM facade rules.
pub fn executing_assembly(
    module: &Module,
    facade: &crate::metadata::Function,
    caller: &crate::metadata::Function,
) -> Result<Value, Fault> {
    let identity =
        crate::assembly_info::executing_assembly(module, [facade, caller].into_iter())
            .ok_or_else(|| Fault::new("ExecutingAssembly requires source caller metadata"))?;
    crate::assembly_info::assembly_value(module, &identity)
}

/// Resolve a recipe handle to its source signature, without publishing native ordinals.
pub fn handle_type(module: &Module, handle: &crate::TypeDescriptor) -> Result<Type, Fault> {
    crate::reflection::from_identity(module, &handle.identity)
}

/// Emit a snapshot handle without giving closed types a second native identity.
/// Open declarations and their parameters use authenticated descriptive tokens.
pub fn snapshot_token(
    target: &mut Module,
    source: &Module,
    handle: &crate::TypeDescriptor,
) -> Result<crate::metadata::Instruction, Fault> {
    use crate::type_identity::TypeIdentity;
    match &handle.identity {
        TypeIdentity::GenericParameter { definition, index } => {
            retain_definition_token(target, source, definition, Some(*index))
        }
        TypeIdentity::Definition { definition, .. }
            if handle
                .generic_arguments
                .iter()
                .any(|arg| matches!(arg.identity, TypeIdentity::GenericParameter { .. })) =>
        {
            let declaration = source
                .types
                .iter()
                .find(|d| d.definition.as_ref() == Some(definition))
                .ok_or_else(|| Fault::new("unknown snapshot definition"))?;
            if *handle != crate::type_identity::describe_definition(source, declaration)? {
                return Err(Fault::new(
                    "partially open snapshot tokens are not supported",
                ));
            }
            retain_definition_token(target, source, definition, None)
        }
        _ => Ok(crate::metadata::Instruction::LoadTypeToken(handle_type(
            source, handle,
        )?)),
    }
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

/// Resolve a class virtual slot using the same ancestry rules as interpreted calls.
/// Returns a concrete inherited implementation or a fault for an invalid receiver/slot.
pub fn class_implementation(
    module: &Module,
    concrete: &Type,
    contract: &crate::metadata::Function,
) -> Result<crate::metadata::Function, Fault> {
    crate::inheritance::dispatch(module, concrete, contract)
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
        &[
            Value::RuntimeTypeHandle(Box::new(crate::type_identity::describe_loaded(
                module, owner,
            )?)),
            Value::Int32(0),
        ],
        &Limits::default(),
        true,
    )
}

#[cfg(test)]
mod module_function_tests {
    use super::*;

    fn fixture() -> Module {
        let mut module = crate::assemble(
            r#"
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
"#,
        )
        .unwrap();
        for (definition, name) in
            module
                .types
                .iter_mut()
                .zip(["System.Int32", "System.String", "System.Void"])
        {
            definition.name = name.into();
        }
        module
            .functions
            .iter_mut()
            .find(|f| f.owner.is_none())
            .unwrap()
            .namespace = "Example.Tests".into();
        module.assemblies =
            vec![serde_json::from_value(serde_json::json!({
            "name":"Package", "full_name":"Package", "modules":["Functions"], "references":[],
            "declaration_modules":{"version":1,"names":["", "Example.Tests", "Other"]}
        })).unwrap()];
        module
    }

    fn fields(value: &Value) -> &[Value] {
        let Value::Object { fields, .. } = value else {
            panic!("expected descriptor");
        };
        fields
    }

    #[test]
    fn declaring_type_recipe_preserves_nested_identity_and_absent_owner() {
        let module = crate::assemble(".module Nested\n.type Outer\n.type Inner\n.end\n.end\n.type Other\n.type Inner\n.end\n.end\n").unwrap();
        for (nested, parent) in [("Outer.Inner", "Outer"), ("Other.Inner", "Other")] {
            let result = declaring_type(&module, &Type::Named(nested.into())).unwrap();
            assert_eq!(fields(&result)[0], Value::Boolean(true));
            let descriptor = &fields(&result)[1];
            assert_eq!(
                fields(descriptor)[0],
                Value::RuntimeTypeHandle(Box::new(
                    crate::type_identity::describe_loaded(&module, &Type::Named(parent.into()))
                        .unwrap()
                ))
            );
        }
        let result = declaring_type(&module, &Type::Named("Outer".into())).unwrap();
        assert_eq!(fields(&result), [Value::Boolean(false)]);
        assert!(declaring_type(&module, &Type::Named("Missing".into())).is_err());
    }

    #[test]
    fn ownerless_snapshot_preserves_signature_and_module_without_a_declaring_type() {
        let module = fixture();
        let ids = module_function_definitions(&module, "Package", "Example.Tests").unwrap();
        assert_eq!(ids.len(), 1);
        let snapshot =
            module_function_snapshot(&module, "Package", "Example.Tests", &ids[0]).unwrap();
        let values = fields(&snapshot);
        assert_eq!(values[0], Value::String("Probe".into()));
        assert!(matches!(values[1], Value::NullObjectReference(_)));
        assert_eq!(fields(&values[3])[0], Value::Boolean(true));
        assert_eq!(
            fields(&fields(&values[3])[1]),
            [
                Value::String("Package".into()),
                Value::String("Example.Tests".into())
            ]
        );
        assert_eq!(values[5], Value::Boolean(true)); // Static.
        let Value::Array { elements, .. } = &values[11] else {
            panic!("expected parameters");
        };
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
        let index = module
            .functions
            .iter()
            .position(|f| f.owner.is_none())
            .unwrap();
        let id = module.functions[index].definition.clone().unwrap();
        module.functions[index].custom_attributes[0]
            .named_arguments
            .push(crate::metadata::CustomAttributeNamedArgument {
                name: "Tag".into(),
                is_field: true,
                value: crate::metadata::AttributeArgument::String(Some("named description".into())),
            });
        let mut parameter_attribute = module.functions[index].custom_attributes[0].clone();
        parameter_attribute.target_token = Some(0x08000001);
        module.functions[index]
            .custom_attributes
            .push(parameter_attribute);
        let result = module_function_attributes(&module, "Package", "Example.Tests", &id).unwrap();
        let Value::Array { elements, .. } = result else {
            panic!("expected attributes");
        };
        assert_eq!(elements.len(), 1);
        let Value::Array {
            elements: arguments,
            ..
        } = &fields(&elements[0])[2]
        else {
            panic!("expected arguments");
        };
        assert_eq!(
            fields(&arguments[0])[1],
            Value::Erased(Box::new(Value::String("a discovered test".into())))
        );
        let Value::Array {
            elements: named, ..
        } = &fields(&elements[0])[3]
        else {
            panic!("expected named arguments");
        };
        assert_eq!(named.len(), 1);
        assert_eq!(fields(&named[0])[0], Value::String("Tag".into()));
        module.functions[index].generic_parameters = vec![Some("T".into())];
        assert!(
            module_function_snapshot(&module, "Package", "Example.Tests", &id)
                .unwrap_err()
                .to_string()
                .contains("generic method definition")
        );
        // Attributes remain available even when signature materialization is unsupported.
        assert!(module_function_attributes(&module, "Package", "Example.Tests", &id).is_ok());
    }
}


/// Retain a verified interface's source identity in a private native compiler shape.
/// These facts are in-memory compiler IR, not serialized metadata or new inheritance.
pub fn retain_interface_shape(
    target: &mut crate::metadata::TypeDef,
    source: &Module,
    index: usize,
    arguments: Vec<Type>,
) -> Result<(), Fault> {
    let original = source.types.get(index).ok_or_else(|| Fault::new("missing source interface"))?;
    if original.representation != crate::metadata::Representation::Interface ||
        !original.generic_variance.iter().any(|v| *v != crate::metadata::Variance::Invariant) {
        return Ok(());
    }
    if arguments.len() != original.generic_parameters.len() {
        return Err(Fault::new("native interface argument arity mismatch"));
    }
    target.native_interface = Some(crate::variance::NativeInterface {
        source: original.definition.clone().unwrap_or(crate::metadata::TypeDefId {
            module: source.name.clone(), revision: source.revision.clone(), index: index as u32,
        }),
        arguments,
        annotations: original.generic_variance.clone(),
    });
    Ok(())
}

/// Retain verified intrinsic String conformance on a closed native interface shape.
/// Callers supply verified source IR and its original (unlowered) interface type.
pub fn retain_string_view(
    target: &mut crate::metadata::TypeDef,
    source: &Module,
    interface: &Type,
) {
    target.native_string_view = crate::interfaces::interface_definition(source, interface).is_ok()
        && crate::interfaces::ensure_reference_implementation(source, &Type::String, interface)
            .is_ok();
}

/// Retain a verified array backing's closed element in private compiler IR.
/// This records nominal conformance, not a new array layout or element conversion.
pub fn retain_array_shape(
    target: &mut crate::metadata::TypeDef,
    source: &Module,
    index: usize,
    arguments: &[Type],
) -> Result<(), Fault> {
    let original = source
        .types
        .get(index)
        .ok_or_else(|| Fault::new("missing source array backing"))?;
    if !source
        .array_backing()
        .is_some_and(|backing| std::ptr::eq(backing, original))
    {
        return Ok(());
    }
    if arguments.len() != 1 || original.generic_parameters.len() != 1 || !original.is_reference_type
    {
        return Err(Fault::new(
            "native array proof requires a verified unary reference backing",
        ));
    }
    target.native_array_element = Some(arguments[0].clone());
    Ok(())
}
/// Preserve a variant contract slot across closed owner specialization.
pub fn retain_interface_slot(target: &mut crate::metadata::Function, source: &Module, index: usize) {
    let original = &source.functions[index];
    if original.owner.as_ref().is_some_and(|owner| interface_has_variance(source, owner)) {
        target.native_interface_slot = Some(original.definition.clone().unwrap_or(crate::metadata::MemberId {
            module: source.name.clone(), revision: source.revision.clone(), index: index as u32,
        }));
    }
}

/// Whether a source or private specialized interface carries declared variance.
pub fn interface_has_variance(module: &Module, interface: &Type) -> bool {
    module.type_definition(interface).is_some_and(|definition|
        definition.native_interface.is_some() || definition.generic_variance.iter()
            .any(|v| *v != crate::metadata::Variance::Invariant))
}

/// Resolve the declared interface view used to satisfy a reference conversion.
pub fn interface_view(module: &Module, source: &Type, target: &Type) -> Result<Type, Fault> {
    crate::variance::implemented_view(module, source, target)
}

/// Closed contract peers needed to preserve a specialized interface slot.
pub fn interface_slot_peers(module: &Module, function: &crate::metadata::Function) -> Vec<usize> {
    let Some(slot) = &function.native_interface_slot else { return vec![]; };
    module.functions.iter().enumerate().filter_map(|(i, f)|
        (f.native_interface_slot.as_ref() == Some(slot)).then_some(i)).collect()
}

/// Type dependencies of the private native interface proof.
pub fn interface_shape_arguments(definition: &crate::metadata::TypeDef) -> &[Type] {
    definition.native_interface.as_ref().map(|shape| shape.arguments.as_slice()).unwrap_or(&[])
}

/// Preserve the verified Object root's reference role before its private native rename.
/// This flag is compiler-owned and cannot be supplied by serialized applications.
pub fn retain_object_reference_root(
    definition: &mut crate::metadata::TypeDef,
) -> Result<(), Fault> {
    if definition.name != "System.Object"
        || !definition.is_reference_type
        || !definition.generic_parameters.is_empty()
    {
        return Err(Fault::new(
            "native Object proof requires the verified nongeneric reference root",
        ));
    }
    definition.native_object_root = true;
    Ok(())
}

/// Preserve private variance and reference-root facts across a row-preserving compiler rewrite.
/// The mapper must perform the same type renaming as the rewrite. Serialized input
/// cannot supply these facts; the original must be verified compiler IR.
pub fn restore_interface_proofs(
    target: &mut Module,
    original: &Module,
    map_type: impl Fn(&Type) -> Result<Type, String>,
) -> Result<(), Fault> {
    if target.types.len() != original.types.len() || target.functions.len() != original.functions.len() {
        return Err(Fault::new("variance proof restoration requires identical rows"));
    }
    for (target, original) in target.types.iter_mut().zip(&original.types) {
        target.native_object_root = original.native_object_root;
        target.native_string_view = original.native_string_view;
        target.native_array_element = original.native_array_element.as_ref().map(&map_type).transpose().map_err(Fault::new)?;
        target.native_interface = original.native_interface.clone();
        if let Some(shape) = &mut target.native_interface {
            shape.arguments = shape.arguments.iter().map(&map_type).collect::<Result<_, _>>().map_err(Fault::new)?;
        }
    }
    for (target, original) in target.functions.iter_mut().zip(&original.functions) {
        target.native_interface_slot = original.native_interface_slot.clone();
    }
    Ok(())
}

/// Retain a descriptive open definition or one of its owner-bound parameters.
/// This trusted compiler bridge grants no executable layout or invocation rights.
pub fn retain_definition_token(
    target: &mut Module,
    source: &Module,
    definition: &crate::metadata::TypeDefId,
    parameter: Option<u16>,
) -> Result<crate::metadata::Instruction, Fault> {
    let declaration = source
        .types
        .iter()
        .find(|d| d.definition.as_ref() == Some(definition))
        .ok_or_else(|| Fault::new("unknown metadata token definition"))?;
    if declaration.generic_parameters.is_empty() {
        return Err(Fault::new(
            "descriptive token requires an open generic definition",
        ));
    }
    let mut value = crate::type_identity::describe_definition(source, declaration)?;
    if let Some(index) = parameter {
        value = value
            .generic_arguments
            .get(index as usize)
            .cloned()
            .ok_or_else(|| Fault::new("generic parameter index out of range"))?;
    }
    let index = if let Some(index) = target
        .native_descriptors
        .iter()
        .position(|d| d.identity == value.identity)
    {
        index
    } else {
        target.native_descriptors.push(value);
        target.native_descriptors.len() - 1
    };
    Ok(crate::metadata::Instruction::LoadMetadataToken(
        u32::try_from(index).map_err(|_| Fault::new("metadata descriptor table overflow"))?,
    ))
}

/// Read a compiler-owned descriptor; serialized programs cannot supply this table.
pub fn descriptor(
    module: &Module,
    index: u32,
) -> Result<&crate::type_identity::TypeDescriptor, Fault> {
    crate::type_identity::retained_descriptor(module, index)
}

/// Preserve descriptive identity across a row-preserving private nominal rewrite.
/// Executable names may change; source descriptor identities must not change.
pub fn restore_descriptor_table(target: &mut Module, original: &Module) -> Result<(), Fault> {
    if target.types.len() != original.types.len()
        || target.functions.len() != original.functions.len()
    {
        return Err(Fault::new(
            "descriptor restoration requires a row-preserving rewrite",
        ));
    }
    target.native_descriptors = original.native_descriptors.clone();
    Ok(())
}

#[cfg(test)]
mod descriptor_token_tests {
    use super::*;
    use crate::metadata::Instruction as Op;
    use crate::type_identity::TypeIdentity;

    #[test]
    fn descriptor_tokens_preserve_owner_identity_and_reject_serialized_forgery() {
        let source = crate::assemble(".module Descriptors\n.entry Main\n.interface First<T>\n.variance out\n.end\n.interface Second<T>\n.end\n.type Plain\n.end\n.function Main() -> Int32\nldc.i4 42\nret\n.end\n").unwrap();
        let first = source.types[0].definition.as_ref().unwrap();
        let second = source.types[1].definition.as_ref().unwrap();
        let mut target = source.clone();
        let a = retain_definition_token(&mut target, &source, first, Some(0)).unwrap();
        let repeated = retain_definition_token(&mut target, &source, first, Some(0)).unwrap();
        let other = retain_definition_token(&mut target, &source, second, Some(0)).unwrap();
        assert_eq!(
            serde_json::to_value(&a).unwrap(),
            serde_json::to_value(&repeated).unwrap()
        );
        assert_ne!(
            serde_json::to_value(&a).unwrap(),
            serde_json::to_value(&other).unwrap()
        );
        let Op::LoadMetadataToken(index) = a.clone() else {
            panic!("descriptive token")
        };
        let parameter = descriptor(&target, index).unwrap();
        assert_eq!(parameter.name, "T");
        assert_eq!(
            parameter.identity,
            TypeIdentity::GenericParameter {
                definition: first.clone(),
                index: 0
            }
        );
        let definition = retain_definition_token(&mut target, &source, first, None).unwrap();
        let Op::LoadMetadataToken(index) = definition else {
            panic!("definition token")
        };
        assert_eq!(
            descriptor(&target, index).unwrap().generic_arguments[0].identity,
            TypeIdentity::GenericParameter {
                definition: first.clone(),
                index: 0
            }
        );
        assert!(retain_definition_token(&mut target, &source, first, Some(1)).is_err());
        assert!(
            retain_definition_token(
                &mut target,
                &source,
                source.types[2].definition.as_ref().unwrap(),
                None
            )
            .is_err()
        );
        target.functions[0].body = vec![
            a,
            repeated,
            Op::Equal,
            Op::BranchFalse(6),
            Op::Int(42),
            Op::Return,
            Op::Int(1),
            Op::Return,
        ];
        let program = crate::LoadedProgram::new(&target).unwrap();
        program.verify().unwrap();
        assert_eq!(
            program.run(crate::Limits::default()).unwrap().value,
            Value::Int32(42)
        );
        let encoded = serde_json::to_value(&target).unwrap();
        assert!(encoded.get("native_descriptors").is_none());
        let decoded: Module = serde_json::from_value(encoded.clone()).unwrap();
        assert!(
            crate::LoadedProgram::new(&decoded)
                .and_then(|p| p.verify())
                .is_err()
        );
        let mut forged = encoded;
        forged["native_descriptors"] = serde_json::json!([]);
        assert!(serde_json::from_value::<Module>(forged).is_err());
    }
}

/// Evaluate a descriptive shape fact using the interpreter's source metadata rules.
pub fn descriptor_shape(
    module: &Module,
    descriptor: &crate::TypeDescriptor,
    selector: i32,
) -> Result<bool, Fault> {
    match crate::reflection::Query::Shape.invoke_profile(
        module,
        &[
            Value::RuntimeTypeHandle(Box::new(descriptor.clone())),
            Value::Int32(selector),
        ],
        &Limits::default(),
        true,
    )? {
        Value::Boolean(value) => Ok(value),
        _ => Err(Fault::new("invalid descriptor shape result")),
    }
}

/// Query full display names or namespaces using source descriptor identity.
pub fn descriptor_display_name(
    module: &Module,
    descriptor: &crate::TypeDescriptor,
    selector: i32,
) -> Result<String, Fault> {
    match crate::reflection::Query::DisplayName.invoke_profile(
        module,
        &[
            Value::RuntimeTypeHandle(Box::new(descriptor.clone())),
            Value::Int32(selector),
        ],
        &Limits::default(),
        true,
    )? {
        Value::String(value) => Ok(value.to_string()),
        _ => Err(Fault::new("invalid descriptor display name result")),
    }
}

/// Query the display name of an ordinary closed type without losing source identity.
pub fn type_display_name(module: &Module, owner: &Type, selector: i32) -> Result<String, Fault> {
    descriptor_display_name(
        module,
        &crate::type_identity::describe_loaded(module, owner)?,
        selector,
    )
}

/// Preserve source definition tokens; parameters have no TypeDef token.
pub fn descriptor_metadata_token(
    module: &Module,
    descriptor: &crate::TypeDescriptor,
) -> Result<i32, Fault> {
    crate::metadata_tokens::type_token(module, &descriptor.identity)
}

#[cfg(test)]
mod type_catalog_tests {
    use super::*;

    fn handles(value: Value) -> Vec<crate::TypeDescriptor> {
        let Value::Array { elements, .. } = value else {
            panic!("type array")
        };
        elements
            .into_iter()
            .map(|value| {
                let Value::Object { fields, ty } = value else {
                    panic!("type snapshot")
                };
                assert_eq!(ty, Type::Named("System.Introspection.TypeInfo".into()));
                let [Value::RuntimeTypeHandle(handle)] = fields.as_slice() else {
                    panic!("type handle")
                };
                (**handle).clone()
            })
            .collect()
    }

    #[test]
    fn discovery_preserves_open_definitions_and_exact_module_ownership() {
        let mut source = crate::assemble(".module Catalog\n.interface Example.Producer<T>\n.variance out\n.end\n.type Other.Plain\n.end\n").unwrap();
        source.assemblies = serde_json::from_value(serde_json::json!([
            {"name":"Package", "full_name":"Package, Version=1.0.0.0", "modules":["Catalog"], "references":[],
             "declaration_modules":{"version":1,"names":["Example","Other","Empty"]}},
            {"name":"Foreign", "full_name":"Foreign", "modules":["Foreign.dll"], "references":[],
             "declaration_modules":{"version":1,"names":["ForeignOnly"]}},
            {"name":"Legacy", "full_name":"Legacy", "modules":["Legacy.dll"], "references":[]}
        ])).unwrap();
        let identity = "Package, Version=1.0.0.0";
        let all = handles(assembly_types(&source, identity).unwrap());
        assert_eq!(all.len(), 2);
        for (actual, definition) in all.iter().zip(&source.types) {
            assert_eq!(
                *actual,
                crate::type_identity::describe_definition(&source, definition).unwrap()
            );
        }
        assert_eq!(all[0].generic_arguments.len(), 1);
        assert_eq!(all[0].generic_arguments[0].name, "T");
        assert!(descriptor_shape(&source, &all[0].generic_arguments[0], 18).unwrap());
        assert!(all[1].generic_arguments.is_empty());
        assert_eq!(
            handles(module_types(&source, identity, "Example").unwrap()),
            vec![all[0].clone()]
        );
        assert_eq!(
            handles(module_types(&source, identity, "Other").unwrap()),
            vec![all[1].clone()]
        );
        assert!(handles(module_types(&source, identity, "Empty").unwrap()).is_empty());
        assert!(handles(assembly_types(&source, "Foreign").unwrap()).is_empty());
        for bad in ["Package", "Missing", "Legacy"] {
            assert!(assembly_types(&source, bad).is_err());
        }
        assert!(module_types(&source, identity, "ForeignOnly").is_err());
        assert!(module_types(&source, identity, "Missing").is_err());
    }
}

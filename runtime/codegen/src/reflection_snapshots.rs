//! Materialize VM-authored metadata recipes with ordinary traced runtime objects.
use neoclr::{
    Value as V,
    metadata::{FunctionRef, Instruction as Op, Type, TypeDefId, Visibility},
};
use serde_json::{Value, json};
type Error = Box<dyn std::error::Error>;
fn name<'a>(m: &'a neoclr::Module, t: &'a Type) -> Option<&'a str> {
    m.type_definition(t)
        .and_then(|d| d.origin.as_ref())
        .map(|o| o.name.as_str())
        .or_else(|| t.definition_name())
}
fn concrete(t: &Type) -> bool {
    match t {
        Type::TypeParameter(_) | Type::MethodTypeParameter(_) => false,
        Type::Constructed { arguments, .. } => arguments.iter().all(concrete),
        Type::ArrayRef(t) => concrete(t),
        _ => true,
    }
}
struct Factory<'a> {
    input: &'a mut neoclr::Module,
    source: &'a neoclr::Module,
    body: Vec<Op>,
    locals: Vec<Type>,
    trusted: Vec<Value>,
}
impl Factory<'_> {
    fn local(&mut self, t: Type) -> usize {
        let i = self.locals.len();
        self.locals.push(t);
        i
    }
    fn constructor(&mut self, owner: &Type, args: &[Type]) -> Result<FunctionRef, Error> {
        let arguments = match owner {
            Type::Constructed { arguments, .. } => arguments.as_slice(),
            _ => &[],
        };
        let candidates: Vec<_> = self
            .input
            .functions
            .iter()
            .enumerate()
            .filter(|(_, f)| {
                f.instance
                    && f.name.ends_with("..ctor")
                    && f.owner.as_ref().and_then(Type::definition_name) == owner.definition_name()
            })
            .filter_map(|(i, f)| {
                let parameters = f
                    .parameters
                    .iter()
                    .map(|p| p.substitute_type_parameters(arguments))
                    .collect::<Result<Vec<_>, _>>()
                    .ok()?;
                (parameters == args).then_some(i)
            })
            .collect();
        let [i] = candidates.as_slice() else {
            return Err(
                format!("missing unique native snapshot constructor: {owner:?} {args:?}").into(),
            );
        };
        let f = &mut self.input.functions[*i];
        if !f.no_result || f.is_internal_call() || f.pinvoke.is_some() {
            return Err("invalid native snapshot constructor contract".into());
        }
        if f.visibility == Visibility::Private {
            // Original guest access was already verified. Only these runtime-owned
            // recipe factories receive internal backend access, recorded below.
            self.trusted
                .push(json!({"constructor":f.definition,"owner":owner}));
            f.visibility = Visibility::Internal;
        }
        Ok(FunctionRef {
            definition: f.definition.clone(),
            name: f.name.clone(),
            owner: Some(owner.clone()),
            instance: true,
            parameters: args.to_vec(),
            generic_arguments: vec![],
        })
    }
    fn emit(&mut self, value: &V, expected: &Type) -> Result<(), Error> {
        match value {
            V::Int32(n) if *expected == Type::Int32 => self.body.push(Op::Int(*n)),
            V::Boolean(b) if *expected == Type::Boolean => self.body.push(Op::Bool(*b)),
            V::String(s) if *expected == Type::String => self.body.push(Op::String(s.to_string())),
            V::RuntimeTypeHandle(h) if *expected == Type::RuntimeTypeHandle => self.body.push(
                neoclr::native_metadata::snapshot_token(self.input, self.source, h)
                    .map_err(|e| e.to_string())?,
            ),
            V::Erased(value) => {
                if !self.source.is_object_reference_type(expected) {
                    return Err("snapshot boxed value requires object reference".into());
                }
                let ty = match value.as_ref() {
                    V::Int32(_) => Type::Int32,
                    V::Boolean(_) => Type::Boolean,
                    V::String(_) => Type::String,
                    _ => return Err("unsupported boxed snapshot value".into()),
                };
                self.emit(value, &ty)?;
                self.body.push(Op::BoxValue(ty));
                self.body.push(Op::CastClass(expected.clone()));
            }
            V::NullObjectReference(_) => {
                if !self.source.is_object_reference_type(expected) {
                    return Err("snapshot null signature mismatch".into());
                }
                let i = self.local(expected.clone());
                self.body.extend([
                    Op::LocalAddress(i),
                    Op::InitializeObject(expected.clone()),
                    Op::Load(i),
                ]);
            }
            V::Object { ty, fields }
                if ty.definition_name() == Some("$ReflectionSnapshot.Option") =>
            {
                if !matches!(
                    name(self.source, expected),
                    Some("System.Option" | "System.Option`1")
                ) {
                    return Err("snapshot requires scoped Option".into());
                }
                let layout = self
                    .source
                    .instantiated_fields(expected)
                    .map_err(|e| e.to_string())?;
                let [tag, some, none] = layout.as_slice() else {
                    return Err("invalid snapshot Option layout".into());
                };
                if tag.ty != Type::Byte {
                    return Err("invalid snapshot Option tag".into());
                }
                let present = matches!(fields.first(), Some(V::Boolean(true)));
                if fields.len() != if present { 2 } else { 1 } {
                    return Err("invalid snapshot Option recipe".into());
                }
                let case = if present { &some.ty } else { &none.ty };
                let args = if present {
                    let layout = self
                        .source
                        .instantiated_fields(case)
                        .map_err(|e| e.to_string())?;
                    let [field] = layout.as_slice() else {
                        return Err("invalid snapshot Some case".into());
                    };
                    self.emit(&fields[1], &field.ty)?;
                    vec![field.ty.clone()]
                } else {
                    vec![]
                };
                let ctor = self.constructor(case, &args)?;
                self.body.push(Op::Construct(ctor));
                let ctor = self.constructor(expected, std::slice::from_ref(case))?;
                self.body.push(Op::Construct(ctor));
            }
            V::Array { elements, .. } => {
                if name(self.source, expected)
                    == Some("System.Runtime.CompilerServices.ParameterSnapshot")
                {
                    let layout = self
                        .source
                        .instantiated_fields(expected)
                        .map_err(|e| e.to_string())?;
                    let [field] = layout.as_slice() else {
                        return Err("invalid ParameterSnapshot layout".into());
                    };
                    if field.name != "items" {
                        return Err("invalid ParameterSnapshot field".into());
                    }
                    self.emit(value, &field.ty)?;
                    let ctor = self.constructor(expected, std::slice::from_ref(&field.ty))?;
                    self.body.push(Op::Construct(ctor));
                } else {
                    let Type::ArrayRef(element) = expected else {
                        return Err("snapshot requires reference array".into());
                    };
                    let i = self.local(expected.clone());
                    self.body.extend([
                        Op::Int(elements.len().try_into()?),
                        Op::ReserveArray((**element).clone()),
                        Op::Store(i),
                    ]);
                    for (n, v) in elements.iter().enumerate() {
                        self.body.extend([Op::Load(i), Op::Int(n.try_into()?)]);
                        self.emit(v, element)?;
                        self.body.push(Op::StoreArrayElement((**element).clone()));
                    }
                    self.body.push(Op::Load(i));
                }
            }
            V::Object { ty, fields } => {
                let logical = ty.definition_name().ok_or("invalid snapshot name")?;
                if name(self.source, expected) != Some(logical) {
                    return Err(
                        format!("snapshot signature mismatch {logical} {expected:?}").into(),
                    );
                }
                if logical == "System.Introspection.TypeInfo" {
                    let [V::RuntimeTypeHandle(handle)] = fields.as_slice() else {
                        return Err("invalid TypeInfo recipe".into());
                    };
                    let provider = self
                        .input
                        .types
                        .iter()
                        .find(|t| {
                            t.origin
                                .as_ref()
                                .is_some_and(|o| o.name == "System.Introspection.RuntimeTypeInfo")
                        })
                        .ok_or("missing RuntimeTypeInfo factory")?;
                    let f = self
                        .input
                        .functions
                        .iter()
                        .find(|f| {
                            f.owner == Some(Type::Named(provider.name.clone()))
                                && f.origin.as_ref().is_some_and(|o| o.name == "FromHandle")
                                && f.parameters == vec![Type::RuntimeTypeHandle]
                        })
                        .ok_or("missing FromHandle factory")?;
                    let call = FunctionRef {
                        definition: f.definition.clone(),
                        name: f.name.clone(),
                        owner: f.owner.clone(),
                        instance: false,
                        parameters: f.parameters.clone(),
                        generic_arguments: vec![],
                    };
                    self.body.extend([
                        neoclr::native_metadata::snapshot_token(self.input, self.source, handle)
                            .map_err(|e| e.to_string())?,
                        Op::Call(call),
                    ]);
                } else {
                    let owner = if matches!(
                        logical,
                        "System.Introspection.CustomAttributeData"
                            | "System.Introspection.CustomAttributeNamedArgument"
                            | "System.Introspection.CustomAttributeTypedArgument"
                    ) {
                        expected.clone()
                    } else {
                        neoclr::native_metadata::provider(self.source, expected, fields)
                            .map_err(|e| e.to_string())?
                    };
                    let layout = self
                        .source
                        .instantiated_fields(&owner)
                        .map_err(|e| e.to_string())?;
                    if layout.len() != fields.len() {
                        return Err(format!("snapshot field count mismatch {owner:?}").into());
                    }
                    let (skip_module, identity) = match logical {
                        "System.Introspection.PropertyInfo"
                        | "System.Introspection.FieldInfo"
                        | "System.Introspection.ConstructorInfo" => (true, None),
                        "System.Introspection.CustomAttributeData"
                        | "System.Introspection.CustomAttributeNamedArgument"
                        | "System.Introspection.CustomAttributeTypedArgument" => (false, None),
                        "System.Introspection.MethodInfo" => (true, Some("StoredIdentity")),
                        "System.Introspection.ParameterInfo" => {
                            (false, Some("StoredMemberIdentity"))
                        }
                        "System.Introspection.ModuleInfo" | "System.Introspection.AssemblyInfo" => {
                            (false, None)
                        }
                        _ => {
                            return Err(
                                format!("unsupported native snapshot provider {logical}").into()
                            );
                        }
                    };
                    let identity_index = identity
                        .map(|n| {
                            layout
                                .iter()
                                .position(|f| f.name == n)
                                .ok_or("missing snapshot identity field")
                        })
                        .transpose()?;
                    let mut args = vec![];
                    for (i, (v, field)) in fields.iter().zip(&layout).enumerate() {
                        if skip_module && i == 3 || identity_index == Some(i) {
                            continue;
                        }
                        self.emit(v, &field.ty)?;
                        args.push(field.ty.clone());
                    }
                    let ctor = self.constructor(&owner, &args)?;
                    self.body.push(Op::Construct(ctor));
                    if let Some(i) = identity_index {
                        let t = self
                            .input
                            .types
                            .iter_mut()
                            .find(|t| Some(t.name.as_str()) == owner.definition_name())
                            .ok_or("missing snapshot provider")?;
                        let field = t
                            .fields
                            .iter_mut()
                            .find(|f| Some(f.name.as_str()) == identity)
                            .ok_or("missing snapshot identity field")?;
                        if field.ty != Type::String {
                            return Err("invalid snapshot identity field contract".into());
                        }
                        field.visibility = Visibility::Internal;
                        self.trusted
                            .push(json!({"identityField":identity,"owner":owner}));
                        self.body.push(Op::Dup);
                        self.emit(&fields[i], &Type::String)?;
                        self.body.push(Op::SetField(i));
                    }
                    if logical == "System.Introspection.MethodInfo"
                        && matches!(fields.get(1), Some(V::NullObjectReference(_)))
                    {
                        // Ownerless methods have no declaring type from which the
                        // managed provider can derive its module. Install only the
                        // runtime-owned metadata field, never execute the target.
                        let field = layout.get(3).ok_or("missing method module slot")?;
                        if field.name != "StoredModule" {
                            return Err("invalid method module slot".into());
                        }
                        let mut declaration = owner.clone();
                        loop {
                            let index = self
                                .input
                                .types
                                .iter()
                                .position(|t| {
                                    Some(t.name.as_str()) == declaration.definition_name()
                                })
                                .ok_or("missing module field owner")?;
                            if let Some(slot) = self.input.types[index]
                                .fields
                                .iter_mut()
                                .find(|f| f.name == "StoredModule")
                            {
                                if slot.ty != field.ty {
                                    return Err("invalid method module field contract".into());
                                }
                                slot.visibility = Visibility::Internal;
                                break;
                            }
                            declaration = self.input.types[index]
                                .base
                                .clone()
                                .ok_or("missing inherited module field")?;
                        }
                        self.trusted
                            .push(json!({"moduleField":"StoredModule","owner":declaration}));
                        self.body.push(Op::Dup);
                        self.emit(&fields[3], &field.ty)?;
                        self.body.push(Op::SetField(3));
                    }
                    self.body.push(Op::CastClass(expected.clone()));
                }
            }
            _ => {
                return Err(format!(
                    "unsupported native snapshot value {value:?} for {expected:?}"
                )
                .into());
            }
        }
        Ok(())
    }
}

fn emit_filtered_members(
    factory: &mut Factory<'_>,
    expected: &Type,
    element: &Type,
    elements: &[V],
    masks: &[(i32, i32)],
) -> Result<(), Error> {
    let array = factory.local(expected.clone());
    let count = factory.local(Type::Int32);
    factory.body.extend([Op::Int(0), Op::Store(count)]);
    if masks.len() != elements.len() {
        return Err("member recipe inventory mismatch".into());
    }
    for &(scope, access) in masks {
        predicate(&mut factory.body, scope, access);
        let skip = factory.body.len();
        factory.body.push(Op::BranchFalse(0));
        factory
            .body
            .extend([Op::Load(count), Op::Int(1), Op::Add, Op::Store(count)]);
        factory.body[skip] = Op::BranchFalse(factory.body.len());
    }
    factory.body.extend([
        Op::Load(count),
        Op::ReserveArray(element.clone()),
        Op::Store(array),
        Op::Int(0),
        Op::Store(count),
    ]);
    for (v, &(scope, access)) in elements.iter().zip(masks) {
        predicate(&mut factory.body, scope, access);
        let skip = factory.body.len();
        factory.body.push(Op::BranchFalse(0));
        factory.body.extend([Op::Load(array), Op::Load(count)]);
        factory.emit(v, element)?;
        factory.body.push(Op::StoreArrayElement(element.clone()));
        factory
            .body
            .extend([Op::Load(count), Op::Int(1), Op::Add, Op::Store(count)]);
        factory.body[skip] = Op::BranchFalse(factory.body.len());
    }
    factory.body.extend([Op::Load(array), Op::Return]);
    Ok(())
}

pub fn bind(
    input: &mut neoclr::Module,
    source: &neoclr::Module,
    definitions: &[TypeDefId],
    retention: &mut Value,
) -> Result<(), Error> {
    let services: Vec<_> = input
        .functions
        .iter()
        .enumerate()
        .filter(|(_, f)| {
            matches!(
                f.name.as_str(),
                "neoCLR.Runtime.TypeFields"
                    | "neoCLR.Runtime.TypeConstructors"
                    | "neoCLR.Runtime.TypeMethods"
                    | "neoCLR.Runtime.TypeProperties"
                    | "neoCLR.Runtime.TypeModule"
                    | "neoCLR.Runtime.TypeDeclaringType"
                    | "neoCLR.Runtime.ModuleAssembly"
                    | "neoCLR.Runtime.AssemblyName"
                    | "neoCLR.Runtime.AssemblyModules"
                    | "neoCLR.Runtime.AssemblyTypes"
                    | "neoCLR.Runtime.ModuleTypes"
                    | "neoCLR.Runtime.TypeElementType"
                    | "neoCLR.Runtime.TypeBaseType"
                    | "neoCLR.Runtime.MemberCustomAttributes"
                    | "neoCLR.Runtime.ModuleMemberCustomAttributes"
                    | "neoCLR.Runtime.ModuleFunctions"
            )
        })
        .map(|(i, _)| i)
        .collect();
    // Close the vector inventory over explicit generic call arguments and retained
    // property signatures, before snapshot factories introduce their own tokens.
    fn vectors(t: &Type, tokens: &mut Vec<Type>) {
        match t {
            Type::ArrayRef(element) => {
                if concrete(t) && !tokens.contains(t) {
                    tokens.push(t.clone());
                }
                vectors(element, tokens);
            }
            Type::Constructed { arguments, .. } => {
                for argument in arguments {
                    vectors(argument, tokens);
                }
            }
            _ => {}
        }
    }
    let mut base_tokens = vec![];
    let mut method_tokens = vec![];
    let mut tokens = vec![];
    for op in input.functions.iter().flat_map(|f| &f.body) {
        let token = match op {
            Op::LoadTypeToken(t) => Some(t),
            _ => super::reflection_arrays::retained_array_type(op),
        };
        if let Some(t) = token {
            vectors(t, &mut tokens);
            if concrete(t) && !method_tokens.contains(t) {
                method_tokens.push(t.clone());
            }
            if concrete(t) && !base_tokens.contains(t) {
                base_tokens.push(t.clone());
            }
        }
        match op {
            Op::Call(target)
            | Op::CallVirtual(target)
            | Op::Construct(target)
            | Op::BindFunction { target, .. } => {
                for argument in &target.generic_arguments {
                    vectors(argument, &mut tokens);
                    // typeof(T) is closed during later specialization. Retained
                    // method views must include explicit generic call arguments.
                    if concrete(argument) && !method_tokens.contains(argument) {
                        method_tokens.push(argument.clone());
                    }
                }
            }
            _ => {}
        }
    }
    for row in retention["types"].as_array().unwrap() {
        if row["properties"] != true {
            continue;
        }
        let id: TypeDefId = serde_json::from_value(row["definition"].clone())?;
        let ti = definitions
            .iter()
            .position(|d| *d == id)
            .ok_or("missing snapshot root")?;
        for property in &source.types[ti].properties {
            vectors(&property.ty, &mut tokens);
        }
    }
    let context_queries = super::runtime_context::prepare(input, source)?;
    // Catalog retention is descriptive only: it roots all declared names, not their types.
    let mut retained_modules = vec![];
    let mut catalogs = vec![];
    for identity in retention["moduleCatalogs"].as_array().into_iter().flatten() {
        let identity = identity.as_str().ok_or("invalid module catalog identity")?;
        let recipe = neoclr::native_metadata::assembly_modules(source, identity)
            .map_err(|error| error.to_string())?;
        let V::Array { elements, .. } = &recipe else {
            return Err("invalid module catalog snapshot".into());
        };
        retained_modules.extend(elements.iter().cloned());
        catalogs.push((identity, recipe));
    }
    if services.iter().any(|index| {
        matches!(
            input.functions[*index].name.as_str(),
            "neoCLR.Runtime.ModuleAssembly" | "neoCLR.Runtime.AssemblyName"
        )
    }) {
        for row in retention["types"].as_array().unwrap() {
            let id: TypeDefId = serde_json::from_value(row["definition"].clone())?;
            let ti = definitions
                .iter()
                .position(|definition| *definition == id)
                .ok_or("missing module snapshot root")?;
            retained_modules.push(
                neoclr::native_metadata::type_module(
                    source,
                    &Type::Named(source.types[ti].name.clone()),
                )
                .map_err(|error| error.to_string())?,
            );
        }
    }
    let mut projections = vec![];
    for index in services {
        let f = &input.functions[index];
        let modules = f.name.ends_with("TypeModule");
        let declaring = f.name.ends_with("TypeDeclaringType");
        let base_type = f.name == "neoCLR.Runtime.TypeBaseType";
        let module_assembly = f.name.ends_with("ModuleAssembly");
        let assembly_name = f.name.ends_with("AssemblyName");
        let assembly_modules = f.name.ends_with("AssemblyModules");
        let assembly_types = f.name == "neoCLR.Runtime.AssemblyTypes";
        let module_types = f.name == "neoCLR.Runtime.ModuleTypes";
        let properties = f.name.ends_with("Properties");
        let fields = f.name.ends_with("TypeFields");
        let constructors = f.name.ends_with("TypeConstructors");
        let methods = f.name.ends_with("TypeMethods");
        let members = fields || constructors;
        let attributes = f.name == "neoCLR.Runtime.MemberCustomAttributes";
        let module_attributes = f.name == "neoCLR.Runtime.ModuleMemberCustomAttributes";
        let module_functions = f.name == "neoCLR.Runtime.ModuleFunctions";
        let attribute_parameters = attributes
            && f.parameters.len() == 2
            && name(source, &f.parameters[0]) == Some("System.Introspection.TypeInfo")
            && f.parameters[1] == Type::Int32;
        if f.instance
            || f.owner.is_some()
            || (attributes && !attribute_parameters)
            || (!attributes
                && f.parameters
                    != if module_attributes {
                        vec![Type::String, Type::String, Type::Int32]
                    } else if module_assembly || module_functions || module_types {
                        vec![Type::String, Type::String]
                    } else if assembly_name || assembly_modules || assembly_types {
                        vec![Type::String]
                    } else if properties || members || methods {
                        vec![Type::RuntimeTypeHandle, Type::Int32]
                    } else {
                        vec![Type::RuntimeTypeHandle]
                    })
            || f.impl_flags != 0x1000
            || !f.body.is_empty()
            || !f.locals.is_empty()
            || f.no_result
            || f.pinvoke.is_some()
            || !f.generic_parameters.is_empty()
            || !f.generic_arguments.is_empty()
            || f.receiver_byref
            || f.receiver_readonly
            || f.is_virtual
            || f.is_abstract
            || f.is_override
            || !f.generic_constraints.is_empty()
            || !f.interface_implementations.is_empty()
            || !f.out_parameters.is_empty()
            || !f.out_when_true.is_empty()
            || !f.readonly_parameters.is_empty()
        {
            return Err("snapshot query requires exact reserved InternalCall contract".into());
        }
        let expected = f.returns.clone();
        let mut factory = Factory {
            input,
            source,
            body: vec![],
            locals: vec![],
            trusted: vec![],
        };
        if assembly_types || module_types {
            let Type::ArrayRef(element) = &expected else {
                return Err("type discovery requires TypeInfo vector".into());
            };
            if name(source, element) != Some("System.Introspection.TypeInfo") {
                return Err("type discovery requires scoped TypeInfo vector".into());
            }
            for identity in retention["typeCatalogs"].as_array().into_iter().flatten() {
                let identity = identity.as_str().ok_or("invalid type catalog identity")?;
                let first = factory.body.len() + 3;
                factory.body.extend([
                    Op::Arg(0),
                    Op::String(identity.into()),
                    Op::Equal,
                    Op::BranchFalse(0),
                ]);
                if assembly_types {
                    let recipe = neoclr::native_metadata::assembly_types(source, identity)
                        .map_err(|e| e.to_string())?;
                    factory.emit(&recipe, &expected)?;
                    factory.body.push(Op::Return);
                } else {
                    let V::Array { elements, .. } =
                        neoclr::native_metadata::assembly_modules(source, identity)
                            .map_err(|e| e.to_string())?
                    else {
                        return Err("invalid module catalog".into());
                    };
                    for module in elements {
                        let V::Object { fields, .. } = module else {
                            return Err("invalid module recipe".into());
                        };
                        let [V::String(_), V::String(module_name)] = fields.as_slice() else {
                            return Err("invalid module identity".into());
                        };
                        let second = factory.body.len() + 3;
                        factory.body.extend([
                            Op::Arg(1),
                            Op::String(module_name.to_string()),
                            Op::Equal,
                            Op::BranchFalse(0),
                        ]);
                        let recipe = neoclr::native_metadata::module_types(
                            source,
                            identity,
                            module_name.as_str(),
                        )
                        .map_err(|e| e.to_string())?;
                        factory.emit(&recipe, &expected)?;
                        factory.body.push(Op::Return);
                        factory.body[second] = Op::BranchFalse(factory.body.len());
                    }
                }
                factory.body[first] = Op::BranchFalse(factory.body.len());
            }
            factory.body.push(Op::Fault("native type catalog was not retained or module is not owned by assembly; configure --reflection-roots".into()));
        } else if module_attributes || module_functions {
            let Type::ArrayRef(element) = &expected else {
                return Err("module member query requires a vector result".into());
            };
            let logical = if module_attributes {
                "System.Introspection.CustomAttributeData"
            } else {
                "System.Introspection.MethodInfo"
            };
            if name(source, element) != Some(logical) {
                return Err(format!("module member query requires scoped {logical}").into());
            }
            for row in retention["moduleFunctions"]
                .as_array()
                .into_iter()
                .flatten()
            {
                if module_attributes && row["customAttributes"] != true {
                    continue;
                }
                let identity = row["assembly"]
                    .as_str()
                    .ok_or("invalid retained function assembly")?;
                let module_name = row["name"]
                    .as_str()
                    .ok_or("invalid retained function module")?;
                let first = factory.body.len() + 3;
                factory.body.extend([
                    Op::Arg(0),
                    Op::String(identity.into()),
                    Op::Equal,
                    Op::BranchFalse(0),
                ]);
                let second = factory.body.len() + 3;
                factory.body.extend([
                    Op::Arg(1),
                    Op::String(module_name.into()),
                    Op::Equal,
                    Op::BranchFalse(0),
                ]);
                let definitions = neoclr::native_metadata::module_function_definitions(
                    source,
                    identity,
                    module_name,
                )
                .map_err(|error| error.to_string())?;
                if module_functions {
                    let elements = definitions
                        .iter()
                        .map(|id| {
                            neoclr::native_metadata::module_function_snapshot(
                                source,
                                identity,
                                module_name,
                                id,
                            )
                            .map_err(|error| error.to_string())
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    factory.emit(
                        &V::Array {
                            element: Type::Named(logical.into()),
                            elements,
                        },
                        &expected,
                    )?;
                    factory.body.push(Op::Return);
                } else {
                    for id in definitions {
                        let target = source
                            .functions
                            .iter()
                            .find(|f| f.definition.as_ref() == Some(&id))
                            .ok_or("missing retained module function")?;
                        let token = target
                            .origin
                            .as_ref()
                            .ok_or("module attributes require source token")?
                            .token;
                        let token: i32 = token.try_into()?;
                        let skip = factory.body.len() + 3;
                        factory.body.extend([
                            Op::Arg(2),
                            Op::Int(token),
                            Op::Equal,
                            Op::BranchFalse(0),
                        ]);
                        let recipe = neoclr::native_metadata::module_function_attributes(
                            source,
                            identity,
                            module_name,
                            &id,
                        )
                        .map_err(|error| error.to_string())?;
                        factory.emit(&recipe, &expected)?;
                        factory.body.push(Op::Return);
                        factory.body[skip] = Op::BranchFalse(factory.body.len());
                    }
                }
                factory.body[first] = Op::BranchFalse(factory.body.len());
                factory.body[second] = Op::BranchFalse(factory.body.len());
            }
            factory.body.push(Op::Fault(if module_attributes {
                "native module-member custom attribute metadata was not retained; configure --reflection-roots"
            } else { "native module function metadata was not retained; configure --reflection-roots" }.into()));
        } else if attributes {
            bind_attributes(&mut factory, index, definitions, retention, &expected)?;
        } else if assembly_modules {
            let Type::ArrayRef(element) = &expected else {
                return Err("AssemblyModules requires ModuleInfo vector".into());
            };
            if name(source, element) != Some("System.Introspection.ModuleInfo") {
                return Err("AssemblyModules requires scoped ModuleInfo vector".into());
            }
            for (identity, recipe) in &catalogs {
                let skip = factory.body.len() + 3;
                factory.body.extend([
                    Op::Arg(0),
                    Op::String((*identity).into()),
                    Op::Equal,
                    Op::BranchFalse(0),
                ]);
                factory.emit(recipe, &expected)?;
                factory.body.push(Op::Return);
                factory.body[skip] = Op::BranchFalse(factory.body.len());
            }
            factory.body.push(Op::Fault(
                "native assembly module catalog was not retained".into(),
            ));
        } else if module_assembly || assembly_name {
            if module_assembly
                && name(source, &expected) != Some("System.Introspection.AssemblyInfo")
                || assembly_name && expected != Type::String
            {
                return Err("assembly snapshot query requires exact result contract".into());
            }
            let mut seen = std::collections::HashSet::new();
            let mut owners = vec![];
            for recipe in &retained_modules {
                let V::Object { fields, .. } = recipe else {
                    return Err("invalid logical module snapshot".into());
                };
                let [V::String(identity), V::String(module_name)] = fields.as_slice() else {
                    return Err("invalid logical module identity".into());
                };
                owners.push((identity.as_str(), Some(module_name.as_str())));
            }
            // An assembly can have an explicit empty catalog without a global module.
            if assembly_name {
                owners.extend(catalogs.iter().map(|(identity, _)| (*identity, None)));
                owners.extend(
                    context_queries
                        .iter()
                        .flat_map(|query| query.assemblies.iter())
                        .map(|(identity, _)| (identity.as_str(), None)),
                );
            }
            for (identity, module_name) in owners {
                let key = (identity, if module_assembly { module_name } else { None });
                if !seen.insert(key) {
                    continue;
                }
                let mut skips = vec![];
                let arguments =
                    std::iter::once(identity).chain(module_name.filter(|_| module_assembly));
                for (argument, value) in arguments.enumerate() {
                    factory.body.extend([
                        Op::Arg(argument),
                        Op::String(value.to_string()),
                        Op::Equal,
                    ]);
                    skips.push(factory.body.len());
                    factory.body.push(Op::BranchFalse(0));
                }
                let value = if module_assembly {
                    neoclr::native_metadata::module_assembly(
                        source,
                        identity,
                        module_name.ok_or("missing retained module name")?,
                    )
                } else {
                    neoclr::native_metadata::assembly_name(source, identity)
                }
                .map_err(|e| e.to_string())?;
                factory.emit(&value, &expected)?;
                factory.body.push(Op::Return);
                for skip in skips {
                    factory.body[skip] = Op::BranchFalse(factory.body.len());
                }
            }
            factory.body.push(Op::Fault(
                "native assembly ownership metadata was not retained".into(),
            ));
        } else if declaring {
            let Type::Constructed { arguments, .. } = &expected else {
                return Err("TypeDeclaringType requires scoped Option<TypeInfo>".into());
            };
            if !matches!(
                name(source, &expected),
                Some("System.Option" | "System.Option`1")
            ) || arguments.len() != 1
                || name(source, &arguments[0]) != Some("System.Introspection.TypeInfo")
            {
                return Err("TypeDeclaringType requires scoped Option<TypeInfo>".into());
            }
            for row in retention["types"].as_array().unwrap() {
                let id: TypeDefId = serde_json::from_value(row["definition"].clone())?;
                let ti = definitions
                    .iter()
                    .position(|definition| *definition == id)
                    .ok_or("missing declaring-type snapshot root")?;
                let owner = Type::Named(source.types[ti].name.clone());
                let token = if source.types[ti].generic_parameters.is_empty() {
                    Op::LoadTypeToken(owner.clone())
                } else {
                    neoclr::native_metadata::retain_definition_token(
                        factory.input,
                        source,
                        &id,
                        None,
                    )
                    .map_err(|e| e.to_string())?
                };
                let branch = factory.body.len() + 3;
                factory
                    .body
                    .extend([Op::Arg(0), token, Op::Equal, Op::BranchFalse(0)]);
                let recipe = neoclr::native_metadata::declaring_type(source, &owner)
                    .map_err(|error| error.to_string())?;
                factory.emit(&recipe, &expected)?;
                factory.body.push(Op::Return);
                factory.body[branch] = Op::BranchFalse(factory.body.len());
            }
            factory.body.push(Op::Fault(
                "native declaring-type metadata was not retained; configure --reflection-roots"
                    .into(),
            ));
        } else if base_type {
            let Type::Constructed { arguments, .. } = &expected else {
                return Err("TypeBaseType requires scoped Option<TypeInfo>".into());
            };
            if !matches!(
                name(source, &expected),
                Some("System.Option" | "System.Option`1")
            ) || arguments.len() != 1
                || name(source, &arguments[0]) != Some("System.Introspection.TypeInfo")
            {
                return Err("TypeBaseType requires scoped Option<TypeInfo>".into());
            }
            for t in &base_tokens {
                let branch = factory.body.len() + 3;
                factory.body.extend([
                    Op::Arg(0),
                    Op::LoadTypeToken(t.clone()),
                    Op::Equal,
                    Op::BranchFalse(0),
                ]);
                factory.emit(
                    &neoclr::native_metadata::base_type(source, t).map_err(|e| e.to_string())?,
                    &expected,
                )?;
                factory.body.push(Op::Return);
                factory.body[branch] = Op::BranchFalse(factory.body.len());
            }
            factory.body.push(Op::Fault(
                "native base-type query requires a retained type token".into(),
            ));
        } else if modules {
            if name(source, &expected) != Some("System.Introspection.ModuleInfo") {
                return Err("TypeModule requires scoped ModuleInfo".into());
            }
            for row in retention["types"].as_array().unwrap() {
                let id: TypeDefId = serde_json::from_value(row["definition"].clone())?;
                let ti = definitions
                    .iter()
                    .position(|definition| *definition == id)
                    .ok_or("missing module snapshot root")?;
                let owner = Type::Named(source.types[ti].name.clone());
                let token = if source.types[ti].generic_parameters.is_empty() {
                    Op::LoadTypeToken(owner.clone())
                } else {
                    neoclr::native_metadata::retain_definition_token(
                        factory.input,
                        source,
                        &id,
                        None,
                    )
                    .map_err(|e| e.to_string())?
                };
                let skip = factory.body.len() + 3;
                factory
                    .body
                    .extend([Op::Arg(0), token, Op::Equal, Op::BranchFalse(0)]);
                factory.emit(
                    &neoclr::native_metadata::type_module(source, &owner)
                        .map_err(|e| e.to_string())?,
                    &expected,
                )?;
                factory.body.push(Op::Return);
                factory.body[skip] = Op::BranchFalse(factory.body.len());
            }
            factory.body.push(Op::Fault(
                "native logical module metadata was not retained".into(),
            ));
        } else if methods {
            let Type::ArrayRef(element) = &expected else {
                return Err("TypeMethods requires MethodInfo vector".into());
            };
            if name(source, element) != Some("System.Introspection.MethodInfo") {
                return Err("TypeMethods requires scoped MethodInfo".into());
            }
            factory.body.extend([
                Op::Arg(1),
                Op::Int(!62),
                Op::BitAnd,
                Op::Int(0),
                Op::Equal,
                Op::BranchTrue(7),
                Op::Fault("unsupported BindingFlags bits".into()),
            ]);
            for entry in retention["methodTypes"].as_array().into_iter().flatten() {
                let id: TypeDefId = serde_json::from_value(entry.clone())?;
                if !definitions.contains(&id) {
                    return Err("missing method snapshot root".into());
                }
                let open_owner = neoclr::native_metadata::definition_owner(source, &id)
                    .map_err(|e| e.to_string())?;
                let mut owners = vec![open_owner.clone()];
                for token in &method_tokens {
                    if source
                        .type_definition(token)
                        .and_then(|d| d.definition.as_ref())
                        == Some(&id)
                        && !owners.contains(token)
                    {
                        owners.push(token.clone());
                    }
                }
                let masks: Vec<_> = source
                    .functions
                    .iter()
                    .filter(|f| {
                        f.owner.as_ref() == Some(&open_owner)
                            && !f.name.ends_with("..ctor")
                            && !f.name.ends_with("..cctor")
                    })
                    .map(|f| {
                        (
                            if f.instance { 4 } else { 8 },
                            if f.origin
                                .as_ref()
                                .map_or(f.visibility == Visibility::Public, |o| {
                                    o.member_access
                                        == Some(neoclr::metadata_origin::SourceAccess::Public)
                                })
                            {
                                16
                            } else {
                                32
                            },
                        )
                    })
                    .collect();
                for owner in owners {
                    let (descriptor, recipe) = neoclr::native_metadata::methods(source, &owner)
                        .map_err(|e| e.to_string())?;
                    let V::Array { elements, .. } = recipe else {
                        return Err("invalid method snapshot recipe".into());
                    };
                    let token =
                        neoclr::native_metadata::snapshot_token(factory.input, source, &descriptor)
                            .map_err(|e| e.to_string())?;
                    let branch = factory.body.len() + 3;
                    factory
                        .body
                        .extend([Op::Arg(0), token, Op::Equal, Op::BranchFalse(0)]);
                    emit_filtered_members(&mut factory, &expected, element, &elements, &masks)?;
                    factory.body[branch] = Op::BranchFalse(factory.body.len());
                }
            }
            factory.body.push(Op::Fault(
                "native method metadata was not retained; configure methodTypes".into(),
            ));
        } else if properties || members {
            let Type::ArrayRef(element) = &expected else {
                return Err("TypeProperties requires PropertyInfo vector".into());
            };
            if name(source, element)
                != Some(if fields {
                    "System.Introspection.FieldInfo"
                } else if constructors {
                    "System.Introspection.ConstructorInfo"
                } else {
                    "System.Introspection.PropertyInfo"
                })
            {
                return Err("TypeProperties requires scoped PropertyInfo".into());
            }
            factory.body.extend([
                Op::Arg(1),
                Op::Int(!62),
                Op::BitAnd,
                Op::Int(0),
                Op::Equal,
                Op::BranchTrue(7),
                Op::Fault("unsupported BindingFlags bits".into()),
            ]);
            for row in retention["types"].as_array().unwrap() {
                if row[if fields {
                    "fields"
                } else if constructors {
                    "constructors"
                } else {
                    "properties"
                }] != true
                {
                    continue;
                }
                let id: TypeDefId = serde_json::from_value(row["definition"].clone())?;
                let ti = definitions
                    .iter()
                    .position(|d| *d == id)
                    .ok_or("missing snapshot root")?;
                let owners: Vec<Type> = if constructors && row.get("constructorOwners").is_some() {
                    serde_json::from_value(row["constructorOwners"].clone())?
                } else {
                    vec![Type::Named(source.types[ti].name.clone())]
                };
                for owner in owners {
                    let V::Array { elements, .. } = (if members {
                        neoclr::native_metadata::members(source, &owner, constructors)
                    } else {
                        neoclr::native_metadata::properties(source, &owner)
                    })
                    .map_err(|e| e.to_string())?
                    else {
                        return Err("invalid property recipe".into());
                    };
                    let branch = factory.body.len() + 3;
                    factory.body.extend([
                        Op::Arg(0),
                        Op::LoadTypeToken(owner),
                        Op::Equal,
                        Op::BranchFalse(0),
                    ]);
                    let mut masks = vec![];
                    if fields {
                        let d = &source.types[ti];
                        masks.extend(d.fields.iter().enumerate().map(|(i, _)| {
                            (
                                4,
                                if d.origin.as_ref().map_or(
                                    d.fields[i].visibility == Visibility::Public,
                                    |o| {
                                        o.field_access.get(i)
                                            == Some(&neoclr::metadata_origin::SourceAccess::Public)
                                    },
                                ) {
                                    16
                                } else {
                                    32
                                },
                            )
                        }));
                        masks.extend(d.static_fields.iter().map(|f| {
                            (
                                8,
                                if f.visibility == Visibility::Public {
                                    16
                                } else {
                                    32
                                },
                            )
                        }));
                        masks.extend(d.literal_fields.iter().map(|f| {
                            (
                                8,
                                if f.visibility == Visibility::Public {
                                    16
                                } else {
                                    32
                                },
                            )
                        }));
                    } else if constructors {
                        masks.extend(
                            source
                                .functions
                                .iter()
                                .filter(|f| {
                                    f.owner.as_ref().and_then(Type::definition_name)
                                        == Some(source.types[ti].name.as_str())
                                        && (f.name.ends_with("..ctor")
                                            || f.name.ends_with("..cctor"))
                                })
                                .map(|f| {
                                    (
                                        if f.instance { 4 } else { 8 },
                                        if f.origin.as_ref().map_or(
                                            f.visibility == Visibility::Public,
                                            |o| {
                                                o.member_access
                                                == Some(
                                                    neoclr::metadata_origin::SourceAccess::Public,
                                                )
                                            },
                                        ) {
                                            16
                                        } else {
                                            32
                                        },
                                    )
                                }),
                        );
                    } else {
                        for p in &source.types[ti].properties {
                            let public = p.getter.iter().chain(&p.setter).any(|r| {
                                source
                                    .functions
                                    .iter()
                                    .find(|f| {
                                        if r.definition.is_some() {
                                            f.definition == r.definition
                                        } else {
                                            f.name == r.name
                                                && f.owner == r.owner
                                                && f.parameters == r.parameters
                                        }
                                    })
                                    .is_some_and(|f| {
                                        f.origin.as_ref().map_or(
                                            f.visibility == Visibility::Public,
                                            |o| {
                                                o.member_access
                                            == Some(neoclr::metadata_origin::SourceAccess::Public)
                                            },
                                        )
                                    })
                            });
                            masks.push((
                                if p.instance { 4 } else { 8 },
                                if public { 16 } else { 32 },
                            ));
                        }
                    }
                    emit_filtered_members(&mut factory, &expected, element, &elements, &masks)?;
                    factory.body[branch] = Op::BranchFalse(factory.body.len());
                }
            }
            factory.body.push(Op::Fault("native reflection property metadata was not retained; configure --reflection-roots".into()));
        } else {
            if !matches!(
                name(source, &expected),
                Some("System.Option" | "System.Option`1")
            ) {
                return Err("TypeElementType requires scoped Option".into());
            }
            let mut seen = vec![];
            for t in &tokens {
                if seen.contains(t) {
                    continue;
                }
                seen.push(t.clone());
                let branch = factory.body.len() + 3;
                factory.body.extend([
                    Op::Arg(0),
                    Op::LoadTypeToken(t.clone()),
                    Op::Equal,
                    Op::BranchFalse(0),
                ]);
                factory.emit(
                    &neoclr::native_metadata::element_type(source, t).map_err(|e| e.to_string())?,
                    &expected,
                )?;
                factory.body.push(Op::Return);
                factory.body[branch] = Op::BranchFalse(factory.body.len());
            }
            let f = factory
                .input
                .functions
                .iter()
                .find(|f| f.name == "neoCLR.Runtime.TypeShape")
                .ok_or("element query requires TypeShape")?;
            let call = FunctionRef {
                definition: f.definition.clone(),
                name: f.name.clone(),
                owner: None,
                instance: false,
                parameters: f.parameters.clone(),
                generic_arguments: vec![],
            };
            let skip = factory.body.len() + 4;
            factory.body.extend([
                Op::Arg(0),
                Op::Int(0),
                Op::Call(call),
                Op::BranchFalse(skip + 1),
                Op::Fault("native reflection element metadata was not retained".into()),
            ]);
            factory.emit(
                &V::Object {
                    ty: Type::from_name("$ReflectionSnapshot.Option"),
                    fields: vec![V::Boolean(false)],
                },
                &expected,
            )?;
            factory.body.push(Op::Return);
        }
        projections.extend(factory.trusted);
        let f = &mut factory.input.functions[index];
        f.impl_flags = 0;
        f.body = factory.body;
        f.locals = factory.locals;
        f.local_names.clear();
    }
    let mut caller_rows = vec![];
    for query in context_queries {
        let expected = input.functions[query.getter].returns.clone();
        let mut factory = Factory {
            input,
            source,
            body: vec![],
            locals: vec![],
            trusted: vec![],
        };
        for (identity, recipe) in &query.assemblies {
            let skip = factory.body.len() + 3;
            factory.body.extend([
                Op::Arg(1),
                Op::String(identity.clone()),
                Op::Equal,
                Op::BranchFalse(0),
            ]);
            factory.emit(recipe, &expected)?;
            factory.body.push(Op::Return);
            factory.body[skip] = Op::BranchFalse(factory.body.len());
        }
        factory.body.push(Op::Fault(
            "native executing assembly caller was not admitted".into(),
        ));
        projections.extend(factory.trusted);
        let getter = &mut factory.input.functions[query.getter];
        getter.body = factory.body;
        getter.locals = factory.locals;
        getter.local_names.clear();
        getter.sequence_points.clear();
        caller_rows.push(json!({"getter":getter.definition,"assemblies":query.assemblies.iter().map(|(identity, _)| identity).collect::<Vec<_>>()}));
    }
    retention["executingAssemblyQueries"] = json!(caller_rows);
    retention["snapshotFactories"] = json!(projections);
    Ok(())
}
fn predicate(body: &mut Vec<Op>, scope: i32, access: i32) {
    body.extend([
        Op::Arg(1),
        Op::Int(scope | access),
        Op::BitAnd,
        Op::Int(scope | access),
        Op::Equal,
    ]);
}

// The source catalogue supplies recipes; only descriptor factories become executable.
// Keeping an attribute never grants invocation rights to its constructor or target.
fn bind_attributes(
    factory: &mut Factory<'_>,
    service: usize,
    definitions: &[TypeDefId],
    retention: &Value,
    expected: &Type,
) -> Result<(), Error> {
    let Type::ArrayRef(element) = expected else {
        return Err("MemberCustomAttributes requires CustomAttributeData vector".into());
    };
    if name(factory.source, element) != Some("System.Introspection.CustomAttributeData") {
        return Err("MemberCustomAttributes requires scoped CustomAttributeData".into());
    }
    let contract = factory.input.functions[service].parameters[0].clone();
    let origin = factory
        .source
        .type_definition(&contract)
        .and_then(|d| d.origin.as_ref())
        .ok_or("missing TypeInfo source identity")?;
    let providers: Vec<_> = factory
        .source
        .types
        .iter()
        .filter(|d| {
            d.origin.as_ref().is_some_and(|o| {
                o.assembly == origin.assembly
                    && o.module == origin.module
                    && o.name == "System.Introspection.RuntimeNominalTypeInfo"
            })
        })
        .collect();
    let [provider] = providers.as_slice() else {
        return Err("missing unique RuntimeNominalTypeInfo provider".into());
    };
    let owner = Type::Named(provider.name.clone());
    let getters: Vec<_> = factory
        .input
        .functions
        .iter()
        .filter(|f| {
            f.owner.as_ref() == Some(&owner)
                && f.instance
                && f.parameters.is_empty()
                && f.returns == Type::RuntimeTypeHandle
                && f.origin
                    .as_ref()
                    .is_some_and(|o| o.name == "get_ExecutionHandle")
        })
        .collect();
    let [getter] = getters.as_slice() else {
        return Err("missing unique nominal type handle accessor".into());
    };
    let call = FunctionRef {
        definition: getter.definition.clone(),
        name: getter.name.clone(),
        owner: Some(owner.clone()),
        instance: true,
        parameters: vec![],
        generic_arguments: vec![],
    };
    let handle = factory.local(Type::RuntimeTypeHandle);
    factory.body.extend([
        Op::Arg(0),
        Op::CastClass(owner),
        Op::Call(call),
        Op::Store(handle),
        Op::Arg(1),
        Op::Int(0),
        Op::Less,
        Op::BranchFalse(9),
        Op::Fault("invalid attribute target".into()),
    ]);
    for row in retention["types"]
        .as_array()
        .ok_or("missing reflection root types")?
    {
        if row["customAttributes"] != true {
            continue;
        }
        let id: TypeDefId = serde_json::from_value(row["definition"].clone())?;
        let ti = definitions
            .iter()
            .position(|d| *d == id)
            .ok_or("missing attribute root")?;
        let declaration = &factory.source.types[ti];
        let origin = declaration
            .origin
            .as_ref()
            .ok_or("attribute root requires source metadata")?;
        let owner = Type::Named(declaration.name.clone());
        let branch = factory.body.len() + 3;
        factory.body.extend([
            Op::Load(handle),
            Op::LoadTypeToken(owner.clone()),
            Op::Equal,
            Op::BranchFalse(0),
        ]);
        let mut tokens: Vec<u32> = declaration
            .custom_attributes
            .iter()
            .map(|a| a.target_token.unwrap_or(origin.token))
            .collect();
        // Match interpreter lookup, including callable and parameter tokens in this module.
        for f in &factory.source.functions {
            if let Some(o) = &f.origin
                && o.assembly == origin.assembly
                && o.module == origin.module
            {
                tokens.extend(
                    f.custom_attributes
                        .iter()
                        .map(|a| a.target_token.unwrap_or(o.token)),
                );
            }
        }
        tokens.sort_unstable();
        tokens.dedup();
        for token in tokens {
            let token: i32 = token.try_into()?;
            if token == 0 {
                continue;
            }
            let skip = factory.body.len() + 3;
            factory
                .body
                .extend([Op::Arg(1), Op::Int(token), Op::Equal, Op::BranchFalse(0)]);
            let recipe = neoclr::native_metadata::custom_attributes(factory.source, &owner, token)
                .map_err(|e| e.to_string())?;
            factory.emit(&recipe, expected)?;
            factory.body.push(Op::Return);
            factory.body[skip] = Op::BranchFalse(factory.body.len());
        }
        // Valid retained owner, no annotation for this target (including token zero).
        factory.body.extend([
            Op::Int(0),
            Op::ReserveArray((**element).clone()),
            Op::Return,
        ]);
        factory.body[branch] = Op::BranchFalse(factory.body.len());
    }
    factory.body.push(Op::Fault(
        "native custom attribute metadata was not retained; configure --reflection-roots".into(),
    ));
    Ok(())
}

#[cfg(test)]
mod catalog_tests {
    use super::*;

    #[test]
    fn method_retention_projects_open_and_observed_closed_owners_without_invocation() {
        let mut source = neoclr::assemble(".module Methods\n.type Empty<T>\n.end\n.type Atom\n.end\n.function neoCLR.Runtime.TypeMethods(RuntimeTypeHandle,Int32) -> System.Introspection.MethodInfo[]\n.methodimpl InternalCall\n.end\n.function Main() -> RuntimeTypeHandle\nldtoken Empty<Atom>\nret\n.end\n").unwrap();
        // Assembly parsing uses the legacy core service shape; the native source
        // library supplies the vector signature verified by this binder.
        source.functions[0].returns =
            Type::ArrayRef(Box::new(Type::from_name("System.Introspection.MethodInfo")));
        let ids: Vec<_> = source
            .types
            .iter()
            .map(|d| d.definition.clone().unwrap())
            .collect();
        let mut input = source.clone();
        let mut policy = json!({"types":[],"methodTypes":[ids[0]]});
        bind(&mut input, &source, &ids, &mut policy).unwrap();
        let body = &input.functions[0].body;
        assert!(body.iter().any(|op| matches!(op, Op::LoadMetadataToken(_))));
        assert!(
            body.iter()
                .any(|op| matches!(op, Op::LoadTypeToken(Type::Constructed { .. })))
        );
        assert!(
            !body
                .iter()
                .any(|op| matches!(op, Op::Call(_) | Op::Construct(_)))
        );
        assert!(
            matches!(body.last(), Some(Op::Fault(message)) if message.contains("not retained"))
        );
        let mut unretained = source.clone();
        bind(&mut unretained, &source, &ids, &mut json!({"types":[]})).unwrap();
        assert!(
            !unretained.functions[0]
                .body
                .iter()
                .any(|op| matches!(op, Op::ReserveArray(_)))
        );
    }

    #[test]
    fn type_discovery_requires_retention_and_exact_contracts() {
        let mut source = neoclr::assemble(".module Catalog\n.function neoCLR.Runtime.AssemblyTypes(String) -> arrayref<System.Introspection.TypeInfo>\n.methodimpl InternalCall\n.end\n.function neoCLR.Runtime.ModuleTypes(String,String) -> arrayref<System.Introspection.TypeInfo>\n.methodimpl InternalCall\n.end\n").unwrap();
        source.assemblies = serde_json::from_value(json!([
            {"name":"One","full_name":"One","modules":["Catalog"],"references":[],
             "declaration_modules":{"version":1,"names":["Empty"]}}
        ]))
        .unwrap();
        let mut retention = json!({"types":[],"typeCatalogs":["One"]});
        let mut input = source.clone();
        bind(&mut input, &source, &[], &mut retention).unwrap();
        for f in &input.functions {
            assert!(f.body.iter().any(|op| matches!(op, Op::ReserveArray(_))));
            assert!(
                matches!(f.body.last(), Some(Op::Fault(message)) if message.contains("type catalog was not retained"))
            );
        }
        let mut unretained = source.clone();
        bind(
            &mut unretained,
            &source,
            &[],
            &mut json!({"types":[],"moduleCatalogs":["One"]}),
        )
        .unwrap();
        for f in &unretained.functions {
            assert!(matches!(f.body.as_slice(), [Op::Fault(_)]));
        }
        let mut invalid = source.clone();
        invalid.functions[1].parameters[1] = Type::Int32;
        assert!(bind(&mut invalid, &source, &[], &mut retention).is_err());
        let mut invalid = source.clone();
        invalid.functions[0].returns = Type::ArrayRef(Box::new(Type::Int32));
        assert!(bind(&mut invalid, &source, &[], &mut retention).is_err());
    }

    #[test]
    fn snapshot_tokens_keep_closed_identity_and_open_parameter_owners() {
        let source = neoclr::assemble(
            ".module Catalog\n.interface Producer<T>\n.variance out\n.end\n.type Plain\n.end\n",
        )
        .unwrap();
        let mut target = source.clone();
        let id = source.types[0].definition.as_ref().unwrap();
        let Op::LoadMetadataToken(index) =
            neoclr::native_metadata::retain_definition_token(&mut target, &source, id, None)
                .unwrap()
        else {
            panic!("open token")
        };
        let open = neoclr::native_metadata::descriptor(&target, index)
            .unwrap()
            .clone();
        let mut factory = Factory {
            input: &mut target,
            source: &source,
            body: vec![],
            locals: vec![],
            trusted: vec![],
        };
        factory
            .emit(
                &V::RuntimeTypeHandle(Box::new(open.clone())),
                &Type::RuntimeTypeHandle,
            )
            .unwrap();
        factory
            .emit(
                &V::RuntimeTypeHandle(Box::new(open.generic_arguments[0].clone())),
                &Type::RuntimeTypeHandle,
            )
            .unwrap();
        assert!(matches!(factory.body[0], Op::LoadMetadataToken(i) if i == index));
        let Op::LoadMetadataToken(parameter) = factory.body[1] else {
            panic!("parameter token")
        };
        assert_eq!(
            neoclr::native_metadata::descriptor(factory.input, parameter).unwrap(),
            &open.generic_arguments[0]
        );
        // The source descriptor of a nongeneric type must still use ordinary ldtoken.
        let mut catalog = source.clone();
        catalog.assemblies = serde_json::from_value(json!([{"name":"One","full_name":"One","modules":["Catalog"],"references":[],"declaration_modules":{"version":1,"names":[""]}}])).unwrap();
        let V::Array { elements, .. } =
            neoclr::native_metadata::assembly_types(&catalog, "One").unwrap()
        else {
            panic!("catalog")
        };
        let V::Object { fields, .. } = &elements[1] else {
            panic!("snapshot")
        };
        factory.emit(&fields[0], &Type::RuntimeTypeHandle).unwrap();
        assert!(
            matches!(factory.body.last(), Some(Op::LoadTypeToken(Type::Named(name))) if name == "Plain")
        );
    }

    #[test]
    fn empty_function_scope_is_explicit_and_preserves_unretained_faults() {
        let mut source = neoclr::assemble(".module Functions\n.function neoCLR.Runtime.ModuleFunctions(String,String) -> arrayref<System.Introspection.MethodInfo>\n.methodimpl InternalCall\n.end\n").unwrap();
        source.assemblies = serde_json::from_value(json!([
            {"name":"One","full_name":"One","modules":["Functions"],"references":[],
             "declaration_modules":{"version":1,"names":["Empty"]}}
        ]))
        .unwrap();
        let mut retention = json!({"types":[],"moduleFunctions":[{"assembly":"One","name":"Empty","customAttributes":false}]});
        let mut input = source.clone();
        bind(&mut input, &source, &[], &mut retention).unwrap();
        assert!(
            input.functions[0]
                .body
                .iter()
                .any(|op| matches!(op, Op::ReserveArray(_)))
        );
        assert!(
            matches!(input.functions[0].body.last(), Some(Op::Fault(message)) if message.contains("module function metadata was not retained"))
        );
        assert_eq!(
            input.functions[0]
                .body
                .iter()
                .filter(|op| matches!(op, Op::BranchFalse(_)))
                .count(),
            2
        );
        let mut invalid = source.clone();
        invalid.functions[0].parameters[1] = Type::Int32;
        assert!(
            bind(&mut invalid, &source, &[], &mut retention)
                .unwrap_err()
                .to_string()
                .contains("exact reserved InternalCall contract")
        );
        let mut invalid = source.clone();
        invalid.functions[0].returns = Type::ArrayRef(Box::new(Type::Int32));
        assert!(
            bind(&mut invalid, &source, &[], &mut retention)
                .unwrap_err()
                .to_string()
                .contains("scoped System.Introspection.MethodInfo")
        );
    }

    #[test]
    fn declaring_type_requires_exact_contract_and_explicit_retention() {
        let source = neoclr::assemble(".module Declaring\n.function neoCLR.Runtime.TypeDeclaringType(RuntimeTypeHandle) -> System.Option<System.Introspection.TypeInfo>\n.methodimpl InternalCall\n.end\n").unwrap();
        let mut retention = json!({"types":[]});
        let mut input = source.clone();
        bind(&mut input, &source, &[], &mut retention).unwrap();
        assert!(
            matches!(&input.functions[0].body[..], [Op::Fault(message)] if message.contains("declaring-type metadata was not retained"))
        );
        let mut invalid = source.clone();
        invalid.functions[0].parameters[0] = Type::Int32;
        assert!(
            bind(&mut invalid, &source, &[], &mut retention)
                .unwrap_err()
                .to_string()
                .contains("exact reserved InternalCall contract")
        );
        let mut invalid = source.clone();
        invalid.functions[0].returns =
            neoclr::assembler::parse_type("System.Option<Int32>").unwrap();
        assert!(
            bind(&mut invalid, &source, &[], &mut retention)
                .unwrap_err()
                .to_string()
                .contains("Option<TypeInfo>")
        );
    }

    #[test]
    fn unretained_module_attributes_bind_an_explicit_fault_with_exact_contract() {
        let source = neoclr::assemble(".module Attributes\n.type class System.Introspection.CustomAttributeData\n.end\n.function neoCLR.Runtime.ModuleMemberCustomAttributes(String,String,Int32) -> arrayref<System.Introspection.CustomAttributeData>\n.methodimpl InternalCall\n.end\n").unwrap();
        let mut retention = json!({"types":[]});
        let mut input = source.clone();
        bind(&mut input, &source, &[], &mut retention).unwrap();
        assert_eq!(input.functions[0].impl_flags, 0);
        assert!(
            matches!(&input.functions[0].body[..], [Op::Fault(message)] if message.contains("module-member custom attribute metadata was not retained"))
        );
        assert_eq!(retention["snapshotFactories"], json!([]));
        let mut invalid = source.clone();
        invalid.functions[0].parameters[2] = Type::String;
        assert!(
            bind(&mut invalid, &source, &[], &mut retention)
                .unwrap_err()
                .to_string()
                .contains("exact reserved InternalCall contract")
        );
        let mut invalid = source.clone();
        invalid.functions[0].returns = Type::ArrayRef(Box::new(Type::Int32));
        assert!(
            bind(&mut invalid, &source, &[], &mut retention)
                .unwrap_err()
                .to_string()
                .contains("scoped System.Introspection.CustomAttributeData")
        );
    }

    #[test]
    fn unrelated_type_retention_does_not_require_assembly_catalog_metadata() {
        let source = neoclr::assemble(".module Legacy\n.type class Model\n.end\n").unwrap();
        let definition = source.types[0].definition.clone().unwrap();
        let mut retention = json!({"types":[{"definition":definition,"construct":false}]});
        bind(&mut source.clone(), &source, &[definition], &mut retention).unwrap();
        assert_eq!(retention["snapshotFactories"], json!([]));
    }
}

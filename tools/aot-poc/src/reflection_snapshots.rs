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
            V::RuntimeTypeHandle(h) if *expected == Type::RuntimeTypeHandle => {
                self.body.push(Op::LoadTypeToken(
                    neoclr::native_metadata::handle_type(self.source, h)
                        .map_err(|e| e.to_string())?,
                ))
            }
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
                let ctor = self.constructor(expected, &[case.clone()])?;
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
                    let ctor = self.constructor(expected, &[field.ty.clone()])?;
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
                        Op::LoadTypeToken(
                            neoclr::native_metadata::handle_type(self.source, handle)
                                .map_err(|e| e.to_string())?,
                        ),
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
                        | "System.Introspection.ConstructorInfo" => (true, None),
                        "System.Introspection.CustomAttributeData"
                        | "System.Introspection.CustomAttributeNamedArgument"
                        | "System.Introspection.CustomAttributeTypedArgument" => (false, None),
                        "System.Introspection.MethodInfo" => (true, Some("StoredIdentity")),
                        "System.Introspection.ParameterInfo" => {
                            (false, Some("StoredMemberIdentity"))
                        }
                        "System.Introspection.ModuleInfo"
                        | "System.Introspection.AssemblyInfo" => (false, None),
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
                    self.body.push(Op::CastClass(expected.clone()));
                }
            }
            _ => {
                return Err(
                    format!("unsupported native snapshot value {value:?} for {expected:?}").into(),
                );
            }
        }
        Ok(())
    }
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
                "neoCLR.Runtime.TypeProperties"
                    | "neoCLR.Runtime.TypeModule"
                    | "neoCLR.Runtime.ModuleAssembly"
                    | "neoCLR.Runtime.AssemblyName"
                    | "neoCLR.Runtime.AssemblyModules"
                    | "neoCLR.Runtime.TypeElementType"
                    | "neoCLR.Runtime.MemberCustomAttributes"
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
                for argument in arguments { vectors(argument, tokens); }
            }
            _ => {}
        }
    }
    let mut tokens = vec![];
    for op in input.functions.iter().flat_map(|f| &f.body) {
        match op {
            Op::LoadTypeToken(t) => vectors(t, &mut tokens),
            Op::Call(target) | Op::CallVirtual(target) | Op::Construct(target)
            | Op::BindFunction { target, .. } => {
                for argument in &target.generic_arguments { vectors(argument, &mut tokens); }
            }
            _ => {}
        }
    }
    for row in retention["types"].as_array().unwrap() {
        if row["properties"] != true { continue; }
        let id: TypeDefId = serde_json::from_value(row["definition"].clone())?;
        let ti = definitions.iter().position(|d| *d == id).ok_or("missing snapshot root")?;
        for property in &source.types[ti].properties { vectors(&property.ty, &mut tokens); }
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
    if services.iter().any(|index| matches!(input.functions[*index].name.as_str(),
        "neoCLR.Runtime.ModuleAssembly" | "neoCLR.Runtime.AssemblyName"))
    {
        for row in retention["types"].as_array().unwrap() {
            let id: TypeDefId = serde_json::from_value(row["definition"].clone())?;
            let ti = definitions.iter().position(|definition| *definition == id)
                .ok_or("missing module snapshot root")?;
            retained_modules.push(neoclr::native_metadata::type_module(
                source, &Type::Named(source.types[ti].name.clone()),
            ).map_err(|error| error.to_string())?);
        }
    }
    let mut projections = vec![];
    for index in services {
        let f = &input.functions[index];
        let modules = f.name.ends_with("TypeModule");
        let module_assembly = f.name.ends_with("ModuleAssembly");
        let assembly_name = f.name.ends_with("AssemblyName");
        let assembly_modules = f.name.ends_with("AssemblyModules");
        let properties = f.name.ends_with("Properties");
        let attributes = f.name.ends_with("CustomAttributes");
        let attribute_parameters = attributes
            && f.parameters.len() == 2
            && name(source, &f.parameters[0]) == Some("System.Introspection.TypeInfo")
            && f.parameters[1] == Type::Int32;
        if f.instance
            || f.owner.is_some()
            || (attributes && !attribute_parameters)
            || (!attributes
                && f.parameters
                    != if module_assembly {
                        vec![Type::String, Type::String]
                    } else if assembly_name || assembly_modules {
                        vec![Type::String]
                    } else if properties {
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
        if attributes {
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
                    Op::Arg(0), Op::String((*identity).into()), Op::Equal, Op::BranchFalse(0),
                ]);
                factory.emit(recipe, &expected)?;
                factory.body.push(Op::Return);
                factory.body[skip] = Op::BranchFalse(factory.body.len());
            }
            factory.body.push(Op::Fault("native assembly module catalog was not retained".into()));
        } else if module_assembly || assembly_name {
            if module_assembly && name(source, &expected) != Some("System.Introspection.AssemblyInfo")
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
                owners.extend(context_queries.iter().flat_map(|query| query.assemblies.iter())
                    .map(|(identity, _)| (identity.as_str(), None)));
            }
            for (identity, module_name) in owners {
                let key = (identity, if module_assembly { module_name } else { None });
                if !seen.insert(key) {
                    continue;
                }
                let mut skips = vec![];
                let arguments = std::iter::once(identity)
                    .chain(module_name.filter(|_| module_assembly));
                for (argument, value) in arguments.enumerate()
                {
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
                        source, identity, module_name.ok_or("missing retained module name")?,
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
        } else if modules {
            if name(source, &expected) != Some("System.Introspection.ModuleInfo") {
                return Err("TypeModule requires scoped ModuleInfo".into());
            }
            for row in retention["types"].as_array().unwrap() {
                let id: TypeDefId = serde_json::from_value(row["definition"].clone())?;
                let ti = definitions.iter().position(|definition| *definition == id)
                    .ok_or("missing module snapshot root")?;
                let owner = Type::Named(source.types[ti].name.clone());
                let skip = factory.body.len() + 3;
                factory.body.extend([Op::Arg(0), Op::LoadTypeToken(owner.clone()), Op::Equal, Op::BranchFalse(0)]);
                factory.emit(&neoclr::native_metadata::type_module(source, &owner).map_err(|e| e.to_string())?, &expected)?;
                factory.body.push(Op::Return);
                factory.body[skip] = Op::BranchFalse(factory.body.len());
            }
            factory.body.push(Op::Fault("native logical module metadata was not retained".into()));
        } else if properties {
            let Type::ArrayRef(element) = &expected else {
                return Err("TypeProperties requires PropertyInfo vector".into());
            };
            if name(source, element) != Some("System.Introspection.PropertyInfo") {
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
                if row["properties"] != true {
                    continue;
                }
                let id: TypeDefId = serde_json::from_value(row["definition"].clone())?;
                let ti = definitions
                    .iter()
                    .position(|d| *d == id)
                    .ok_or("missing snapshot root")?;
                let owner = Type::Named(source.types[ti].name.clone());
                let V::Array { elements, .. } = neoclr::native_metadata::properties(source, &owner)
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
                let array = factory.local(expected.clone());
                let count = factory.local(Type::Int32);
                factory.body.extend([Op::Int(0), Op::Store(count)]);
                let mut masks = vec![];
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
                                f.origin
                                    .as_ref()
                                    .map_or(f.visibility == Visibility::Public, |o| {
                                        o.member_access
                                            == Some(neoclr::metadata_origin::SourceAccess::Public)
                                    })
                            })
                    });
                    masks.push((if p.instance { 4 } else { 8 }, if public { 16 } else { 32 }));
                }
                if masks.len() != elements.len() {
                    return Err("property recipe inventory mismatch".into());
                }
                for &(scope, access) in &masks {
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
                    Op::ReserveArray((**element).clone()),
                    Op::Store(array),
                    Op::Int(0),
                    Op::Store(count),
                ]);
                for (v, &(scope, access)) in elements.iter().zip(&masks) {
                    predicate(&mut factory.body, scope, access);
                    let skip = factory.body.len();
                    factory.body.push(Op::BranchFalse(0));
                    factory.body.extend([Op::Load(array), Op::Load(count)]);
                    factory.emit(v, element)?;
                    factory
                        .body
                        .push(Op::StoreArrayElement((**element).clone()));
                    factory
                        .body
                        .extend([Op::Load(count), Op::Int(1), Op::Add, Op::Store(count)]);
                    factory.body[skip] = Op::BranchFalse(factory.body.len());
                }
                factory.body.extend([Op::Load(array), Op::Return]);
                factory.body[branch] = Op::BranchFalse(factory.body.len());
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
        let mut factory = Factory { input, source, body: vec![], locals: vec![], trusted: vec![] };
        for (identity, recipe) in &query.assemblies {
            let skip = factory.body.len() + 3;
            factory.body.extend([Op::Arg(1), Op::String(identity.clone()), Op::Equal, Op::BranchFalse(0)]);
            factory.emit(recipe, &expected)?;
            factory.body.push(Op::Return);
            factory.body[skip] = Op::BranchFalse(factory.body.len());
        }
        factory.body.push(Op::Fault("native executing assembly caller was not admitted".into()));
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
            if let Some(o) = &f.origin {
                if o.assembly == origin.assembly && o.module == origin.module {
                    tokens.extend(
                        f.custom_attributes
                            .iter()
                            .map(|a| a.target_token.unwrap_or(o.token)),
                    );
                }
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
    fn unrelated_type_retention_does_not_require_assembly_catalog_metadata() {
        let source = neoclr::assemble(".module Legacy\n.type class Model\n.end\n").unwrap();
        let definition = source.types[0].definition.clone().unwrap();
        let mut retention = json!({"types":[{"definition":definition,"construct":false}]});
        bind(&mut source.clone(), &source, &[definition], &mut retention).unwrap();
        assert_eq!(retention["snapshotFactories"], json!([]));
    }
}

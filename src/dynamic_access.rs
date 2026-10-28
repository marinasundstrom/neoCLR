//! Conservative native retention analysis for type-bearing parameter and return contracts.
//! Unknown flows are diagnostics, never evidence that metadata is retained.
use crate::{
    Fault, Module,
    metadata::{AttributeArgument, Function, Instruction as Op, Type},
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

const ATTRIBUTE: &str = "System.Diagnostics.CodeAnalysis.DynamicallyAccessedMembersAttribute";
pub const PUBLIC_CONSTRUCTORS: u32 = 3;
pub const PUBLIC_PROPERTIES: u32 = 512;
const SUPPORTED: u32 = PUBLIC_CONSTRUCTORS | PUBLIC_PROPERTIES;

#[derive(Debug, Default, serde::Serialize)]
pub struct Analysis {
    /// Source type row and the union of proven member requirements.
    pub types: BTreeMap<usize, u32>,
    pub diagnostics: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Value {
    Unknown,
    Types(Vec<Type>),
    Required(u32),
}

fn source_name<'a>(module: &'a Module, ty: &Type) -> Option<&'a str> {
    let declaration = module.type_definition(ty)?;
    Some(
        declaration
            .origin
            .as_ref()
            .map_or(declaration.name.as_str(), |o| o.name.as_str()),
    )
}

fn display_name(module: &Module, function: &Function) -> String {
    match &function.origin {
        Some(origin) => match function.owner.as_ref().and_then(|t| source_name(module, t)) {
            Some(owner) => format!("{owner}.{}", origin.name),
            None => origin.full_name.as_ref().unwrap_or(&origin.name).clone(),
        },
        None => function.name.clone(),
    }
}

#[derive(Default)]
struct Contract {
    parameters: Vec<u32>,
    returns: u32,
    receiver: u32,
    generic: Vec<u32>,
}
impl Contract {
    fn is_empty(&self) -> bool {
        self.receiver == 0
            && self.returns == 0
            && self.parameters.iter().chain(&self.generic).all(|f| *f == 0)
    }
}

fn flags(
    module: &Module,
    attribute: &crate::metadata::CustomAttribute,
) -> Result<Option<u32>, Fault> {
    if attribute
        .constructor
        .owner
        .as_ref()
        .and_then(|t| source_name(module, t))
        != Some(ATTRIBUTE)
    {
        return Ok(None);
    }
    let [AttributeArgument::Int32(value)] = attribute.arguments.as_slice() else {
        return Err(Fault::new(
            "DAM1001: member requirements must be Int32 flags",
        ));
    };
    if *value < 0 || (*value as u32 & !SUPPORTED) != 0 || !attribute.named_arguments.is_empty() {
        return Err(Fault::new("DAM1001: unsupported dynamic member flags"));
    }
    Ok(Some(*value as u32))
}

fn property_flags(
    module: &Module,
    owner: &crate::metadata::TypeDef,
    index: usize,
) -> Result<u32, Fault> {
    let token = owner
        .origin
        .as_ref()
        .and_then(|o| o.property_tokens.get(index));
    let mut result = 0;
    for attribute in &owner.custom_attributes {
        if token.is_some() && attribute.target_token.as_ref() == token {
            if let Some(value) = flags(module, attribute)? {
                result |= value;
            }
        }
    }
    Ok(result)
}

fn field_flags(module: &Module, owner: &Type, index: usize, is_static: bool) -> Result<u32, Fault> {
    // Tuple/function carriers have structural fields but no source declaration
    // on which a field annotation can reside.
    if module.type_definition(owner).is_none() {
        return Ok(0);
    }
    let (owner, index) = if is_static {
        (owner.clone(), index)
    } else {
        crate::inheritance::field_owner(module, owner, index)?
    };
    let definition = module
        .type_definition(&owner)
        .ok_or_else(|| Fault::new("missing field owner"))?;
    let offset = if is_static {
        definition.fields.len()
    } else {
        0
    };
    let token = definition
        .origin
        .as_ref()
        .and_then(|o| o.field_tokens.get(offset + index));
    let mut result = 0;
    for attribute in &definition.custom_attributes {
        if token.is_some() && attribute.target_token.as_ref() == token {
            if let Some(value) = flags(module, attribute)? {
                result |= value;
            }
        }
    }
    // A getter that returns exactly one field establishes its storage contract.
    // Arbitrary computed getters still have their return flow checked normally.
    for (property_index, property) in definition.properties.iter().enumerate() {
        let value = property_flags(module, definition, property_index)?;
        if value == 0 {
            continue;
        }
        let Some(getter) = &property.getter else {
            continue;
        };
        let getter = crate::vm::resolve(module, getter)?;
        let matches = match getter.body.as_slice() {
            [Op::Arg(0), Op::Field(field), Op::Return] if !is_static => {
                let (field_owner, field_index) = crate::inheritance::field_owner(
                    module,
                    getter.owner.as_ref().unwrap(),
                    *field,
                )?;
                module
                    .type_definition(&field_owner)
                    .is_some_and(|t| std::ptr::eq(t, definition))
                    && field_index == index
            }
            [
                Op::StaticField {
                    owner: field_owner,
                    index: field,
                },
                Op::Return,
            ] if is_static => {
                module
                    .type_definition(field_owner)
                    .is_some_and(|t| std::ptr::eq(t, definition))
                    && *field == index
            }
            _ => false,
        };
        if matches {
            result |= value;
        }
    }
    Ok(result)
}

fn validate_storage_contracts(module: &Module) -> Result<bool, Fault> {
    let mut any = false;
    for ty in &module.types {
        for attribute in &ty.custom_attributes {
            let Some(_) = flags(module, attribute)? else {
                continue;
            };
            any = true;
            let fail = || {
                Fault::new(format!(
                    "DAM1001: unsupported dynamic member target on {}",
                    ty.name
                ))
            };
            let origin = ty.origin.as_ref().ok_or_else(fail)?;
            let token = attribute.target_token.ok_or_else(fail)?;
            if origin.generic_parameter_tokens.contains(&token) {
                continue;
            }
            let target = if let Some(index) = origin.field_tokens.iter().position(|t| *t == token) {
                ty.fields
                    .iter()
                    .map(|f| &f.ty)
                    .chain(ty.static_fields.iter().map(|f| &f.ty))
                    .nth(index)
                    .ok_or_else(fail)?
            } else if let Some(index) = origin.property_tokens.iter().position(|t| *t == token) {
                &ty.properties.get(index).ok_or_else(fail)?.ty
            } else {
                return Err(fail());
            };
            if source_name(module, target) != Some("System.Introspection.TypeInfo") {
                return Err(fail());
            }
        }
    }
    Ok(any)
}

fn type_requirements(module: &Module, ty: &crate::metadata::TypeDef) -> Result<Vec<u32>, Fault> {
    let mut result = vec![0; ty.generic_parameters.len()];
    if let Some(origin) = &ty.origin {
        for attribute in &ty.custom_attributes {
            let Some(token) = attribute.target_token else {
                continue;
            };
            if let Some(index) = origin
                .generic_parameter_tokens
                .iter()
                .position(|t| *t == token)
            {
                if let Some(value) = flags(module, attribute)? {
                    *result.get_mut(index).ok_or_else(|| {
                        Fault::new("DAM1001: generic parameter ordinal is outside type")
                    })? |= value;
                }
            }
        }
    }
    Ok(result)
}

fn type_value(module: &Module, function: &Function, ty: &Type) -> Result<Value, Fault> {
    Ok(match ty {
        Type::MethodTypeParameter(index) => Value::Required(
            requirements(module, function)?
                .generic
                .get(*index as usize)
                .copied()
                .unwrap_or(0),
        ),
        Type::TypeParameter(index) => Value::Required(
            function
                .owner
                .as_ref()
                .and_then(|t| module.type_definition(t))
                .map(|t| type_requirements(module, t))
                .transpose()?
                .and_then(|v| v.get(*index as usize).copied())
                .unwrap_or(0),
        ),
        _ => Value::Types(vec![ty.clone()]),
    })
}

fn check_type_arguments(
    module: &Module,
    function: &Function,
    ty: &Type,
    report: &mut Analysis,
) -> Result<(), Fault> {
    if let Type::Constructed { arguments, .. } | Type::Scoped { arguments, .. } = ty {
        if let Some(definition) = module.type_definition(ty) {
            for (index, flags) in type_requirements(module, definition)?
                .into_iter()
                .enumerate()
            {
                if let Some(argument) = arguments.get(index) {
                    require(
                        module,
                        &type_value(module, function, argument)?,
                        flags,
                        &format!(
                            "{} -> {} generic parameter {index}",
                            display_name(module, function),
                            definition.name
                        ),
                        report,
                    );
                }
            }
        }
        for argument in arguments {
            check_type_arguments(module, function, argument, report)?;
        }
    }
    match ty {
        Type::Array(t)
        | Type::ArrayRef(t)
        | Type::ByRef(t)
        | Type::ReadOnlyByRef(t)
        | Type::Ptr(t)
        | Type::InterfaceRef(t) => check_type_arguments(module, function, t, report)?,
        Type::Union(types) | Type::Intersection(types) => {
            for ty in types {
                check_type_arguments(module, function, ty, report)?;
            }
        }
        Type::Tuple(tuple) => {
            for ty in tuple.iter() {
                check_type_arguments(module, function, ty, report)?;
            }
        }
        Type::Function(shape) => {
            for ty in shape.parameters.iter().chain([&shape.returns]) {
                check_type_arguments(module, function, ty, report)?;
            }
        }
        _ => (),
    }
    Ok(())
}

fn check_method_arguments(
    module: &Module,
    function: &Function,
    target: &crate::metadata::FunctionRef,
    callee: &Function,
    report: &mut Analysis,
) -> Result<(), Fault> {
    for (index, flags) in requirements(module, callee)?
        .generic
        .into_iter()
        .enumerate()
    {
        if let Some(argument) = target.generic_arguments.get(index) {
            require(
                module,
                &type_value(module, function, argument)?,
                flags,
                &format!(
                    "{} -> {} generic parameter {index}",
                    display_name(module, function),
                    display_name(module, callee)
                ),
                report,
            );
        }
    }
    Ok(())
}

fn descriptor_receiver(module: &Module, ty: &Type) -> Result<bool, Fault> {
    if source_name(module, ty) == Some("System.Introspection.TypeInfo") {
        return Ok(true);
    }
    Ok(crate::interfaces::closure(module, ty)?
        .iter()
        .any(|interface| source_name(module, interface) == Some("System.Introspection.TypeInfo")))
}

fn requirements(module: &Module, function: &Function) -> Result<Contract, Fault> {
    let mut result = Contract {
        parameters: vec![0; function.parameters.len()],
        generic: vec![0; function.generic_parameters.len()],
        ..Contract::default()
    };
    for attribute in &function.custom_attributes {
        let Some(value) = flags(module, attribute)? else {
            continue;
        };
        let fail = || {
            Fault::new(format!(
                "DAM1001: unsupported DynamicallyAccessedMembers contract on {}",
                function.name
            ))
        };
        let Some(token) = attribute.target_token else {
            if !function.instance
                || function.name.ends_with("..ctor")
                || !function
                    .owner
                    .as_ref()
                    .map(|owner| descriptor_receiver(module, owner))
                    .transpose()?
                    .unwrap_or(false)
            {
                return Err(fail());
            }
            result.receiver |= value;
            continue;
        };
        let origin = function.origin.as_ref().ok_or_else(fail)?;
        if let Some(position) = origin
            .generic_parameter_tokens
            .iter()
            .position(|t| *t == token)
        {
            *result.generic.get_mut(position).ok_or_else(fail)? |= value;
            continue;
        }
        let (target_type, target_flags) = if origin.return_token == Some(token) {
            if function.no_result {
                return Err(fail());
            }
            (&function.returns, &mut result.returns)
        } else {
            let position = origin
                .parameter_tokens
                .iter()
                .position(|t| *t == token)
                .ok_or_else(fail)?;
            (
                function.parameters.get(position).ok_or_else(fail)?,
                &mut result.parameters[position],
            )
        };
        if source_name(module, target_type) != Some("System.Introspection.TypeInfo") {
            return Err(fail());
        }
        *target_flags |= value;
    }
    if let Some(owner) = function
        .owner
        .as_ref()
        .and_then(|t| module.type_definition(t))
    {
        for (index, property) in owner.properties.iter().enumerate() {
            let value = property_flags(module, owner, index)?;
            if value == 0 {
                continue;
            }
            let matches = |target: &crate::metadata::FunctionRef| {
                target.name == function.name
                    && target.instance == function.instance
                    && target.parameters == function.parameters
            };
            if property.getter.as_ref().is_some_and(matches) {
                result.returns |= value;
            }
            if property.setter.as_ref().is_some_and(matches) {
                let slot = result
                    .parameters
                    .last_mut()
                    .ok_or_else(|| Fault::new("DAM1001: property setter lacks value parameter"))?;
                *slot |= value;
            }
        }
    }
    Ok(result)
}

/// Diagnose parameter and return annotation mismatches in verified, source-identity metadata.
/// This bounded check covers nongeneric nominal classes and interface methods;
/// it does not add dispatch targets to retention or analyze their bodies.
pub fn interface_contract_diagnostics(module: &Module) -> Result<BTreeSet<String>, Fault> {
    crate::runtime_lookup::with_module(module, || {
        let mut diagnostics = BTreeSet::new();
        if !module
            .functions
            .iter()
            .map(|f| requirements(module, f))
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .any(|c| !c.is_empty())
        {
            return Ok(diagnostics);
        }
        for ty in &module.types {
            if !ty.is_reference_type
                || ty.is_abstract
                || !ty.generic_parameters.is_empty()
                || ty.representation != crate::metadata::Representation::Record
            {
                continue;
            }
            let concrete = Type::Named(ty.name.clone());
            for interface in crate::interfaces::closure(module, &concrete)? {
                if !matches!(&interface, Type::Named(_)) {
                    continue;
                }
                let definition = crate::interfaces::interface_definition(module, &interface)?;
                if !definition.generic_parameters.is_empty() {
                    continue;
                }
                for contract in crate::runtime_lookup::functions_for_owner(module, definition) {
                    if crate::interfaces::is_helper(contract)
                        || !contract.interface_implementations.is_empty()
                        || !contract.generic_parameters.is_empty()
                    {
                        continue;
                    }
                    let implementation =
                        crate::interfaces::implementation(module, &concrete, &interface, contract)?;
                    let declared = requirements(module, contract)?;
                    let actual = requirements(module, &implementation)?;
                    if declared.receiver != actual.receiver {
                        diagnostics.insert(format!("DAM2005: {} receiver requires {}, but interface {} requires {}; corresponding receiver annotations must match", display_name(module, &implementation), actual.receiver, display_name(module, contract), declared.receiver));
                    }
                    if declared.returns != actual.returns {
                        diagnostics.insert(format!("DAM2005: {} return requires {}, but interface {} requires {}; corresponding return annotations must match", display_name(module, &implementation), actual.returns, display_name(module, contract), declared.returns));
                    }
                    for (index, (expected, found)) in declared
                        .parameters
                        .iter()
                        .zip(&actual.parameters)
                        .enumerate()
                    {
                        if expected != found {
                            diagnostics.insert(format!(
                                "DAM2005: {} parameter {index} requires {found}, but interface {} requires {expected}; corresponding parameter annotations must match",
                                display_name(module, &implementation), display_name(module, contract)));
                        }
                    }
                }
            }
        }
        Ok(diagnostics)
    })
}

/// Check nongeneric nominal reference-class override parameter and return contracts.
/// Abstract declarations participate; checking does not select executable targets.
pub fn override_contract_diagnostics(module: &Module) -> Result<BTreeSet<String>, Fault> {
    crate::runtime_lookup::with_module(module, || {
        let mut diagnostics = BTreeSet::new();
        for implementation in &module.functions {
            if !implementation.is_override || !implementation.generic_parameters.is_empty() {
                continue;
            }
            let Some(owner) = &implementation.owner else {
                continue;
            };
            let Some(ty) = module.type_definition(owner) else {
                continue;
            };
            if !ty.is_reference_type
                || !ty.generic_parameters.is_empty()
                || ty.representation != crate::metadata::Representation::Record
            {
                continue;
            }
            // A verified nongeneric owner has closed ancestry. The shared lookup
            // substitutes each constructed base before matching the exact slot.
            let Some(contract) =
                crate::inheritance::inherited_method(module, owner, implementation)?
            else {
                // Implicit runtime Object slots have no TypeInfo parameter contract.
                continue;
            };
            if !contract.generic_parameters.is_empty() {
                continue;
            }
            let declared = requirements(module, &contract)?;
            let actual = requirements(module, implementation)?;
            if declared.receiver != actual.receiver {
                diagnostics.insert(format!("DAM2006: {} receiver requires {}, but overridden {} requires {}; corresponding receiver annotations must match", display_name(module, implementation), actual.receiver, display_name(module, &contract), declared.receiver));
            }
            if declared.returns != actual.returns {
                diagnostics.insert(format!("DAM2006: {} return requires {}, but overridden {} requires {}; corresponding return annotations must match", display_name(module, implementation), actual.returns, display_name(module, &contract), declared.returns));
            }
            for (index, (expected, found)) in declared
                .parameters
                .iter()
                .zip(&actual.parameters)
                .enumerate()
            {
                if expected != found {
                    diagnostics.insert(format!(
                        "DAM2006: {} parameter {index} requires {found}, but overridden {} requires {expected}; corresponding parameter annotations must match",
                        display_name(module, implementation), display_name(module, &contract)));
                }
            }
        }
        Ok(diagnostics)
    })
}

#[derive(Clone, PartialEq, Eq)]
struct State {
    stack: Vec<Value>,
    locals: Vec<Value>,
    arguments: Vec<Value>,
}

fn merge_value(target: &mut Value, incoming: &Value) {
    if target == incoming {
        return;
    }
    *target = match (&*target, incoming) {
        (Value::Types(a), Value::Types(b)) if a.len() + b.len() <= 64 => {
            let mut types = a.clone();
            for ty in b {
                if !types.contains(ty) {
                    types.push(ty.clone());
                }
            }
            Value::Types(types)
        }
        (Value::Required(a), Value::Required(b)) => Value::Required(a & b),
        _ => Value::Unknown,
    };
}

fn require(module: &Module, value: &Value, flags: u32, site: &str, report: &mut Analysis) {
    if flags == 0 {
        return;
    }
    match value {
        Value::Required(available) if available & flags == flags => (),
        Value::Types(types) => {
            for ty in types {
                if let Some(index) = module.types.iter().position(|d| {
                    ty == &Type::Named(d.name.clone()) && d.generic_parameters.is_empty()
                }) {
                    *report.types.entry(index).or_default() |= flags;
                } else {
                    report.diagnostics.insert(format!("DAM2002: {site}: unsupported dynamic member target {ty:?}; explicit retention is required"));
                }
            }
        }
        _ => {
            report.diagnostics.insert(format!("DAM2001: {site}: cannot prove dynamic member requirement {flags}; annotate the incoming TypeInfo parameter or supply explicit retention"));
        }
    }
}

/// Analyze a verified, linked module. This initial implementation admits TypeInfo
/// parameter and return contracts for public constructors and properties. Other targets/flags
/// fail explicitly rather than becoming inert annotations.
pub fn analyze(module: &Module, root: &str) -> Result<Analysis, Fault> {
    analyze_with_roots(module, root, &[])
}

/// Include executable bodies admitted by a validated explicit retention policy.
/// These seeds do not imply member requirements on their declaring types.
pub fn analyze_with_roots(
    module: &Module,
    root: &str,
    retained_bodies: &[usize],
) -> Result<Analysis, Fault> {
    if retained_bodies.iter().any(|i| *i >= module.functions.len()) {
        return Err(Fault::new(
            "DAM analysis retained body is outside the module",
        ));
    }
    crate::runtime_lookup::with_module(module, || analyze_indexed(module, root, retained_bodies))
}

fn analyze_indexed(
    module: &Module,
    root: &str,
    retained_bodies: &[usize],
) -> Result<Analysis, Fault> {
    let root = module
        .functions
        .iter()
        .position(|f| f.name == root)
        .ok_or_else(|| Fault::new("DAM analysis root is missing"))?;
    let mut roots = BTreeSet::from([root]);
    roots.extend(retained_bodies.iter().copied());
    loop {
        let report = analyze_roots(module, &roots)?;
        let before = roots.len();
        for (&index, &flags) in &report.types {
            let ty = &module.types[index];
            let owner = Type::Named(ty.name.clone());
            for (index, function) in module.functions.iter().enumerate() {
                if function.owner.as_ref() != Some(&owner)
                    || function.visibility != crate::metadata::Visibility::Public
                    || function.origin.as_ref().is_some_and(|o| {
                        o.member_access
                            .is_some_and(|a| a != crate::metadata_origin::SourceAccess::Public)
                    })
                {
                    continue;
                }
                let constructor = flags & PUBLIC_CONSTRUCTORS != 0
                    && function.instance
                    && function.name.ends_with("..ctor");
                // Property associations retain source identities while linked
                // function rows have been relocated. Match the verified signature.
                let accessor = flags & PUBLIC_PROPERTIES != 0
                    && ty
                        .properties
                        .iter()
                        .filter(|p| p.instance && p.parameters.is_empty())
                        .flat_map(|p| p.getter.iter().chain(&p.setter))
                        .any(|target| {
                            target.name == function.name
                                && target.owner == function.owner
                                && target.instance == function.instance
                                && target.parameters == function.parameters
                        });
                if constructor || accessor {
                    roots.insert(index);
                }
            }
        }
        if roots.len() == before {
            return Ok(report);
        }
        // Roots only grow within this finite module. Recompute flow to include
        // calls made by newly retained constructors/accessors, including cycles.
    }
}

fn analyze_roots(module: &Module, roots: &BTreeSet<usize>) -> Result<Analysis, Fault> {
    let mut report = Analysis::default();
    let storage_contracts = validate_storage_contracts(module)?;
    let contracts = module
        .functions
        .iter()
        .map(|f| requirements(module, f))
        .collect::<Result<Vec<_>, _>>()?;
    if !storage_contracts && contracts.iter().all(Contract::is_empty) {
        return Ok(report);
    }
    let mut reachable = BTreeSet::new();
    let mut pending: Vec<_> = roots.iter().copied().collect();
    while let Some(index) = pending.pop() {
        if !reachable.insert(index) {
            continue;
        }
        // Static field access and strict type entry can execute initializers even
        // though the IL contains no direct call to .cctor.
        let function = &module.functions[index];
        let owners = crate::type_initialization::entry_owner(module, function)
            .into_iter()
            .chain(function.body.iter().filter_map(|op| match op {
                Op::StaticField { owner, .. } | Op::SetStaticField { owner, .. } => Some(owner),
                _ => None,
            }));
        for owner in owners {
            if let Some(initializer) = crate::type_initialization::initializer(module, owner)? {
                for (candidate, f) in module.functions.iter().enumerate() {
                    if f.name == initializer.name
                        && f.owner
                            .as_ref()
                            .and_then(|t| module.type_definition(t))
                            .zip(module.type_definition(owner))
                            .is_some_and(|(a, b)| std::ptr::eq(a, b))
                    {
                        pending.push(candidate);
                    }
                }
            }
        }
        for op in &function.body {
            let target = match op {
                Op::Call(t)
                | Op::CallVirtual(t)
                | Op::Construct(t)
                | Op::BindFunction { target: t, .. }
                | Op::CallSelf { target: t, .. } => t,
                _ => continue,
            };
            let resolved = crate::vm::resolve(module, target)?;
            let targets = vec![resolved.clone()];
            // Dynamic implementations are selected after specialization. The first
            // slice checks the declared call contract, but never roots every
            // same-named implementation in the input libraries.
            if matches!(op, Op::CallVirtual(_) | Op::CallSelf { .. }) {
                report.diagnostics.insert("DAM2004: dynamic-dispatch implementation bodies are not analyzed; reflection inside them may require explicit retention".into());
                continue;
            }
            for resolved in targets {
                let candidates =
                    crate::runtime_lookup::functions(module, &resolved.name).unwrap_or_default();
                for &candidate in candidates.iter() {
                    let f = &module.functions[candidate];
                    if resolved.definition.as_ref().is_some_and(|id| {
                        f.definition.as_ref().map_or(
                            id.module == module.name && id.index as usize == candidate,
                            |d| d == id,
                        )
                    }) {
                        pending.push(candidate);
                    }
                }
            }
        }
    }
    for index in reachable {
        let function = &module.functions[index];
        let contract = &contracts[index];
        let _ = function.map_types(|ty| {
            check_type_arguments(module, function, ty, &mut report)?;
            Ok(ty.clone())
        })?;
        if function.body.is_empty() {
            continue;
        }
        let mut arguments =
            vec![Value::Required(contract.receiver); usize::from(function.instance)];
        arguments.extend(contract.parameters.iter().copied().map(Value::Required));
        let initial = State {
            stack: vec![],
            locals: vec![Value::Unknown; function.locals.len()],
            arguments,
        };
        let mut states = vec![None; function.body.len()];
        states[0] = Some(initial);
        let mut queue = VecDeque::from([0]);
        let mut relevant = storage_contracts || contract.returns != 0;
        for op in &function.body {
            if let Op::Call(t)
            | Op::CallVirtual(t)
            | Op::CallSelf { target: t, .. }
            | Op::Construct(t)
            | Op::BindFunction { target: t, .. } = op
            {
                let callee = crate::vm::resolve(module, t)?;
                relevant |= !requirements(module, &callee)?.is_empty();
            }
        }
        if !relevant {
            continue;
        }
        let mut escaped_locals = BTreeSet::new();
        let mut escaped_arguments = BTreeSet::new();
        for op in &function.body {
            match op {
                Op::LocalAddress(i)
                | Op::Receiver {
                    argument: false,
                    index: i,
                    readonly_value: false,
                } => {
                    escaped_locals.insert(*i);
                }
                Op::ArgumentAddress(i)
                | Op::Receiver {
                    argument: true,
                    index: i,
                    readonly_value: false,
                } => {
                    escaped_arguments.insert(*i);
                }
                _ => (),
            }
        }
        let effects = crate::verifier::instruction_effects(module, function)?;
        while let Some(pc) = queue.pop_front() {
            let mut state = states[pc].clone().unwrap();
            let op = &function.body[pc];
            let effect = effects[pc]
                .as_ref()
                .ok_or_else(|| Fault::new("DAM analysis reached an unverified instruction"))?;
            let (pops, pushes) = (effect.pops, effect.pushes);
            if state.stack.len() < pops {
                return Err(Fault::new("DAM analysis stack underflow"));
            }
            let values = state.stack.split_off(state.stack.len() - pops);
            let mut produced = vec![Value::Unknown; pushes];
            match op {
                Op::Field(index) | Op::SetField(index) | Op::FieldAddress(index) => {
                    let owner = effect
                        .receiver
                        .as_ref()
                        .ok_or_else(|| Fault::new("DAM field receiver missing"))?;
                    let value = field_flags(module, owner, *index, false)?;
                    match op {
                        Op::Field(_) => produced[0] = Value::Required(value),
                        Op::SetField(_) => require(
                            module,
                            &values[1],
                            value,
                            &format!(
                                "{} instruction {pc} field store",
                                display_name(module, function)
                            ),
                            &mut report,
                        ),
                        _ if value != 0 => {
                            report.diagnostics.insert(format!("DAM2007: {} instruction {pc}: address of annotated field escapes flow analysis; writes through it require explicit retention", display_name(module, function)));
                        }
                        _ => (),
                    }
                }
                Op::StaticField { owner, index } => {
                    produced[0] = Value::Required(field_flags(module, owner, *index, true)?)
                }
                Op::SetStaticField { owner, index } => require(
                    module,
                    &values[0],
                    field_flags(module, owner, *index, true)?,
                    &format!(
                        "{} instruction {pc} static field store",
                        display_name(module, function)
                    ),
                    &mut report,
                ),
                Op::New(owner) => {
                    for (index, value) in values.iter().enumerate() {
                        require(
                            module,
                            value,
                            field_flags(module, owner, index, false)?,
                            &format!(
                                "{} instruction {pc} field {index} initialization",
                                display_name(module, function)
                            ),
                            &mut report,
                        );
                    }
                }
                Op::LoadTypeToken(ty) => produced[0] = type_value(module, function, ty)?,
                Op::Arg(index) if !escaped_arguments.contains(index) => {
                    produced[0] = state.arguments[*index].clone()
                }
                Op::Load(index) if !escaped_locals.contains(index) => {
                    produced[0] = state.locals[*index].clone()
                }
                Op::Store(index) => state.locals[*index] = values[0].clone(),
                Op::StoreArg(index) => state.arguments[*index] = values[0].clone(),
                Op::ResetLocal(index) => state.locals[*index] = Value::Unknown,
                Op::Dup => produced.fill(values[0].clone()),
                Op::CastClass(_) | Op::BorrowInterface(_) => produced[0] = values[0].clone(),
                Op::Call(target)
                | Op::CallVirtual(target)
                | Op::CallSelf { target, .. }
                | Op::Construct(target) => {
                    let callee = crate::vm::resolve(module, target)?;
                    check_method_arguments(module, function, target, &callee, &mut report)?;
                    let callee_contract = requirements(module, &callee)?;
                    if callee_contract.receiver != 0 {
                        require(
                            module,
                            &values[0],
                            callee_contract.receiver,
                            &format!(
                                "{} instruction {pc} -> {} receiver",
                                display_name(module, function),
                                display_name(module, &callee)
                            ),
                            &mut report,
                        );
                    }
                    if callee_contract.returns != 0 && pushes == 1 {
                        produced[0] = Value::Required(callee_contract.returns);
                    }
                    for (index, flags) in callee_contract.parameters.into_iter().enumerate() {
                        require(
                            module,
                            &values[index
                                + usize::from(target.instance && !matches!(op, Op::Construct(_)))],
                            flags,
                            &format!(
                                "{} instruction {pc} -> {} parameter {index}",
                                display_name(module, function),
                                display_name(module, &callee)
                            ),
                            &mut report,
                        );
                    }
                    // The exact runtime bridge turns a type token into its TypeInfo view.
                    let bridge = callee
                        .origin
                        .as_ref()
                        .zip(callee.owner.as_ref())
                        .is_some_and(|(origin, owner)| match source_name(module, owner) {
                            Some("System.Introspection.RuntimeTypeInfo") => {
                                !callee.instance && origin.name == "FromHandle"
                            }
                            Some("System.Runtime.RuntimeContext") => {
                                callee.instance && origin.name == "GetTypeInfoFromHandle"
                            }
                            _ => false,
                        });
                    if bridge
                        && callee.parameters == [Type::RuntimeTypeHandle]
                        && pushes == 1
                        && source_name(module, &callee.returns)
                            == Some("System.Introspection.TypeInfo")
                    {
                        produced[0] = values.last().unwrap().clone();
                    }
                }
                Op::Return if contract.returns != 0 => {
                    require(
                        module,
                        &values[0],
                        contract.returns,
                        &format!("{} instruction {pc} return", display_name(module, function)),
                        &mut report,
                    );
                }
                Op::BindFunction { target, .. } => {
                    let callee = crate::vm::resolve(module, target)?;
                    check_method_arguments(module, function, target, &callee, &mut report)?;
                    if !requirements(module, &callee)?.is_empty() {
                        report.diagnostics.insert(format!("DAM2003: {} instruction {pc}: indirect call to annotated {} requires explicit retention", display_name(module, function), display_name(module, &callee)));
                    }
                }
                _ => (),
            }
            state.stack.extend(produced);
            let successors = match op {
                Op::Return | Op::Fault(_) => vec![],
                Op::Branch(t) => vec![*t],
                Op::BranchTrue(t)
                | Op::BranchFalse(t)
                | Op::BranchEqual(t)
                | Op::BranchNotEqual(t)
                | Op::BranchGreater(t)
                | Op::BranchGreaterUnsigned(t)
                | Op::BranchLess(t)
                | Op::BranchLessUnsigned(t)
                | Op::BranchGreaterEqual(t)
                | Op::BranchGreaterEqualUnsigned(t)
                | Op::BranchLessEqual(t)
                | Op::BranchLessEqualUnsigned(t) => vec![*t, pc + 1],
                Op::Switch(targets) => targets.iter().copied().chain([pc + 1]).collect(),
                _ => vec![pc + 1],
            };
            for next in successors {
                if next >= states.len() {
                    return Err(Fault::new("DAM analysis branch outside body"));
                }
                if let Some(previous) = &mut states[next] {
                    if previous.stack.len() != state.stack.len() {
                        return Err(Fault::new(format!(
                            "DAM analysis inconsistent stack in {} at {pc} -> {next}: {} vs {}",
                            function.name,
                            previous.stack.len(),
                            state.stack.len()
                        )));
                    }
                    let old = previous.clone();
                    for (a, b) in previous
                        .stack
                        .iter_mut()
                        .zip(&state.stack)
                        .chain(previous.locals.iter_mut().zip(&state.locals))
                        .chain(previous.arguments.iter_mut().zip(&state.arguments))
                    {
                        merge_value(a, b);
                    }
                    if old != *previous {
                        queue.push_back(next);
                    }
                } else {
                    states[next] = Some(state.clone());
                    queue.push_back(next);
                }
            }
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::{CustomAttribute, FunctionRef};
    use serde_json::json;

    fn function(name: &str, parameters: Vec<Type>, body: Vec<Op>) -> Function {
        serde_json::from_value(json!({"name":name,"parameters":parameters,"returns":"Void","no_result":true,"body":body})).unwrap()
    }

    fn call(f: &Function) -> Op {
        Op::Call(
            serde_json::from_value::<FunctionRef>(json!({"name":f.name,"parameters":f.parameters}))
                .unwrap(),
        )
    }

    fn fixture() -> Module {
        let mut m = crate::assemble(&".module Test\n.type Attr\n.end\n.type Info\n.end\n.type RuntimeInfo\n.end\n.type Model\n.end\n.type Other\n.end\n").unwrap();
        for (i, name) in [
            ATTRIBUTE,
            "System.Introspection.TypeInfo",
            "System.Introspection.RuntimeTypeInfo",
        ]
        .iter()
        .enumerate()
        {
            m.types[i].origin = Some(
                serde_json::from_value(
                    json!({"assembly":"Test","module":"Test.dll","name":name,"token":33554433+i}),
                )
                .unwrap(),
            );
        }
        let info = Type::Named("Info".into());
        let mut required = function("Require", vec![info.clone()], vec![Op::Return]);
        required.origin = Some(serde_json::from_value(json!({"assembly":"Test","module":"Test.dll","name":"Require","token":100663297,"parameter_tokens":[134217729]})).unwrap());
        required.custom_attributes.push(serde_json::from_value::<CustomAttribute>(json!({
            "constructor":{"name":".ctor","owner":{"Named":"Attr"},"instance":true,"parameters":["Int32"]},
            "arguments":[{"Int32":3}],"target_token":134217729})).unwrap());
        let mut bridge = function("FromHandle", vec![Type::RuntimeTypeHandle], vec![]);
        bridge.owner = Some(Type::Named("RuntimeInfo".into()));
        bridge.origin = Some(serde_json::from_value(json!({"assembly":"System.Runtime","module":"System.Runtime.dll","name":"FromHandle","token":100663298})).unwrap());
        bridge.no_result = false;
        bridge.returns = info;
        let bridge_call = Op::Call(
            serde_json::from_value(
                json!({"name":"FromHandle","owner":bridge.owner,"parameters":bridge.parameters}),
            )
            .unwrap(),
        );
        let main = function(
            "Main",
            vec![],
            vec![
                Op::LoadTypeToken(Type::Named("Model".into())),
                bridge_call,
                call(&required),
                Op::Return,
            ],
        );
        m.functions = vec![main, required, bridge];
        m
    }

    fn generic_fixture() -> Module {
        let mut m = fixture();
        let mut generic = function("Generic", vec![], m.functions[0].body.clone());
        generic.generic_parameters = vec![Some("T".into())];
        generic.body[0] = Op::LoadTypeToken(Type::MethodTypeParameter(0));
        generic.origin = Some(serde_json::from_value(json!({"assembly":"Test","module":"Test.dll","name":"Generic","token":100663299,"generic_parameter_tokens":[704643073]})).unwrap());
        let mut attr = m.functions[1].custom_attributes[0].clone();
        attr.target_token = Some(704643073);
        generic.custom_attributes.push(attr);
        m.functions[0].body = vec![Op::Call(serde_json::from_value(json!({"name":"Generic","parameters":[],"generic_arguments":[{"Named":"Model"}]})).unwrap()), Op::Return];
        m.functions.push(generic);
        m
    }

    #[test]
    fn generic_method_argument_retains_members_and_typeof_satisfies_consumer() {
        let m = generic_fixture();
        let report = analyze(&m, "Main").unwrap();
        assert_eq!(report.types.get(&3), Some(&3));
        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    }

    #[test]
    fn unannotated_generic_parameter_does_not_satisfy_typeof_consumer() {
        let mut m = generic_fixture();
        m.functions[3].custom_attributes.clear();
        let report = analyze(&m, "Main").unwrap();
        assert!(report.types.is_empty());
        assert!(
            report.diagnostics.iter().any(|d| d.contains("DAM2001")),
            "{:?}",
            report.diagnostics
        );
    }

    #[test]
    fn generic_forwarding_requires_matching_annotation() {
        let mut m = generic_fixture();
        let mut forward = function("Forward", vec![], vec![Op::Call(serde_json::from_value(json!({"name":"Generic","parameters":[],"generic_arguments":[{"MethodTypeParameter":0}]})).unwrap()), Op::Return]);
        forward.generic_parameters = vec![Some("U".into())];
        m.functions[0].body = vec![Op::Call(serde_json::from_value(json!({"name":"Forward","parameters":[],"generic_arguments":[{"Named":"Model"}]})).unwrap()), Op::Return];
        m.functions.push(forward);
        let report = analyze(&m, "Main").unwrap();
        assert!(report.types.is_empty());
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.contains("DAM2001") && d.contains("generic parameter 0"))
        );
        m.functions[4].origin = m.functions[3].origin.clone();
        m.functions[4].origin.as_mut().unwrap().name = "Forward".into();
        m.functions[4].custom_attributes = m.functions[3].custom_attributes.clone();
        let report = analyze(&m, "Main").unwrap();
        assert_eq!(report.types.get(&3), Some(&3));
        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    }

    #[test]
    fn constructed_type_argument_retains_generic_contract_without_method_contracts() {
        let mut m = fixture();
        m.types[4].generic_parameters = vec![Some("T".into())];
        m.types[4].origin = Some(serde_json::from_value(json!({"assembly":"Test","module":"Test.dll","name":"Other`1","token":33554437,"generic_parameter_tokens":[704643073]})).unwrap());
        let mut attr = m.functions[1].custom_attributes[0].clone();
        attr.target_token = Some(704643073);
        m.types[4].custom_attributes.push(attr);
        m.functions.truncate(1);
        m.functions[0].body = vec![
            Op::LoadTypeToken(Type::Constructed {
                definition: "Other".into(),
                arguments: vec![Type::Named("Model".into())],
            }),
            Op::Pop,
            Op::Return,
        ];
        let report = analyze(&m, "Main").unwrap();
        assert_eq!(report.types.get(&3), Some(&3));
        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    }

    fn return_fixture() -> Module {
        let mut m = fixture();
        let mut factory = function("Factory", vec![], m.functions[0].body[..2].to_vec());
        factory.body.push(Op::Return);
        factory.returns = Type::Named("Info".into());
        factory.no_result = false;
        factory.origin = Some(serde_json::from_value(json!({"assembly":"Test","module":"Test.dll","name":"Factory","token":100663299,"return_token":134217730})).unwrap());
        let mut attribute = m.functions[1].custom_attributes[0].clone();
        attribute.target_token = Some(134217730);
        factory.custom_attributes.push(attribute);
        m.functions[0].body = vec![call(&factory), call(&m.functions[1]), Op::Return];
        m.functions.push(factory);
        m
    }

    #[test]
    fn return_contract_roots_factory_type_and_satisfies_consumer() {
        let m = return_fixture();
        let report = analyze(&m, "Main").unwrap();
        assert_eq!(report.types.get(&3), Some(&3));
        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    }

    #[test]
    fn unknown_return_is_diagnosed_even_without_annotated_consumers() {
        let mut m = return_fixture();
        m.functions[3].parameters.push(Type::RuntimeTypeHandle);
        m.functions[3]
            .origin
            .as_mut()
            .unwrap()
            .parameter_tokens
            .push(0);
        m.functions[3].body[0] = Op::Arg(0);
        let report = analyze(&m, "Factory").unwrap();
        assert!(report.types.is_empty());
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.contains("DAM2001") && d.contains("Factory") && d.contains("return"))
        );
    }

    #[test]
    fn return_contract_does_not_invent_stronger_guarantee() {
        let mut m = return_fixture();
        m.functions[1].custom_attributes[0].arguments = vec![AttributeArgument::Int32(512)];
        let report = analyze(&m, "Main").unwrap();
        assert_eq!(report.types.get(&3), Some(&3));
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.contains("DAM2001") && d.contains("parameter 0"))
        );
        m.functions[3].no_result = true;
        assert!(
            analyze(&m, "Main")
                .unwrap_err()
                .to_string()
                .contains("DAM1001")
        );
    }

    fn storage_fixture(property: bool) -> Module {
        let mut m = fixture();
        m.types[3].is_reference_type = true;
        m.types[3]
            .fields
            .push(serde_json::from_value(json!({"name":"Stored", "ty":{"Named":"Info"}})).unwrap());
        m.types[3].origin = Some(serde_json::from_value(json!({"assembly":"Test","module":"Test.dll","name":"Model","token":33554436,"field_tokens":[67108865],"property_tokens":if property {vec![385875969]} else {vec![]}})).unwrap());
        let mut attribute = m.functions[1].custom_attributes[0].clone();
        attribute.target_token = Some(if property { 385875969 } else { 67108865 });
        m.types[3].custom_attributes.push(attribute);
        let mut getter = function(
            "Model.get_Type",
            vec![],
            vec![Op::Arg(0), Op::Field(0), Op::Return],
        );
        getter.owner = Some(Type::Named("Model".into()));
        getter.instance = true;
        getter.returns = Type::Named("Info".into());
        getter.no_result = false;
        let mut setter = function(
            "Model.set_Type",
            vec![Type::Named("Info".into())],
            vec![Op::Arg(0), Op::Arg(1), Op::SetField(0), Op::Return],
        );
        setter.owner = getter.owner.clone();
        setter.instance = true;
        let reference = |f: &Function| {
            serde_json::from_value::<FunctionRef>(
                json!({"name":f.name,"owner":f.owner,"instance":true,"parameters":f.parameters}),
            )
            .unwrap()
        };
        if property {
            m.types[3].properties.push(serde_json::from_value(json!({"name":"Type","instance":true,"parameters":[],"ty":{"Named":"Info"},"getter":reference(&getter),"setter":reference(&setter)})).unwrap());
        }
        m.functions[0].body = vec![
            Op::LoadTypeToken(Type::Named("Other".into())),
            m.functions[0].body[1].clone(),
            Op::New(Type::Named("Model".into())),
            if property {
                Op::Call(reference(&getter))
            } else {
                Op::Field(0)
            },
            call(&m.functions[1]),
            Op::Return,
        ];
        m.functions.extend([getter, setter]);
        m
    }

    #[test]
    fn field_and_property_storage_infer_types_and_check_unknown_writes() {
        for property in [false, true] {
            let mut m = storage_fixture(property);
            let report = analyze(&m, "Main").unwrap();
            assert_eq!(report.types, BTreeMap::from([(4, 3)]));
            assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
            m.functions[0].parameters.push(Type::RuntimeTypeHandle);
            m.functions[0].body[0] = Op::Arg(0);
            assert!(
                analyze(&m, "Main")
                    .unwrap()
                    .diagnostics
                    .iter()
                    .any(|d| d.contains("DAM2001") && d.contains("initialization"))
            );
            // Unannotated field stores must warn; a property setter has the
            // property's incoming contract and can safely forward to its storage.
            let report = analyze(&m, "Model.set_Type").unwrap();
            assert_eq!(report.diagnostics.is_empty(), property);
        }
    }

    #[test]
    fn property_caller_must_satisfy_setter_value_contract() {
        let mut m = storage_fixture(true);
        m.functions[0].parameters = vec![Type::Named("Model".into()), Type::Named("Info".into())];
        m.functions[0].body = vec![
            Op::Arg(0),
            Op::Arg(1),
            Op::Call(m.types[3].properties[0].setter.clone().unwrap()),
            Op::Return,
        ];
        let report = analyze(&m, "Main").unwrap();
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.contains("DAM2001") && d.contains("parameter 0"))
        );
    }

    #[test]
    fn computed_property_getter_does_not_invent_backing_field_contract() {
        let mut m = storage_fixture(true);
        m.functions[3].parameters.push(Type::Named("Info".into()));
        m.functions[3].body = vec![Op::Arg(1), Op::Return];
        m.types[3].properties[0]
            .parameters
            .push(Type::Named("Info".into()));
        m.types[3].properties[0]
            .getter
            .as_mut()
            .unwrap()
            .parameters
            .push(Type::Named("Info".into()));
        let report = analyze(&m, "Model.get_Type").unwrap();
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.contains("DAM2001") && d.contains("return"))
        );
    }

    #[test]
    fn structural_fields_have_no_nominal_attribute_owner() {
        let mut m = storage_fixture(true);
        let tuple = Type::Tuple(
            crate::tuple_signature::TupleSignature::new(vec![Type::Int32], vec![]).unwrap(),
        );
        m.functions[0]
            .body
            .splice(0..0, [Op::Int(7), Op::New(tuple), Op::Field(0), Op::Pop]);
        let report = analyze(&m, "Main").unwrap();
        assert!(report.diagnostics.is_empty());
        assert_eq!(report.types, BTreeMap::from([(4, 3)]));
    }

    #[test]
    fn inherited_storage_uses_the_declaring_field_contract() {
        let mut m = storage_fixture(false);
        m.types[4].is_reference_type = true;
        m.types[4].base = Some(Type::Named("Model".into()));
        m.functions[0].body[2] = Op::New(Type::Named("Other".into()));
        assert_eq!(analyze(&m, "Main").unwrap().types, BTreeMap::from([(4, 3)]));
        m.types[3].fields[0].ty = Type::Int32;
        assert!(
            analyze(&m, "Main")
                .unwrap_err()
                .to_string()
                .contains("DAM1001")
        );
    }

    #[test]
    fn annotated_field_address_is_explicitly_diagnosed() {
        let mut m = storage_fixture(false);
        m.functions[0].body[3] = Op::FieldAddress(0);
        m.functions[0].body[4] = Op::Pop;
        assert!(
            analyze(&m, "Main")
                .unwrap()
                .diagnostics
                .iter()
                .any(|d| d.contains("DAM2007"))
        );
    }

    #[test]
    fn static_field_initialization_is_analyzed_without_an_explicit_call() {
        let mut m = storage_fixture(false);
        m.types[3].fields.clear();
        m.types[3]
            .static_fields
            .push(serde_json::from_value(json!({"name":"Stored","ty":{"Named":"Info"}})).unwrap());
        let mut init = function(
            "Model..cctor",
            vec![],
            vec![
                Op::LoadTypeToken(Type::Named("Other".into())),
                m.functions[0].body[1].clone(),
                Op::SetStaticField {
                    owner: Type::Named("Model".into()),
                    index: 0,
                },
                Op::Return,
            ],
        );
        init.owner = Some(Type::Named("Model".into()));
        init.visibility = crate::metadata::Visibility::Private;
        m.functions.truncate(3);
        m.functions.push(init);
        m.functions[0].body = vec![
            Op::StaticField {
                owner: Type::Named("Model".into()),
                index: 0,
            },
            call(&m.functions[1]),
            Op::Return,
        ];
        let report = analyze(&m, "Main").unwrap();
        assert_eq!(report.types, BTreeMap::from([(4, 3)]));
        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    }

    #[test]
    fn receiver_contract_retains_known_type_and_flows_through_self() {
        let mut m = fixture();
        m.types[1].is_reference_type = true;
        let mut method = function(
            "Info.Use",
            vec![],
            vec![Op::Arg(0), call(&m.functions[1]), Op::Return],
        );
        method.owner = Some(Type::Named("Info".into()));
        method.instance = true;
        let mut attribute = m.functions[1].custom_attributes[0].clone();
        attribute.target_token = None;
        method.custom_attributes.push(attribute);
        let target = serde_json::from_value(
            json!({"name":method.name,"owner":method.owner,"instance":true,"parameters":[]}),
        )
        .unwrap();
        m.functions[0].body[2] = Op::Call(target);
        m.functions.push(method);
        let report = analyze(&m, "Main").unwrap();
        assert_eq!(report.types, BTreeMap::from([(3, 3)]));
        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
        m.functions[0].parameters.push(Type::RuntimeTypeHandle);
        m.functions[0].body[0] = Op::Arg(0);
        assert!(
            analyze(&m, "Main")
                .unwrap()
                .diagnostics
                .iter()
                .any(|d| d.contains("DAM2001") && d.contains("receiver"))
        );
        m.functions[3].instance = false;
        assert!(
            analyze(&m, "Main")
                .unwrap_err()
                .to_string()
                .contains("DAM1001")
        );
    }

    #[test]
    fn override_receiver_requirements_match_inherited_descriptor_contract() {
        let mut m = override_fixture();
        let mut interface = interface_fixture().types.pop().unwrap();
        interface.origin=Some(serde_json::from_value(json!({"assembly":"Test","module":"Test.dll","name":"System.Introspection.TypeInfo","token":33554438})).unwrap());
        m.types.push(interface);
        m.types[4].implements.push(Type::Named("Sink".into()));
        for f in &mut m.functions[3..5] {
            f.custom_attributes[0].target_token = None;
        }
        assert!(override_contract_diagnostics(&m).unwrap().is_empty());
        m.functions[4].custom_attributes.clear();
        let diagnostics = override_contract_diagnostics(&m).unwrap();
        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics
                .iter()
                .next()
                .unwrap()
                .contains("receiver requires")
        );
    }

    #[test]
    fn interface_receiver_requirements_match_descriptor_implementations() {
        let mut m = interface_fixture();
        m.types[5].origin=Some(serde_json::from_value(json!({"assembly":"Test","module":"Test.dll","name":"System.Introspection.TypeInfo","token":33554438})).unwrap());
        for f in &mut m.functions[3..5] {
            f.custom_attributes[0].target_token = None;
        }
        assert!(interface_contract_diagnostics(&m).unwrap().is_empty());
        m.functions[4].custom_attributes[0].arguments = vec![AttributeArgument::Int32(512)];
        let diagnostics = interface_contract_diagnostics(&m).unwrap();
        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics
                .iter()
                .next()
                .unwrap()
                .contains("receiver requires")
        );
    }

    #[test]
    fn known_type_retains_constructors_and_unknown_input_diagnoses() {
        let mut m = fixture();
        let result = analyze(&m, "Main").unwrap();
        assert_eq!(result.types.get(&3), Some(&3));
        assert!(result.diagnostics.is_empty());
        m.functions[0].parameters.push(Type::RuntimeTypeHandle);
        m.functions[0].body[0] = Op::Arg(0);
        let result = analyze(&m, "Main").unwrap();
        assert!(result.types.is_empty());
        assert!(result.diagnostics.iter().any(|d| d.contains("DAM2001")));
    }

    #[test]
    fn contracts_propagate_through_parameters_and_locals() {
        let mut m = fixture();
        let target = m.functions[1].clone();
        let mut forward = target.clone();
        forward.name = "Forward".into();
        forward.origin.as_mut().unwrap().name = "Forward".into();
        forward.origin.as_mut().unwrap().full_name = Some("Forward".into());
        forward.locals = target.parameters.clone();
        forward.body = vec![
            Op::Arg(0),
            Op::Store(0),
            Op::Load(0),
            call(&target),
            Op::Return,
        ];
        m.functions[0].body[2] = call(&forward);
        m.functions.push(forward);
        let result = analyze(&m, "Main").unwrap();
        assert_eq!(result.types.get(&3), Some(&3));
        assert!(result.diagnostics.is_empty());
        m.functions[3].custom_attributes.clear();
        assert!(
            analyze(&m, "Main")
                .unwrap()
                .diagnostics
                .iter()
                .any(|d| d.contains("Forward"))
        );
    }

    #[test]
    fn branches_union_known_types_but_unknown_arm_diagnoses() {
        let mut m = fixture();
        let bridge = m.functions[0].body[1].clone();
        let target = m.functions[0].body[2].clone();
        m.functions[0].body = vec![
            Op::Bool(true),
            Op::BranchFalse(4),
            Op::LoadTypeToken(Type::Named("Model".into())),
            Op::Branch(5),
            Op::LoadTypeToken(Type::Named("Other".into())),
            bridge,
            target,
            Op::Return,
        ];
        let result = analyze(&m, "Main").unwrap();
        assert_eq!(result.types.len(), 2);
        assert!(result.diagnostics.is_empty());
        m.functions[0].parameters.push(Type::RuntimeTypeHandle);
        m.functions[0].body[4] = Op::Arg(0);
        assert!(!analyze(&m, "Main").unwrap().diagnostics.is_empty());
    }

    #[test]
    fn unsupported_flags_and_targets_are_never_silent() {
        let mut m = fixture();
        m.functions[1].custom_attributes[0].arguments = vec![AttributeArgument::Int32(8)];
        assert!(
            analyze(&m, "Main")
                .unwrap_err()
                .to_string()
                .contains("DAM1001")
        );
        m.functions[1].custom_attributes[0].arguments = vec![AttributeArgument::Int32(3)];
        m.functions[1].custom_attributes[0].target_token = None;
        assert!(analyze(&m, "Main").is_err());
        m.functions[1].custom_attributes[0].target_token = Some(134217729);
        m.functions[1].parameters.clear();
        assert!(
            analyze(&m, "Main")
                .unwrap_err()
                .to_string()
                .contains("DAM1001")
        );
    }

    #[test]
    fn no_annotations_leave_existing_programs_unchanged() {
        let mut m = fixture();
        m.functions[1].custom_attributes.clear();
        let result = analyze(&m, "Main").unwrap();
        assert!(result.types.is_empty());
        assert!(result.diagnostics.is_empty());
    }
    #[test]
    fn unused_callers_do_not_retain_types() {
        let mut m = fixture();
        let mut unused = m.functions[0].clone();
        unused.name = "Unused".into();
        unused.body[0] = Op::LoadTypeToken(Type::Named("Other".into()));
        m.functions.push(unused);
        let result = analyze(&m, "Main").unwrap();
        assert_eq!(result.types.len(), 1);
        assert!(result.types.contains_key(&3));
    }

    #[test]
    fn reference_field_stores_use_verified_stack_effects() {
        let mut m = fixture();
        m.types[3].is_reference_type = true;
        m.types[3]
            .fields
            .push(serde_json::from_value(json!({"name":"Value","ty":"Int32"})).unwrap());
        let prefix = vec![
            Op::Int(0),
            Op::New(Type::Named("Model".into())),
            Op::Dup,
            Op::Int(2),
            Op::SetField(0),
            Op::Pop,
        ];
        m.functions[0].body.splice(0..0, prefix);
        let result = analyze(&m, "Main").unwrap();
        assert!(result.diagnostics.is_empty());
        assert_eq!(result.types.get(&3), Some(&3));
    }
    #[test]
    fn unrelated_addressed_local_does_not_erase_parameter_contract() {
        let mut m = fixture();
        let target = m.functions[1].clone();
        let mut forward = target.clone();
        forward.name = "Forward".into();
        forward.origin.as_mut().unwrap().name = "Forward".into();
        forward.origin.as_mut().unwrap().full_name = Some("Forward".into());
        forward.locals = vec![Type::Int32];
        forward.body = vec![
            Op::Int(0),
            Op::Store(0),
            Op::LocalAddress(0),
            Op::Pop,
            Op::Arg(0),
            call(&target),
            Op::Return,
        ];
        m.functions[0].body[2] = call(&forward);
        m.functions.push(forward);
        assert!(analyze(&m, "Main").unwrap().diagnostics.is_empty());
        m.functions[3].body[2] = Op::ArgumentAddress(0);
        assert!(!analyze(&m, "Main").unwrap().diagnostics.is_empty());
    }
    #[test]
    fn reflected_constructors_close_over_dependencies_and_cycles() {
        let mut m = fixture();
        let bridge = m.functions[0].body[1].clone();
        let require = m.functions[0].body[2].clone();
        for (owner, dependency) in [("Model", "Other"), ("Other", "Model")] {
            let mut constructor = function(
                &format!("{owner}..ctor"),
                vec![],
                vec![
                    Op::LoadTypeToken(Type::Named(dependency.into())),
                    bridge.clone(),
                    require.clone(),
                    Op::Return,
                ],
            );
            constructor.owner = Some(Type::Named(owner.into()));
            constructor.instance = true;
            m.functions.push(constructor);
        }
        let report = analyze(&m, "Main").unwrap();
        assert_eq!(report.types, BTreeMap::from([(3, 3), (4, 3)]));
        assert!(report.diagnostics.is_empty());
        // A private constructor is not admitted by this public-member contract.
        m.functions[3].visibility = crate::metadata::Visibility::Private;
        assert_eq!(analyze(&m, "Main").unwrap().types, BTreeMap::from([(3, 3)]));
    }

    #[test]
    fn reflected_accessors_are_followed_only_for_property_requirements() {
        let mut m = fixture();
        let mut getter = function(
            "Model.get_Value",
            vec![],
            vec![
                Op::LoadTypeToken(Type::Named("Other".into())),
                m.functions[0].body[1].clone(),
                m.functions[0].body[2].clone(),
                Op::Int(7),
                Op::Return,
            ],
        );
        getter.owner = Some(Type::Named("Model".into()));
        getter.instance = true;
        getter.no_result = false;
        getter.returns = Type::Int32;
        m.types[3].properties.push(
            serde_json::from_value(json!({
                "name":"Value", "instance":true, "parameters":[], "ty":"Int32",
                "getter":{"name":getter.name, "owner":getter.owner, "instance":true,
                    "parameters":[]}
            }))
            .unwrap(),
        );
        m.functions.push(getter);
        assert_eq!(analyze(&m, "Main").unwrap().types, BTreeMap::from([(3, 3)]));
        m.functions[1].custom_attributes[0].arguments = vec![AttributeArgument::Int32(512)];
        assert_eq!(
            analyze(&m, "Main").unwrap().types,
            BTreeMap::from([(3, 512), (4, 512)])
        );
    }

    #[test]
    fn unknown_flows_in_reflected_bodies_are_diagnosed() {
        let mut m = fixture();
        let mut constructor = function(
            "Model..ctor",
            vec![Type::Named("Info".into())],
            vec![Op::Arg(1), m.functions[0].body[2].clone(), Op::Return],
        );
        constructor.owner = Some(Type::Named("Model".into()));
        constructor.instance = true;
        m.functions.push(constructor);
        let report = analyze(&m, "Main").unwrap();
        assert_eq!(report.types, BTreeMap::from([(3, 3)]));
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.contains("DAM2001") && d.contains("Model..ctor"))
        );
    }
    #[test]
    fn explicit_body_seeds_infer_dependencies_without_granting_owner_members() {
        let mut m = fixture();
        let mut retained = m.functions[0].clone();
        retained.name = "ExplicitBody".into();
        retained.body[0] = Op::LoadTypeToken(Type::Named("Other".into()));
        m.functions.push(retained);
        m.functions[0].body = vec![Op::Return];
        assert!(analyze(&m, "Main").unwrap().types.is_empty());
        assert_eq!(
            analyze_with_roots(&m, "Main", &[3]).unwrap().types,
            BTreeMap::from([(4, 3)])
        );
        assert!(analyze_with_roots(&m, "Main", &[m.functions.len()]).is_err());
    }
    fn interface_fixture() -> Module {
        let mut m = fixture();
        let mut interface = crate::assemble(".module Extra\n.interface Sink\n.end\n")
            .unwrap()
            .types
            .remove(0);
        interface.definition = Some(crate::metadata::TypeDefId {
            module: m.name.clone(),
            revision: m.revision.clone(),
            index: m.types.len() as u32,
        });
        m.types.push(interface);
        m.types[3].is_reference_type = true;
        m.types[3].implements.push(Type::Named("Sink".into()));
        let mut contract = m.functions[1].clone();
        contract.definition = Some(crate::metadata::MemberId {
            module: m.name.clone(),
            revision: m.revision.clone(),
            index: 3,
        });
        contract.name = "Sink.Require".into();
        contract.owner = Some(Type::Named("Sink".into()));
        contract.instance = true;
        contract.is_abstract = true;
        contract.body.clear();
        let mut implementation = contract.clone();
        implementation.definition.as_mut().unwrap().index = 4;
        implementation.name = "Model.Require".into();
        implementation.owner = Some(Type::Named("Model".into()));
        implementation.is_abstract = false;
        implementation.body = vec![Op::Return];
        m.functions.extend([contract, implementation]);
        m
    }

    #[test]
    fn interface_parameter_requirements_must_match_exactly() {
        let mut m = interface_fixture();
        assert!(interface_contract_diagnostics(&m).unwrap().is_empty());
        for flags in [0, 1, 515] {
            m.functions[4].custom_attributes[0].arguments = vec![AttributeArgument::Int32(flags)];
            let diagnostics = interface_contract_diagnostics(&m).unwrap();
            assert_eq!(diagnostics.len(), 1);
            assert!(diagnostics.iter().next().unwrap().contains("DAM2005"));
        }
        m.functions[4].custom_attributes.clear();
        assert_eq!(interface_contract_diagnostics(&m).unwrap().len(), 1);
        m.functions[3].custom_attributes.clear();
        assert!(interface_contract_diagnostics(&m).unwrap().is_empty());
        m.functions[4].custom_attributes = m.functions[1].custom_attributes.clone();
        assert_eq!(interface_contract_diagnostics(&m).unwrap().len(), 1);
    }

    #[test]
    fn explicit_interface_mapping_uses_declared_identity() {
        let mut m = interface_fixture();
        let contract = &m.functions[3];
        let target: FunctionRef = serde_json::from_value(json!({"name":contract.name,
            "owner":contract.owner,"instance":true,"parameters":contract.parameters}))
        .unwrap();
        m.functions[4].name = "Model.PrivateImplementation".into();
        m.functions[4].visibility = crate::metadata::Visibility::Private;
        m.functions[4].interface_implementations.push(target);
        assert!(interface_contract_diagnostics(&m).unwrap().is_empty());
        m.functions[4].custom_attributes.clear();
        assert_eq!(interface_contract_diagnostics(&m).unwrap().len(), 1);
    }
    #[test]
    fn inherited_interface_contracts_are_checked_without_retaining_bodies() {
        let mut m = interface_fixture();
        let mut child = m.types[5].clone();
        child.name = "ChildSink".into();
        child.definition.as_mut().unwrap().index = 6;
        child.implements = vec![Type::Named("Sink".into())];
        m.types.push(child);
        m.types[3].implements = vec![Type::Named("ChildSink".into())];
        assert!(interface_contract_diagnostics(&m).unwrap().is_empty());
        m.functions[4].custom_attributes.clear();
        assert_eq!(interface_contract_diagnostics(&m).unwrap().len(), 1);
        m.functions[0].body = vec![Op::Return];
        assert!(analyze(&m, "Main").unwrap().types.is_empty());
    }
    fn override_fixture() -> Module {
        let mut m = fixture();
        m.types[3].is_reference_type = true;
        m.types[4].is_reference_type = true;
        m.types[3].base = Some(Type::Named("Other".into()));
        let mut contract = m.functions[1].clone();
        contract.name = "Other.Require".into();
        contract.owner = Some(Type::Named("Other".into()));
        contract.instance = true;
        contract.is_virtual = true;
        let mut implementation = contract.clone();
        implementation.name = "Model.Require".into();
        implementation.owner = Some(Type::Named("Model".into()));
        implementation.is_override = true;
        m.functions.extend([contract, implementation]);
        m
    }

    #[test]
    fn return_contracts_match_interface_and_override_declarations() {
        for (mut m, interface) in [(interface_fixture(), true), (override_fixture(), false)] {
            for f in &mut m.functions[3..5] {
                f.parameters.clear();
                f.origin.as_mut().unwrap().parameter_tokens.clear();
                f.origin.as_mut().unwrap().return_token = Some(134217729);
                f.returns = Type::Named("Info".into());
                f.no_result = false;
            }
            let check = if interface {
                interface_contract_diagnostics
            } else {
                override_contract_diagnostics
            };
            assert!(check(&m).unwrap().is_empty());
            for flags in [0, 1, 515] {
                m.functions[4].custom_attributes[0].arguments =
                    vec![AttributeArgument::Int32(flags)];
                let diagnostics = check(&m).unwrap();
                assert_eq!(diagnostics.len(), 1);
                assert!(
                    diagnostics
                        .iter()
                        .next()
                        .unwrap()
                        .contains("return requires")
                );
            }
            m.functions[4].custom_attributes.clear();
            assert_eq!(check(&m).unwrap().len(), 1);
        }
    }

    #[test]
    fn virtual_override_parameter_requirements_match_exactly() {
        let mut m = override_fixture();
        assert!(override_contract_diagnostics(&m).unwrap().is_empty());
        for flags in [0, 1, 515] {
            m.functions[4].custom_attributes[0].arguments = vec![AttributeArgument::Int32(flags)];
            let diagnostics = override_contract_diagnostics(&m).unwrap();
            assert_eq!(diagnostics.len(), 1);
            assert!(diagnostics.iter().next().unwrap().contains("DAM2006"));
        }
        m.functions[4].custom_attributes.clear();
        assert_eq!(override_contract_diagnostics(&m).unwrap().len(), 1);
        m.functions[3].custom_attributes.clear();
        assert!(override_contract_diagnostics(&m).unwrap().is_empty());
        m.functions[4].custom_attributes = m.functions[1].custom_attributes.clone();
        assert_eq!(override_contract_diagnostics(&m).unwrap().len(), 1);
    }

    #[test]
    fn overrides_compare_nearest_ancestor_including_abstract_declarations() {
        let mut m = override_fixture();
        let mut middle = m.types[4].clone();
        middle.name = "Middle".into();
        middle.definition.as_mut().unwrap().index = 5;
        middle.base = Some(Type::Named("Other".into()));
        middle.is_abstract = true;
        m.types.push(middle);
        m.types[3].base = Some(Type::Named("Middle".into()));
        let mut method = m.functions[4].clone();
        method.owner = Some(Type::Named("Middle".into()));
        method.name = "Middle.Require".into();
        method.is_abstract = true;
        method.body.clear();
        method.custom_attributes[0].arguments = vec![AttributeArgument::Int32(515)];
        m.functions[4].custom_attributes = method.custom_attributes.clone();
        m.functions.push(method);
        let diagnostics = override_contract_diagnostics(&m).unwrap();
        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics
                .iter()
                .next()
                .unwrap()
                .contains("Middle.Require")
        );
        assert!(!diagnostics.iter().next().unwrap().contains("Model.Require"));
        m.functions[0].body = vec![Op::Return];
        assert!(analyze(&m, "Main").unwrap().types.is_empty());
    }
    #[test]
    fn closed_generic_base_contracts_use_substituted_overload_signatures() {
        let mut m = override_fixture();
        m.types[4].generic_parameters = vec![Some("T".into())];
        m.functions[3].owner = Some(m.types[4].open_type());
        m.types[3].base = Some(Type::Constructed {
            definition: "Other".into(),
            arguments: vec![Type::Int32],
        });
        m.functions[3].parameters.push(Type::TypeParameter(0));
        m.functions[4].parameters.push(Type::Int32);
        let mut overload = m.functions[3].clone();
        overload.parameters[1] = Type::String;
        overload.custom_attributes[0].arguments = vec![AttributeArgument::Int32(512)];
        m.functions.push(overload);
        assert!(override_contract_diagnostics(&m).unwrap().is_empty());
        m.functions[4].custom_attributes[0].arguments = vec![AttributeArgument::Int32(1)];
        let diagnostics = override_contract_diagnostics(&m).unwrap();
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics.iter().next().unwrap().contains("requires 3"));
        // The String overload has its own annotation; do not select by name alone.
        m.functions[4].parameters[1] = Type::String;
        m.functions[4].custom_attributes[0].arguments = vec![AttributeArgument::Int32(512)];
        assert!(override_contract_diagnostics(&m).unwrap().is_empty());
    }

    #[test]
    fn closed_generic_ancestry_composes_substitutions_across_levels() {
        let mut m = override_fixture();
        m.types[4].generic_parameters = vec![Some("T".into())];
        m.functions[3].owner = Some(m.types[4].open_type());
        let mut middle = m.types[4].clone();
        middle.name = "Middle".into();
        middle.definition.as_mut().unwrap().index = 5;
        middle.base = Some(Type::Constructed {
            definition: "Other".into(),
            arguments: vec![Type::ArrayRef(Box::new(Type::TypeParameter(0)))],
        });
        m.types.push(middle);
        m.types[3].base = Some(Type::Constructed {
            definition: "Middle".into(),
            arguments: vec![Type::Int32],
        });
        m.functions[3].parameters.push(Type::TypeParameter(0));
        m.functions[4]
            .parameters
            .push(Type::ArrayRef(Box::new(Type::Int32)));
        assert!(override_contract_diagnostics(&m).unwrap().is_empty());
        m.functions[4].custom_attributes.clear();
        assert_eq!(override_contract_diagnostics(&m).unwrap().len(), 1);
    }
}

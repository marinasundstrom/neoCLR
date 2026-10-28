//! Conservative native retention analysis for type-bearing parameter contracts.
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

fn requirements(module: &Module, function: &Function) -> Result<Vec<u32>, Fault> {
    let mut result = vec![0; function.parameters.len()];
    for attribute in &function.custom_attributes {
        if attribute
            .constructor
            .owner
            .as_ref()
            .and_then(|t| source_name(module, t))
            != Some(ATTRIBUTE)
        {
            continue;
        }
        let fail = || {
            Fault::new(format!(
                "DAM1001: unsupported DynamicallyAccessedMembers contract on {}",
                function.name
            ))
        };
        let token = attribute.target_token.ok_or_else(fail)?;
        let position = function
            .origin
            .as_ref()
            .and_then(|o| o.parameter_tokens.iter().position(|t| *t == token))
            .ok_or_else(fail)?;
        if source_name(module, function.parameters.get(position).ok_or_else(fail)?)
            != Some("System.Introspection.TypeInfo")
        {
            return Err(fail());
        }
        let [AttributeArgument::Int32(flags)] = attribute.arguments.as_slice() else {
            return Err(fail());
        };
        if *flags < 0 || (*flags as u32 & !SUPPORTED) != 0 || !attribute.named_arguments.is_empty()
        {
            return Err(fail());
        }
        result[position] |= *flags as u32;
    }
    Ok(result)
}

/// Diagnose parameter annotation mismatches in verified, source-identity metadata.
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
            .flatten()
            .any(|f| *f != 0)
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
                    for (index, (expected, found)) in declared.iter().zip(&actual).enumerate() {
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

/// Check nongeneric nominal reference-class override parameter contracts.
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
            let chain = crate::inheritance::lineage(module, owner)?;
            if chain.iter().any(|base| {
                !matches!(base, Type::Named(_))
                    || module
                        .type_definition(base)
                        .is_some_and(|t| !t.generic_parameters.is_empty())
            }) {
                continue;
            }
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
            for (index, (expected, found)) in declared.iter().zip(&actual).enumerate() {
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
/// parameter contracts for public constructors and properties. Other targets/flags
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
    for ty in &module.types {
        if ty.custom_attributes.iter().any(|a| {
            a.constructor
                .owner
                .as_ref()
                .and_then(|t| source_name(module, t))
                == Some(ATTRIBUTE)
        }) {
            return Err(Fault::new(
                "DAM1001: only TypeInfo parameter annotations are currently supported",
            ));
        }
    }
    let contracts = module
        .functions
        .iter()
        .map(|f| requirements(module, f))
        .collect::<Result<Vec<_>, _>>()?;
    if contracts.iter().all(|c| c.iter().all(|f| *f == 0)) {
        return Ok(report);
    }
    let mut reachable = BTreeSet::new();
    let mut pending: Vec<_> = roots.iter().copied().collect();
    while let Some(index) = pending.pop() {
        if !reachable.insert(index) {
            continue;
        }
        for op in &module.functions[index].body {
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
        if function.body.is_empty() {
            continue;
        }
        let mut arguments = vec![Value::Unknown; usize::from(function.instance)];
        arguments.extend(contract.iter().copied().map(Value::Required));
        let initial = State {
            stack: vec![],
            locals: vec![Value::Unknown; function.locals.len()],
            arguments,
        };
        let mut states = vec![None; function.body.len()];
        states[0] = Some(initial);
        let mut queue = VecDeque::from([0]);
        let mut relevant = false;
        for op in &function.body {
            if let Op::Call(t)
            | Op::CallVirtual(t)
            | Op::CallSelf { target: t, .. }
            | Op::Construct(t)
            | Op::BindFunction { target: t, .. } = op
            {
                let callee = crate::vm::resolve(module, t)?;
                relevant |= requirements(module, &callee)?.iter().any(|r| *r != 0);
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
            let (pops, pushes) = effects[pc]
                .ok_or_else(|| Fault::new("DAM analysis reached an unverified instruction"))?;
            if state.stack.len() < pops {
                return Err(Fault::new("DAM analysis stack underflow"));
            }
            let values = state.stack.split_off(state.stack.len() - pops);
            let mut produced = vec![Value::Unknown; pushes];
            match op {
                Op::LoadTypeToken(ty) => produced[0] = Value::Types(vec![ty.clone()]),
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
                    for (index, flags) in requirements(module, &callee)?.into_iter().enumerate() {
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
                Op::BindFunction { target, .. } => {
                    let callee = crate::vm::resolve(module, target)?;
                    if requirements(module, &callee)?.iter().any(|r| *r != 0) {
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
}

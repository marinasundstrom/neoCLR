use crate::{
    ExecutionOptions, Fault, Module, Value,
    metadata::{FunctionRef, Instruction as Op, Representation, Type},
};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub instructions: usize,
    pub frames: usize,
    pub stack: usize,
    /// Maximum simultaneously live managed heap objects, checked after collection.
    pub heap_objects: usize,
    /// Logical array payload budget across active frames and managed heap.
    pub array_elements: usize,
    pub array_bytes: usize,
    pub pointer_bytes: usize,
    pub pointer_allocations: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            instructions: 100_000,
            frames: 256,
            stack: 4096,
            heap_objects: 4096,
            array_elements: 65_536,
            array_bytes: 16 * 1024 * 1024,
            pointer_bytes: 16 * 1024 * 1024,
            pointer_allocations: 4096,
        }
    }
}

#[derive(Debug)]
pub struct Execution {
    pub value: Value,
    /// Captured lines when no host console is supplied; empty with live console I/O.
    pub output: Vec<String>,
    /// Objects reachable from the returned value; identities may contain gaps.
    pub heap: crate::ManagedHeap,
    pub memory: crate::memory::PointerHeap,
    /// Retains libraries for the lifetime of returned native addresses.
    pub native_libraries: Option<crate::interop::NativeLibraries>,
}

pub(crate) fn resolve(
    module: &Module,
    target: &FunctionRef,
) -> Result<crate::metadata::Function, Fault> {
    let mut found = None;
    for (index, definition) in module.functions.iter().enumerate() {
        let identity = definition
            .definition
            .clone()
            .unwrap_or_else(|| crate::metadata::MemberId {
                module: module.name.clone(),
                revision: module.revision.clone(),
                index: index as u32,
            });
        if target
            .definition
            .as_ref()
            .is_some_and(|wanted| wanted != &identity)
        {
            continue;
        }
        if definition.name != target.name || definition.instance != target.instance {
            continue;
        }
        let arity = definition
            .owner
            .as_ref()
            .and_then(|o| module.type_definition(o))
            .map_or(0, |d| d.generic_parameters.len());
        let mut candidate = if arity > 0 {
            let Some(Type::Constructed {
                definition: owner,
                arguments,
            }) = &target.owner
            else {
                continue;
            };
            if definition.owner.as_ref().and_then(Type::definition_name) != Some(owner.as_str())
                || arguments.len() != arity
            {
                continue;
            }
            let mut instantiated =
                definition.map_types(|ty| ty.substitute_type_parameters(arguments))?;
            instantiated.owner = target.owner.clone();
            instantiated
        } else {
            if target
                .owner
                .as_ref()
                .is_some_and(|owner| definition.owner.as_ref() != Some(owner))
            {
                continue;
            }
            definition.clone()
        };
        candidate.definition = Some(identity);
        if candidate.parameters == target.parameters {
            if found.is_some() {
                return Err(Fault::new(
                    "ambiguous function overload after type substitution",
                ));
            }
            found = Some(candidate);
        }
    }
    found.ok_or_else(|| {
        Fault::new(format!(
            "unknown function overload {}({:?})",
            target.name, target.parameters
        ))
    })
}

pub(crate) fn record_fields(
    module: &Module,
    ty: &Type,
    arity: usize,
) -> Result<Vec<crate::metadata::Field>, Fault> {
    check_type_context(ty, module, arity, 0)?;
    let (name, arguments): (&str, &[Type]) = match ty {
        Type::Named(name) => (name, &[]),
        Type::Constructed {
            definition,
            arguments,
        } => (definition, arguments),
        _ => return Err(Fault::new("expected record type reference")),
    };
    let def = module
        .types
        .iter()
        .find(|d| {
            d.name == name
                && d.generic_parameters.len() == arguments.len()
                && d.representation == Representation::Record
        })
        .ok_or_else(|| Fault::new("expected record definition"))?;
    def.fields
        .iter()
        .map(|f| {
            Ok(crate::metadata::Field {
                visibility: f.visibility,
                name: f.name.clone(),
                ty: f.ty.substitute_type_parameters(arguments)?,
            })
        })
        .collect()
}

pub(crate) fn resolve_constructor(
    module: &Module,
    target: &FunctionRef,
) -> Result<crate::metadata::Function, Fault> {
    let function = resolve(module, target)?;
    let owner = target
        .owner
        .as_ref()
        .ok_or_else(|| Fault::new("constructor requires an explicit owner"))?;
    let name = match owner {
        Type::Constructed { definition, .. } => definition.as_str(),
        _ => owner
            .definition_name()
            .ok_or_else(|| Fault::new("constructor requires a record owner"))?,
    };
    if !target.instance
        || function.name != format!("{name}..ctor")
        || function.returns != Type::Void
        || function.is_internal_call()
        || function.pinvoke.is_some()
        || !module
            .type_definition(owner)
            .is_some_and(|d| d.representation == Representation::Record)
    {
        return Err(Fault::new(
            "newobj constructor requires an instance IL .ctor returning Void on a record type",
        ));
    }
    Ok(function)
}

pub(crate) fn validate(module: &Module) -> Result<(), Fault> {
    if module.name == "System" {
        crate::references::validate_list(module, &[module])?;
        if !module.entry.is_empty() {
            return Err(Fault::new(
                "System is reserved for the runtime library without an entry point",
            ));
        }
        let mut normalized = module.clone();
        normalized.normalize_definition_ids()?;
        let normalized = crate::scope::normalize_module(&normalized, &normalized)?;
        validate_linked(&normalized)
    } else {
        crate::library::link(module, crate::library::system()?).map(|_| ())
    }
}

pub(crate) fn validate_linked(module: &Module) -> Result<(), Fault> {
    if module.name.is_empty() {
        return Err(Fault::new("module name must not be empty"));
    }
    if module.format != 5 {
        return Err(Fault::new("unsupported module format (expected 5)"));
    }
    // Validate ownership before any traversal by access checking or execution.
    for def in &module.types {
        let mut current = def;
        let mut seen = HashSet::new();
        while let Some(owner) = &current.declaring_type {
            if !seen.insert(owner) || seen.len() > 32 {
                return Err(Fault::new("cyclic or excessive type ownership nesting"));
            }
            let parent = module
                .types
                .iter()
                .find(|candidate| candidate.definition.as_ref() == Some(owner))
                .ok_or_else(|| Fault::new("missing nested type owner"))?;
            if current
                .definition
                .as_ref()
                .is_none_or(|id| id.module != owner.module || id.revision != owner.revision)
            {
                return Err(Fault::new(
                    "nested type owner must belong to the same module and revision",
                ));
            }
            if !parent.generic_parameters.is_empty() {
                return Err(Fault::new(
                    "nesting under generic types is not supported yet",
                ));
            }
            let prefix = format!("{}.", parent.name);
            if current
                .name
                .strip_prefix(&prefix)
                .is_none_or(|name| name.is_empty() || name.contains('.'))
            {
                return Err(Fault::new(
                    "nested type name must qualify its immediate owner",
                ));
            }
            current = parent;
        }
    }
    let mut names = HashSet::new();
    let mut type_identities = HashSet::new();
    for def in &module.types {
        if def.visibility == crate::metadata::Visibility::Private {
            return Err(Fault::new(
                "types currently support public or internal visibility",
            ));
        }
        if def
            .definition
            .as_ref()
            .is_some_and(|id| !type_identities.insert(id))
        {
            return Err(Fault::new("duplicate type definition identity"));
        }
        if def.name.is_empty() || !names.insert((&def.name, def.generic_parameters.len())) {
            return Err(Fault::new("empty or duplicate type name"));
        }
        let ty = Type::from_name(&def.name);
        if ty.definition_name() != Some(def.name.as_str()) {
            return Err(Fault::new("type definitions must use canonical names"));
        }
        if (def.representation == Representation::Runtime) != ty.is_primitive() {
            return Err(Fault::new(
                "runtime representation is reserved for canonical primitive types",
            ));
        }
        if def.representation == Representation::Runtime && !def.fields.is_empty() {
            return Err(Fault::new(
                "runtime primitive types cannot declare record fields",
            ));
        }
        if def.generic_parameters.len() > u16::MAX as usize + 1 {
            return Err(Fault::new("too many type parameters"));
        }
        let mut parameter_names = HashSet::new();
        for name in def.generic_parameters.iter().flatten() {
            if !crate::metadata::valid_slot_name(name)
                || Type::from_name(name).is_primitive()
                || !parameter_names.insert(name)
            {
                return Err(Fault::new("invalid or duplicate type parameter name"));
            }
        }
        if !def.generic_parameters.is_empty()
            && (!matches!(
                def.representation,
                Representation::Record | Representation::Interface
            ) || matches!(
                def.name.as_str(),
                "Option" | "Result" | "Ref" | "Ptr" | "InterfaceRef"
            ))
        {
            return Err(Fault::new(
                "reserved type cannot declare generic parameters",
            ));
        }
        if def.packing.is_some() || def.minimum_size.is_some() {
            if def.representation != Representation::Record {
                return Err(Fault::new("runtime types cannot override layout"));
            }
            if def.generic_parameters.is_empty() {
                crate::memory::layout(module, &ty)?;
            } else if def
                .packing
                .is_some_and(|p| !matches!(p, 0 | 1 | 2 | 4 | 8 | 16 | 32 | 64 | 128))
                || def.minimum_size.is_some_and(|s| s > i32::MAX as u32)
            {
                return Err(Fault::new("invalid generic record layout controls"));
            }
        }
        for interface in &def.implements {
            check_type_context(interface, module, def.generic_parameters.len(), 0)?;
        }
        let mut fields = HashSet::new();
        for field in &def.fields {
            if field.name.is_empty() || !fields.insert(&field.name) {
                return Err(Fault::new("empty or duplicate field name"));
            }
            check_type_context(&field.ty, module, def.generic_parameters.len(), 0)?;
        }
    }
    let mut identities = HashSet::new();
    let mut access_signatures = HashSet::new();
    let mut signatures = HashSet::new();
    let free_signatures: HashSet<_> = module
        .functions
        .iter()
        .filter(|function| function.owner.is_none())
        .map(|function| (&function.name, &function.parameters, function.instance))
        .collect();
    for function in &module.functions {
        if function
            .definition
            .as_ref()
            .is_some_and(|identity| !identities.insert(identity))
        {
            return Err(Fault::new("duplicate function definition identity"));
        }
        let parameters_without_access: Vec<_> = function
            .parameters
            .iter()
            .map(|ty| match ty {
                Type::ReadOnlyByRef(target) => Type::ByRef(target.clone()),
                other => other.clone(),
            })
            .collect();
        if !access_signatures.insert((
            &function.owner,
            &function.name,
            function.instance,
            parameters_without_access,
        )) {
            return Err(Fault::new(
                "duplicate function signature differing only by reference access",
            ));
        }
        if function.name.is_empty()
            || (function.owner.is_some()
                && free_signatures.contains(&(
                    &function.name,
                    &function.parameters,
                    function.instance,
                )))
            || !signatures.insert((
                &function.owner,
                &function.name,
                &function.parameters,
                function.instance,
            ))
        {
            return Err(Fault::new(
                "empty name, duplicate or reserved function signature",
            ));
        }
    }
    for function in &module.functions {
        crate::metadata::validate_slot_names(
            &function.parameter_names,
            function.parameters.len(),
            function.instance,
        )?;
        crate::metadata::validate_slot_names(&function.local_names, function.locals.len(), false)?;
        let mut previous = None;
        for point in &function.sequence_points {
            if point.instruction >= function.body.len()
                || point.line == 0
                || point.column == 0
                || point.document.is_empty()
                || point.document.len() > 4096
                || point.document.chars().any(char::is_control)
                || previous.is_some_and(|pc| pc >= point.instruction)
            {
                return Err(Fault::new("invalid source sequence point"));
            }
            previous = Some(point.instruction);
        }

        if function.instance && function.owner.is_none() {
            return Err(Fault::new("instance method requires a declaring type"));
        }
        if function.visibility == crate::metadata::Visibility::Private && function.owner.is_none() {
            return Err(Fault::new(
                "private requires a declaring type; use internal for a module function",
            ));
        }
        let arity = function
            .owner
            .as_ref()
            .and_then(|owner| module.type_definition(owner))
            .map_or(0, |d| d.generic_parameters.len());
        let check = |ty: &Type| check_type_context(ty, module, arity, 0);
        if let Some(owner) = &function.owner {
            if arity == 0 {
                check_type(owner, module)?;
            }
            let def = module
                .type_definition(owner)
                .ok_or_else(|| Fault::new("method owner has no type definition"))?;
            if owner != &def.open_type() {
                return Err(Fault::new("method owner must be its open type definition"));
            }
            if matches!((&def.definition, &function.definition), (Some(ty), Some(method)) if ty.module != method.module)
            {
                return Err(Fault::new(
                    "method and declaring type must belong to the same module",
                ));
            }
            let prefix = format!("{}.", def.name);
            let member = function
                .name
                .strip_prefix(&prefix)
                .ok_or_else(|| Fault::new("method name does not match its declaring type"))?;
            if member.is_empty() || (member.contains('.') && member != ".ctor") {
                return Err(Fault::new("invalid member name"));
            }
            if member == ".ctor" && (!function.instance || function.returns != Type::Void) {
                return Err(Fault::new(
                    ".ctor requires an instance method returning Void",
                ));
            }
        }
        if (function.is_internal_call() || function.pinvoke.is_some())
            && function
                .parameters
                .iter()
                .chain([&function.returns])
                .any(crate::slots::contains)
        {
            return Err(Fault::new(
                "managed references cannot cross native helper boundaries",
            ));
        }
        if function.receiver_byref
            && (!function.instance
                || function.name.ends_with(".ctor")
                || function.is_internal_call()
                || function.pinvoke.is_some())
        {
            return Err(Fault::new(
                "byref receiver requires a non-constructor IL instance method",
            ));
        }
        if function.receiver_readonly && !function.receiver_byref {
            return Err(Fault::new("readonly receiver requires a byref receiver"));
        }
        let mut readonly_parameters = HashSet::new();
        for index in &function.readonly_parameters {
            if !readonly_parameters.insert(*index)
                || !matches!(function.parameters.get(*index), Some(Type::ByRef(_)))
                || function.out_parameters.contains(index)
                || function.out_when_true.contains(index)
                || function.is_internal_call()
                || function.pinvoke.is_some()
            {
                return Err(Fault::new("invalid readonly parameter contract"));
            }
        }
        let mut out_parameters = HashSet::new();
        if !function.out_when_true.is_empty() && function.returns != Type::Boolean {
            return Err(Fault::new("conditional output requires a Boolean return"));
        }
        for index in function
            .out_parameters
            .iter()
            .chain(&function.out_when_true)
        {
            if !out_parameters.insert(*index)
                || !matches!(function.parameters.get(*index), Some(Type::ByRef(_)))
            {
                return Err(Fault::new(
                    "out contract requires a unique byref parameter index",
                ));
            }
        }
        for ty in function
            .parameters
            .iter()
            .chain(&function.locals)
            .chain([&function.returns])
        {
            check(ty)?;
        }
        if arity > 0 && (function.pinvoke.is_some() || function.impl_flags != 0) {
            return Err(Fault::new("generic owners require IL methods"));
        }
        if crate::interfaces::is_contract(module, function) {
            crate::interfaces::validate_contract(function)?;
            continue;
        }
        if function.pinvoke.is_some() {
            crate::interop::validate(function)?;
            continue;
        }
        match function.impl_flags {
            crate::metadata::INTERNAL_CALL => {
                if !function.body.is_empty() || !function.locals.is_empty() {
                    return Err(Fault::new(
                        "InternalCall must not have an IL body or locals",
                    ));
                }
                crate::native::bind(function)?;
                continue;
            }
            0 => (),
            _ => return Err(Fault::new("unsupported method implementation flags")),
        }
        if function.body.is_empty() {
            return Err(Fault::new("empty function body"));
        }
        let valid_target = |index: usize| {
            index < function.body.len()
                && (index == 0 || !matches!(function.body[index - 1], Op::Unaligned(_)))
        };
        for (pc, op) in function.body.iter().enumerate() {
            match op {
                Op::Unaligned(alignment) => {
                    if !matches!(alignment, 1 | 2 | 4) {
                        return Err(Fault::new("unaligned. requires alignment 1, 2, or 4"));
                    }
                    if !function.body.get(pc + 1).is_some_and(Op::accepts_unaligned) {
                        return Err(Fault::new(
                            "unaligned. must immediately precede a supported memory access",
                        ));
                    }
                }
                Op::Branch(i)
                | Op::BranchTrue(i)
                | Op::BranchFalse(i)
                | Op::BranchEqual(i)
                | Op::BranchNotEqual(i)
                | Op::BranchGreater(i)
                | Op::BranchGreaterUnsigned(i)
                | Op::BranchLess(i)
                | Op::BranchLessUnsigned(i)
                | Op::BranchGreaterEqual(i)
                | Op::BranchGreaterEqualUnsigned(i)
                | Op::BranchLessEqual(i)
                | Op::BranchLessEqualUnsigned(i)
                    if !valid_target(*i) =>
                {
                    return Err(Fault::new(
                        "branch outside function or into prefixed instruction",
                    ));
                }
                Op::Switch(targets) if targets.iter().any(|i| !valid_target(*i)) => {
                    return Err(Fault::new(
                        "switch target outside function or into prefixed instruction",
                    ));
                }
                Op::Arg(i) | Op::StoreArg(i) | Op::ArgumentAddress(i)
                    if *i >= function.argument_types().len() =>
                {
                    return Err(Fault::new("argument index outside signature"));
                }
                Op::Load(i) | Op::Store(i) | Op::ResetLocal(i) | Op::LocalAddress(i)
                    if *i >= function.locals.len() =>
                {
                    return Err(Fault::new("local index outside signature"));
                }
                Op::LocalAddress(i)
                    if matches!(
                        &function.locals[*i],
                        Type::ByRef(_) | Type::ReadOnlyByRef(_)
                    ) =>
                {
                    return Err(Fault::new(
                        "cannot take the address of a managed reference local",
                    ));
                }
                Op::ArgumentAddress(i) | Op::StoreArg(i)
                    if matches!(
                        &function.argument_types()[*i],
                        Type::ByRef(_) | Type::ReadOnlyByRef(_)
                    ) =>
                {
                    return Err(Fault::new(
                        "cannot rebind or take the address of a managed reference parameter",
                    ));
                }
                Op::Call(target) | Op::CallVirtual(target) | Op::Construct(target) => {
                    if let Some(owner) = &target.owner {
                        check(owner)?;
                    }
                    for ty in &target.parameters {
                        check(ty)?;
                    }
                    let callee = resolve(module, target)?;
                    if matches!(op, Op::CallVirtual(_))
                        != crate::interfaces::is_contract(module, &callee)
                    {
                        return Err(Fault::new(
                            "interface declarations require callvirt; class virtual calls are not supported",
                        ));
                    }
                    if matches!(op, Op::Construct(_)) {
                        resolve_constructor(module, target)?;
                    }
                    crate::access::check_call(module, Some(function), &callee)?;
                }
                Op::AllocateArray(ty)
                | Op::NewArray(ty)
                | Op::CreateArray(ty)
                | Op::ArrayElement(ty)
                | Op::StoreArrayElement(ty)
                | Op::ArrayAddress(ty) => {
                    check(&Type::Array(Box::new(ty.clone())))?;
                    if matches!(op, Op::ArrayAddress(_))
                        && matches!(ty, Type::ByRef(_) | Type::ReadOnlyByRef(_))
                    {
                        return Err(Fault::new("nested managed references are not supported"));
                    }
                    if matches!(op, Op::NewArray(_)) && check_type(ty, module).is_ok() {
                        crate::initialization::default_value(module, ty)?;
                    }
                }
                Op::New(ty) => {
                    record_fields(module, ty, arity)?;
                    crate::access::check_construction(module, function, ty)?;
                }
                Op::SizeOf(ty) | Op::AlignOf(ty) | Op::Allocate(ty) | Op::CopyObject(ty) => {
                    check(ty)?;
                    if check_type(ty, module).is_ok() {
                        crate::memory::layout(module, ty)?;
                    }
                }
                Op::InitializeObject(ty) => {
                    check(ty)?;
                    if check_type(ty, module).is_ok() {
                        crate::initialization::default_value(module, ty)?;
                    }
                }
                Op::LoadObject(ty) | Op::StoreObject(ty) => {
                    check(ty)?;
                    if crate::interfaces::interface_definition(module, ty).is_ok() {
                        return Err(Fault::new(
                            "interface views support dispatch, not value storage",
                        ));
                    }
                    if matches!(ty, Type::ByRef(_) | Type::ReadOnlyByRef(_)) {
                        return Err(Fault::new("managed references cannot be indirectly stored"));
                    }
                }
                Op::BorrowInterface(ty) => {
                    check(ty)?;
                    crate::interfaces::interface_definition(module, ty)?;
                }
                Op::LoadTypeToken(ty)
                | Op::PackValue(ty)
                | Op::IsValue(ty)
                | Op::UnpackValue(ty)
                | Op::NullPointer(ty)
                | Op::PointerCast(ty)
                | Op::PointerFromInt(ty) => check(ty)?,
                _ => (),
            }
        }
    }
    for definition in &module.types {
        let arity = definition.generic_parameters.len();
        let owner = if arity == 0 {
            Type::from_name(&definition.name)
        } else {
            Type::Constructed {
                definition: definition.name.clone(),
                arguments: (0..arity)
                    .map(|index| Type::TypeParameter(index as u16))
                    .collect(),
            }
        };
        let mut signatures = HashSet::new();
        for property in &definition.properties {
            if !crate::metadata::valid_slot_name(&property.name)
                || !signatures.insert((&property.name, property.instance, &property.parameters))
            {
                return Err(Fault::new("invalid or duplicate property signature"));
            }
            check_type_context(&property.ty, module, arity, 0)?;
            for parameter in &property.parameters {
                check_type_context(parameter, module, arity, 0)?;
            }
            if property.getter.is_none() && property.setter.is_none() {
                return Err(Fault::new("property requires an accessor"));
            }
            for (target, setter) in property
                .getter
                .iter()
                .map(|t| (t, false))
                .chain(property.setter.iter().map(|t| (t, true)))
            {
                if target.owner.as_ref() != Some(&owner) || target.instance != property.instance {
                    return Err(Fault::new(
                        "property accessor owner or instance kind mismatch",
                    ));
                }
                let mut parameters = property.parameters.clone();
                if setter {
                    parameters.push(property.ty.clone());
                }
                if target.parameters != parameters {
                    return Err(Fault::new("property accessor parameter mismatch"));
                }
                let accessor = resolve(module, target)?;
                if accessor.returns
                    != if setter {
                        Type::Void
                    } else {
                        property.ty.clone()
                    }
                {
                    return Err(Fault::new("property accessor return type mismatch"));
                }
                if definition
                    .definition
                    .as_ref()
                    .zip(accessor.definition.as_ref())
                    .is_some_and(|(ty, method)| {
                        ty.module != method.module || ty.revision != method.revision
                    })
                {
                    return Err(Fault::new(
                        "property accessor must belong to the declaring module",
                    ));
                }
            }
        }
    }
    for attributes in module
        .types
        .iter()
        .map(|d| &d.custom_attributes)
        .chain(module.functions.iter().map(|f| &f.custom_attributes))
    {
        for attribute in attributes {
            validate_attribute(module, attribute)?;
        }
    }
    crate::access::validate_types(module)?;
    if let Some(entry) = module
        .functions
        .iter()
        .find(|f| f.name == module.entry && f.parameters.is_empty() && !f.instance)
    {
        crate::access::check_entry(module, entry)?;
    }
    crate::interfaces::validate(module)?;
    if !module.entry.is_empty()
        && !module.functions.iter().any(|f| {
            f.name == module.entry
                && f.parameters.is_empty()
                && !f.is_internal_call()
                && f.pinvoke.is_none()
                && !f.instance
                && f.owner
                    .as_ref()
                    .and_then(|o| module.type_definition(o))
                    .is_none_or(|d| d.generic_parameters.is_empty())
        })
    {
        return Err(Fault::new("parameterless entry function not found"));
    }
    Ok(())
}

pub(crate) fn check_type(ty: &Type, module: &Module) -> Result<(), Fault> {
    check_type_context(ty, module, 0, 0)
}

fn check_type_context(ty: &Type, module: &Module, arity: usize, depth: usize) -> Result<(), Fault> {
    if depth > 32 {
        return Err(Fault::new("type nesting exceeds 32"));
    }
    let nested = |ty: &Type| check_type_context(ty, module, arity, depth + 1);
    match ty {
        Type::Scoped { .. } => Err(Fault::new("unresolved scoped type signature")),
        Type::TypeParameter(index) if *index as usize >= arity => {
            Err(Fault::new("type parameter outside declaring context"))
        }
        Type::Named(name) => {
            if Type::from_name(name).is_primitive() {
                return Err(Fault::new(
                    "primitive type requires canonical primitive signature encoding",
                ));
            }
            let def = module
                .types
                .iter()
                .find(|t| &t.name == name && t.generic_parameters.is_empty())
                .ok_or_else(|| Fault::new(format!("unknown type {name}")))?;
            if !def.generic_parameters.is_empty() {
                return Err(Fault::new("generic type requires type arguments"));
            }
            Ok(())
        }
        Type::Constructed {
            definition,
            arguments,
        } => {
            let def = module
                .types
                .iter()
                .find(|t| &t.name == definition && t.generic_parameters.len() == arguments.len())
                .ok_or_else(|| Fault::new(format!("unknown generic type {definition}")))?;
            if def.generic_parameters.is_empty() || def.generic_parameters.len() != arguments.len()
            {
                return Err(Fault::new("generic type argument count mismatch"));
            }
            for argument in arguments {
                nested(argument)?;
            }
            Ok(())
        }
        Type::Array(t) => {
            nested(t)?;
            if crate::interfaces::interface_definition(module, t).is_ok() {
                return Err(Fault::new("array elements require concrete values"));
            }
            Ok(())
        }
        Type::InterfaceRef(t) => {
            nested(t)?;
            crate::interfaces::interface_definition(module, t)?;
            Ok(())
        }
        Type::ByRef(t) | Type::ReadOnlyByRef(t) => {
            if matches!(t.as_ref(), Type::ByRef(_) | Type::ReadOnlyByRef(_)) {
                return Err(Fault::new("nested managed references are not supported"));
            }
            nested(t)
        }
        Type::Ptr(t) => {
            if crate::slots::contains(t) {
                return Err(Fault::new(
                    "managed references cannot be stored in pointer or owner types",
                ));
            }
            nested(t)
        }
        _ => Ok(()),
    }
}

fn restrict_reference_arguments(
    function: &crate::metadata::Function,
    args: &mut [Value],
) -> Result<(), Fault> {
    let offset = usize::from(function.instance);
    for (index, value) in args.iter_mut().enumerate() {
        let reference = match value {
            Value::SlotReference(reference)
            | Value::SlotInterface {
                receiver: reference,
                ..
            } => reference,
            _ => continue,
        };
        if (index == 0 && function.receiver_readonly)
            || (index >= offset
                && (function.readonly_parameters.contains(&(index - offset))
                    || matches!(
                        function.parameters.get(index - offset),
                        Some(Type::ReadOnlyByRef(_))
                    )))
        {
            reference.assigned()?;
            reference.restrict_readonly();
        } else {
            reference.require_writable()?;
        }
    }
    Ok(())
}

struct Frame {
    function: std::rc::Rc<crate::metadata::Function>,
    pc: usize,
    trace_pc: usize,
    args: Vec<crate::slots::Cell>,
    constructing: bool,
    locals: Vec<crate::slots::Cell>,
    stack: Vec<Value>,
    allocations: Vec<crate::memory::Pointer>,
    outputs: Vec<(crate::SlotReference, bool)>,
}

impl Frame {
    fn new(function: crate::metadata::Function, mut args: Vec<Value>) -> Result<Self, Fault> {
        restrict_reference_arguments(&function, &mut args)?;
        let offset = args.len().saturating_sub(function.parameters.len());
        let mut outputs = vec![];
        for (index, arg) in args.iter_mut().enumerate() {
            if let Value::SlotInterface { receiver, .. } = arg {
                if index >= offset
                    && (function.out_parameters.contains(&(index - offset))
                        || function.out_when_true.contains(&(index - offset)))
                {
                    return Err(Fault::new(
                        "an interface view is not an output storage slot",
                    ));
                }
                receiver.assigned()?;
            }
            if let Value::SlotReference(reference) = arg {
                if index >= offset
                    && (function.out_parameters.contains(&(index - offset))
                        || function.out_when_true.contains(&(index - offset)))
                {
                    *reference = reference.output()?;
                    outputs.push((
                        reference.clone(),
                        function.out_when_true.contains(&(index - offset)),
                    ));
                } else {
                    reference.assigned()?;
                }
            }
        }
        let locals = function
            .locals
            .iter()
            .map(|ty| crate::slots::Slot::new(ty.clone(), None))
            .collect();
        Ok(Self {
            function: std::rc::Rc::new(function),
            pc: 0,
            trace_pc: 0,
            args: args
                .into_iter()
                .map(|v| crate::slots::Slot::new(v.ty(), Some(v)))
                .collect(),
            constructing: false,
            locals,
            stack: vec![],
            allocations: vec![],
            outputs,
        })
    }
    fn check_reference_return(&self, value: &Value) -> Result<(), Fault> {
        let reference = match value {
            Value::SlotReference(reference) => reference,
            Value::SlotInterface { receiver, .. } => receiver,
            _ => return Ok(()),
        };
        if self
            .args
            .iter()
            .chain(&self.locals)
            .any(|cell| reference.addresses(cell))
        {
            return Err(Fault::new(
                "cannot return a managed reference to the current frame",
            ));
        }
        Ok(())
    }

    fn pop(&mut self) -> Result<Value, Fault> {
        self.stack
            .pop()
            .ok_or_else(|| Fault::new("evaluation stack underflow"))
    }
    fn pointer(&mut self) -> Result<crate::memory::Pointer, Fault> {
        match self.pop()? {
            Value::Pointer(p) => Ok(p),
            _ => Err(Fault::new("expected Ptr")),
        }
    }
    fn args(&mut self, types: &[Type]) -> Result<Vec<Value>, Fault> {
        if self.stack.len() < types.len() {
            return Err(Fault::new("not enough arguments"));
        }
        let args = self.stack.split_off(self.stack.len() - types.len());
        args.into_iter()
            .zip(types)
            .map(|(value, ty)| value.for_storage(ty))
            .collect()
    }
}

// Stored and returned references must be readable. Uninitialized capabilities
// remain confined to direct output-argument paths in this implementation slice.
fn assigned_reference(value: &Value) -> Result<(), Fault> {
    match value {
        Value::SlotReference(reference) => reference.assigned(),
        Value::SlotInterface { receiver, .. } => receiver.assigned(),
        _ => Ok(()),
    }
}

fn expect(value: &Value, ty: &Type) -> Result<(), Fault> {
    if value.ty() == *ty {
        Ok(())
    } else {
        Err(Fault::new(format!("expected {ty:?}, got {:?}", value.ty())))
    }
}

pub fn run(module: &Module, options: impl Into<ExecutionOptions>) -> Result<Execution, Fault> {
    run_with_library(module, crate::library::system()?, options)
}

/// Execute against an explicitly compiled System library artifact.
pub fn run_with_library(
    module: &Module,
    library: &Module,
    options: impl Into<ExecutionOptions>,
) -> Result<Execution, Fault> {
    if module.entry.is_empty() {
        return Err(Fault::new(
            "cannot execute a library without an entry point",
        ));
    }
    crate::LoadedProgram::with_library(module, library)?.run(options)
}

/// Execute a trusted module with native imports enabled.
///
/// # Safety
/// Every executed native declaration must match its exported C ABI signature.
/// Native code and library initializers/destructors must uphold pointer validity,
/// allocation lifetimes, and Rust's memory safety requirements. Guest metadata alone
/// cannot establish these guarantees. The caller must trust the code being executed.
pub unsafe fn run_with_native(
    module: &Module,
    library: &Module,
    options: impl Into<ExecutionOptions>,
) -> Result<Execution, Fault> {
    if module.entry.is_empty() {
        return Err(Fault::new(
            "cannot execute a library without an entry point",
        ));
    }
    // SAFETY: the caller accepts the same native-code contract as LoadedProgram.
    unsafe { crate::LoadedProgram::with_library(module, library)?.run_with_native(options) }
}

pub(crate) fn interpret(
    module: &Module,
    options: ExecutionOptions,
    native_libraries: Option<crate::interop::NativeLibraries>,
) -> Result<Execution, Fault> {
    let entry = module
        .functions
        .iter()
        .position(|f| f.name == module.entry && f.parameters.is_empty() && !f.instance)
        .ok_or_else(|| Fault::new("missing entry"))?;
    crate::access::check_entry(module, &module.functions[entry])?;
    interpret_function(
        module,
        module.functions[entry].clone(),
        vec![],
        options,
        native_libraries,
    )
}

pub(crate) fn interpret_function(
    module: &Module,
    function: crate::metadata::Function,
    arguments: Vec<Value>,
    options: ExecutionOptions,
    native_libraries: Option<crate::interop::NativeLibraries>,
) -> Result<Execution, Fault> {
    let debugger = options.debugger.clone();
    if let Some(debugger) = &debugger {
        debugger.begin()?;
    }
    let result = (|| {
        let initial_fault = |fault: Fault| {
            fault.with_stack_trace(crate::StackTrace::capture(std::iter::once((&function, 0))))
        };
        options
            .check_cancellation(&function.name, 0)
            .map_err(initial_fault)?;
        let limits = options.limits;
        if limits.frames == 0 {
            return Err(initial_fault(Fault::new("frame limit exceeded")));
        }
        let mut frames = vec![Frame::new(function, arguments)?];
        let result = interpret_frames(module, &mut frames, options, native_libraries);
        result.map_err(|fault: Fault| {
            fault.with_stack_trace(crate::StackTrace::capture(
                frames
                    .iter()
                    .rev()
                    .map(|frame| (frame.function.as_ref(), frame.trace_pc)),
            ))
        })
    })();
    if let (Some(debugger), Err(fault)) = (&debugger, &result) {
        if debugger.snapshot().revision == 0 {
            debugger.finish(Default::default(), Some(fault.to_string()));
        }
    }
    result
}

fn interpret_frames(
    module: &Module,
    frames: &mut Vec<Frame>,
    options: ExecutionOptions,
    mut native_libraries: Option<crate::interop::NativeLibraries>,
) -> Result<Execution, Fault> {
    let mut heap = crate::ManagedHeap::default();
    let mut memory = crate::memory::PointerHeap::default();
    let mut output = vec![];
    let result = interpret_instructions(
        module,
        frames,
        &options,
        &mut native_libraries,
        &mut heap,
        &mut memory,
        &mut output,
    );
    if let Some(debugger) = &options.debugger {
        let mut snapshot = debug_snapshot(module, frames, &heap, &memory, &output, result.is_err());
        if let Ok(value) = &result {
            snapshot.result = Some(debug_value(module, frames, value, 0, &mut 2048));
        }
        debugger.finish(snapshot, result.as_ref().err().map(|e| e.message.clone()));
    }
    result.map(|value| Execution {
        value,
        output,
        heap,
        memory,
        native_libraries,
    })
}

#[allow(clippy::too_many_arguments)]
fn interpret_instructions(
    module: &Module,
    frames: &mut Vec<Frame>,
    options: &ExecutionOptions,
    native_libraries: &mut Option<crate::interop::NativeLibraries>,
    heap: &mut crate::ManagedHeap,
    memory: &mut crate::memory::PointerHeap,
    output: &mut Vec<String>,
) -> Result<Value, Fault> {
    let limits = options.limits;
    let mut collection_threshold = limits.heap_objects.min(64);
    let mut arrays_used = false;
    for _ in 0..limits.instructions {
        if let (Some(debugger), Some(frame)) = (&options.debugger, frames.last()) {
            let before_host_call = matches!(frame.function.body.get(frame.pc), Some(Op::Call(target))
                if resolve(module, target).is_ok_and(|f| f.pinvoke.is_some() || f.is_internal_call()));
            debugger.checkpoint(
                (&frame.function.name, frame.pc, frames.len()),
                source_point(&frame.function, frame.pc),
                options.cancellation.as_ref(),
                before_host_call,
                || debug_snapshot(module, frames, heap, memory, output, false),
            )?;
        }

        if arrays_used {
            let check = |frames: &[Frame], heap: &crate::ManagedHeap| -> Result<(), Fault> {
                let mut usage = crate::arrays::Usage::default();
                for frame in frames {
                    for cell in frame.args.iter().chain(&frame.locals) {
                        cell.borrow().array_usage(&mut usage, &limits)?;
                    }
                    for value in &frame.stack {
                        crate::arrays::measure(value, &mut usage, &limits)?;
                    }
                }
                heap.array_usage(&mut usage, &limits)
            };
            if check(frames, heap).is_err() {
                let mut roots = vec![];
                for frame in frames.iter() {
                    for cell in frame.args.iter().chain(&frame.locals) {
                        cell.borrow().trace_heap(&mut roots);
                    }
                    for value in &frame.stack {
                        crate::gc::trace(value, &mut roots);
                    }
                }
                heap.collect(roots, crate::CollectionReason::AllocationPressure)?;
                check(frames, heap).map_err(|mut error| {
                    if let Some(frame) = frames.last() {
                        error.function = Some(frame.function.name.clone());
                        error.instruction = Some(frame.trace_pc);
                    }
                    error
                })?;
            }
        }
        let frame = frames
            .last_mut()
            .ok_or_else(|| Fault::new("missing frame"))?;
        let function = frame.function.clone();
        let pc = frame.pc;
        frame.trace_pc = pc;
        options.check_cancellation(&function.name, pc)?;
        let op = function.body.get(pc).ok_or_else(|| Fault {
            message: "function fell through without ret".into(),
            function: Some(function.name.clone()),
            instruction: Some(pc),
            stack_trace: None,
        })?;
        let access_alignment = match pc.checked_sub(1).and_then(|i| function.body.get(i)) {
            Some(Op::Unaligned(alignment)) => Some(*alignment as usize),
            _ => None,
        };
        frame.pc += 1;
        let context = function.name.clone();
        // Collect only between instructions, before allocation operands leave roots.
        if matches!(op, Op::HeapNew | Op::NewArray(_) | Op::AllocateArray(_))
            && heap.len() >= collection_threshold
        {
            let mut roots = vec![];
            for frame in frames.iter() {
                for cell in frame.args.iter().chain(&frame.locals) {
                    cell.borrow().trace_heap(&mut roots);
                }
                for value in &frame.stack {
                    crate::gc::trace(value, &mut roots);
                }
            }
            heap.collect(roots, crate::CollectionReason::AllocationPressure)?;
            collection_threshold = heap
                .len()
                .saturating_mul(2)
                .max(64)
                .min(limits.heap_objects);
        }

        // Host Result propagates terminal faults; there is no guest exception machinery.
        let step = (|| -> Result<Option<Value>, Fault> {
            let frame = frames
                .last_mut()
                .ok_or_else(|| Fault::new("missing frame"))?;
            match op {
                Op::Unaligned(_) => (),
                Op::Int(n) => frame.stack.push(Value::Int32(*n)),
                Op::Float32 { bits } => frame
                    .stack
                    .push(Value::Double(f32::from_bits(*bits) as f64)),
                Op::Float64 { bits } => frame.stack.push(Value::Double(f64::from_bits(*bits))),
                Op::CheckFinite => {
                    let value = frame.pop()?;
                    match value {
                        Value::Double(n) if n.is_finite() => frame.stack.push(value),
                        Value::Double(_) => {
                            return Err(Fault::new("non-finite floating-point value"));
                        }
                        _ => return Err(Fault::new("ckfinite requires floating-point value")),
                    }
                }
                Op::ConvertFloat32 | Op::ConvertFloat64 | Op::ConvertFloatUnsigned => {
                    let value = frame.pop()?;
                    frame.stack.push(crate::floating::convert(op, value)?);
                }
                Op::Int64(n) => frame.stack.push(Value::Int64(*n)),
                Op::Bool(b) => frame.stack.push(Value::Boolean(*b)),
                Op::String(s) => frame.stack.push(Value::String(s.clone())),
                Op::Void => frame.stack.push(Value::Void),
                Op::Error(s) => frame.stack.push(Value::Error(s.clone())),
                Op::LocalAddress(i) => {
                    frame
                        .stack
                        .push(Value::SlotReference(crate::SlotReference::new(
                            &frame.locals[*i],
                        )))
                }
                Op::ArgumentAddress(i) => {
                    frame
                        .stack
                        .push(Value::SlotReference(crate::SlotReference::new(
                            &frame.args[*i],
                        )))
                }
                Op::Arg(i) => frame.stack.push(frame.args[*i].borrow().get()?.on_stack()),
                Op::StoreArg(i) => {
                    let value = frame.pop()?;
                    frame.args[*i].borrow_mut().set(value)?;
                }
                Op::Load(i) => frame
                    .stack
                    .push(frame.locals[*i].borrow().get()?.on_stack()),
                Op::ResetLocal(i) => crate::slots::Slot::reset(&frame.locals[*i])?,
                Op::Store(i) => {
                    let value = frame.pop()?;
                    assigned_reference(&value)?;
                    frame.locals[*i].borrow_mut().set(value)?;
                }
                Op::Dup => {
                    let value = frame
                        .stack
                        .last()
                        .ok_or_else(|| Fault::new("evaluation stack underflow"))?
                        .clone();
                    frame.stack.push(value);
                }
                Op::Pop => {
                    frame.pop()?;
                }
                Op::BitNot | Op::Negate => {
                    let value = frame.pop()?;
                    frame.stack.push(crate::numeric::unary(op, value)?);
                }
                Op::ShiftLeft | Op::ShiftRight | Op::ShiftRightUnsigned => {
                    let count = frame.pop()?;
                    let value = frame.pop()?;
                    frame.stack.push(crate::numeric::shift(op, value, count)?);
                }
                Op::BitAnd
                | Op::BitOr
                | Op::BitXor
                | Op::Remainder
                | Op::RemainderUnsigned
                | Op::Add
                | Op::Sub
                | Op::Mul
                | Op::AddChecked
                | Op::SubChecked
                | Op::MulChecked
                | Op::AddCheckedUnsigned
                | Op::SubCheckedUnsigned
                | Op::MulCheckedUnsigned
                | Op::Divide
                | Op::DivideUnsigned
                | Op::Less
                | Op::LessUnsigned
                | Op::Greater
                | Op::GreaterUnsigned => {
                    let right = frame.pop()?;
                    let left = frame.pop()?;
                    frame.stack.push(crate::numeric::binary(op, left, right)?);
                }
                Op::CheckedInt8
                | Op::CheckedUInt8
                | Op::CheckedInt16
                | Op::CheckedUInt16
                | Op::CheckedInt32
                | Op::CheckedUInt32
                | Op::CheckedInt64
                | Op::CheckedUInt64
                | Op::CheckedNativeInt
                | Op::CheckedNativeUInt
                | Op::CheckedInt8Unsigned
                | Op::CheckedUInt8Unsigned
                | Op::CheckedInt16Unsigned
                | Op::CheckedUInt16Unsigned
                | Op::CheckedInt32Unsigned
                | Op::CheckedUInt32Unsigned
                | Op::CheckedInt64Unsigned
                | Op::CheckedUInt64Unsigned
                | Op::CheckedNativeIntUnsigned
                | Op::CheckedNativeUIntUnsigned => {
                    let value = frame.pop()?;
                    frame.stack.push(crate::checked::convert(op, value)?);
                }
                Op::ConvertNativeInt
                | Op::ConvertNativeUInt
                | Op::ConvertInt32
                | Op::ConvertInt8
                | Op::ConvertUInt8
                | Op::ConvertInt16
                | Op::ConvertUInt16
                | Op::ConvertUInt32
                | Op::ConvertInt64
                | Op::ConvertUInt64 => {
                    let value = frame.pop()?;
                    frame.stack.push(crate::numeric::convert(op, value)?);
                }
                Op::PointerFromInt(ty) => {
                    let address = match frame.pop()? {
                        Value::IntPtr(n) => n as usize,
                        Value::UIntPtr(n) => n,
                        _ => return Err(Fault::new("ptr.fromint requires native integer")),
                    };
                    frame.stack.push(Value::Pointer(
                        memory.pointer_at_address(address, ty.clone()),
                    ));
                }
                Op::Equal => {
                    let right = frame.pop()?;
                    let left = frame.pop()?;
                    expect(&right, &left.ty())?;
                    frame.stack.push(Value::Boolean(left == right));
                }
                Op::Branch(i) => frame.pc = *i,
                Op::BranchTrue(i) | Op::BranchFalse(i) => {
                    let condition = match frame.pop()? {
                        Value::Boolean(value) => value,
                        Value::Int32(value) => value != 0,
                        Value::Int64(value) => value != 0,
                        Value::IntPtr(value) => value != 0,
                        Value::UIntPtr(value) => value != 0,
                        // A condition tests the address, not pointee validity or lifetime.
                        Value::Pointer(pointer) => pointer.address != 0,
                        Value::SlotReference(reference)
                        | Value::SlotInterface {
                            receiver: reference,
                            ..
                        } => {
                            reference.assigned()?;
                            true
                        }

                        _ => {
                            return Err(Fault::new(
                                "conditional branch requires Boolean, integer, pointer, or managed reference",
                            ));
                        }
                    };
                    if condition == matches!(op, Op::BranchTrue(_)) {
                        frame.pc = *i;
                    }
                }
                Op::BranchEqual(i)
                | Op::BranchNotEqual(i)
                | Op::BranchGreater(i)
                | Op::BranchGreaterUnsigned(i)
                | Op::BranchLess(i)
                | Op::BranchLessUnsigned(i)
                | Op::BranchGreaterEqual(i)
                | Op::BranchGreaterEqualUnsigned(i)
                | Op::BranchLessEqual(i)
                | Op::BranchLessEqualUnsigned(i) => {
                    let right = frame.pop()?;
                    let left = frame.pop()?;
                    if crate::numeric::branch_condition(op, left, right)? {
                        frame.pc = *i;
                    }
                }
                Op::Switch(targets) => {
                    let Value::Int32(index) = frame.pop()? else {
                        return Err(Fault::new("switch requires Int32"));
                    };
                    if let Some(target) = targets.get(index as u32 as usize) {
                        frame.pc = *target;
                    }
                }
                Op::BorrowInterface(interface) => {
                    let view = match frame.pop()? {
                        Value::SlotReference(receiver) => {
                            receiver.assigned()?;
                            crate::interfaces::ensure_implementation(
                                module,
                                receiver.target(),
                                interface,
                            )?;
                            Value::SlotInterface {
                                interface: interface.clone(),
                                receiver,
                            }
                        }
                        Value::Pointer(receiver) => {
                            crate::interfaces::ensure_implementation(
                                module,
                                &receiver.target,
                                interface,
                            )?;
                            crate::memory::layout(module, &receiver.target)?;
                            Value::InterfaceRef {
                                interface: interface.clone(),
                                receiver,
                            }
                        }
                        _ => {
                            return Err(Fault::new(
                                "interface.borrow requires a typed pointer or managed slot reference",
                            ));
                        }
                    };
                    frame.stack.push(view);
                }
                Op::CallVirtual(target) => {
                    let contract = resolve(module, target)?;
                    crate::access::check_call(module, Some(&function), &contract)?;
                    let mut args = frame.args(&contract.argument_types()[1..])?;
                    let (interface, concrete, storage) = match frame.pop()? {
                        Value::SlotInterface {
                            interface,
                            receiver,
                        } => (interface, receiver.target().clone(), Ok(receiver)),
                        Value::InterfaceRef {
                            interface,
                            receiver,
                        } => (interface, receiver.target.clone(), Err(receiver)),
                        _ => {
                            return Err(Fault::new("callvirt requires an interface view receiver"));
                        }
                    };
                    if contract.owner.as_ref() != Some(&interface) {
                        return Err(Fault::new("interface receiver type mismatch"));
                    }
                    let callee = crate::interfaces::implementation(
                        module, &concrete, &interface, &contract,
                    )?;
                    let receiver = match storage {
                        Ok(slot) => {
                            slot.assigned()?;
                            if callee.receiver_byref {
                                Value::SlotReference(slot)
                            } else {
                                slot.read()?
                            }
                        }
                        Err(pointer) => {
                            if callee.receiver_byref {
                                return Err(Fault::new(
                                    "byref interface receiver requires a managed slot view",
                                ));
                            }
                            memory.read(&pointer, &crate::memory::layout(module, &concrete)?)?
                        }
                    };
                    args.insert(0, receiver);
                    if frames.len() >= limits.frames {
                        return Err(Fault::new("frame limit exceeded"));
                    }
                    frames.push(Frame::new(callee, args)?);
                }
                Op::ReferenceEqual => {
                    let mut reference = || -> Result<crate::SlotReference, Fault> {
                        let reference = match frame.pop()? {
                            Value::SlotReference(reference)
                            | Value::SlotInterface {
                                receiver: reference,
                                ..
                            } => reference,
                            _ => return Err(Fault::new("ref.eq requires managed references")),
                        };
                        reference.assigned()?;
                        Ok(reference)
                    };
                    let right = reference()?;
                    let left = reference()?;
                    frame.stack.push(Value::Boolean(left.same_location(&right)));
                }
                Op::ReferenceType => {
                    let reference = match frame.pop()? {
                        Value::SlotReference(reference)
                        | Value::SlotInterface {
                            receiver: reference,
                            ..
                        } => reference,
                        _ => return Err(Fault::new("ref.type requires a managed reference")),
                    };
                    reference.assigned()?;
                    frame.stack.push(Value::RuntimeTypeHandle(Box::new(
                        crate::type_identity::describe_loaded(module, reference.target())?,
                    )));
                }
                Op::LoadTypeToken(ty) => {
                    frame.stack.push(Value::RuntimeTypeHandle(Box::new(
                        crate::type_identity::describe(module, ty)?,
                    )));
                }
                Op::PackValue(ty) => {
                    let value = frame.pop()?.erase(ty)?;
                    frame.stack.push(value);
                }
                Op::IsValue(ty) | Op::UnpackValue(ty) => {
                    let Value::Erased(value) = frame.pop()? else {
                        return Err(Fault::new("expected System.Value"));
                    };
                    if matches!(op, Op::IsValue(_)) {
                        frame.stack.push(Value::Boolean(value.ty() == *ty));
                    } else if value.ty() == *ty {
                        frame.stack.push(value.on_stack());
                    } else {
                        return Err(Fault::new(format!(
                            "erased value contains {:?}, requested {ty:?}",
                            value.ty()
                        )));
                    }
                }
                Op::Construct(target) => {
                    let callee = resolve_constructor(module, target)?;
                    crate::access::check_call(module, Some(&function), &callee)?;
                    let owner = callee
                        .owner
                        .clone()
                        .ok_or_else(|| Fault::new("missing constructor owner"))?;
                    check_type(&owner, module)?;
                    let args = frame.args(&callee.parameters)?;
                    if frames.len() >= limits.frames {
                        return Err(Fault::new("frame limit exceeded"));
                    }
                    let empty = module.instantiated_fields(&owner)?.is_empty();
                    let mut child = Frame::new(callee, args)?;
                    child.constructing = true;
                    child.args.insert(
                        0,
                        crate::slots::Slot::new(
                            owner.clone(),
                            if empty {
                                Some(Value::Object {
                                    ty: owner,
                                    fields: vec![],
                                })
                            } else {
                                None
                            },
                        ),
                    );
                    frames.push(child);
                }
                Op::Call(target) => {
                    let callee = resolve(module, target)?;
                    if crate::interfaces::is_contract(module, &callee) {
                        return Err(Fault::new("interface declarations require callvirt"));
                    }
                    crate::access::check_call(module, Some(&function), &callee)?;
                    callee.map_types(|ty| {
                        check_type(ty, module)?;
                        Ok(ty.clone())
                    })?;
                    let mut args = frame.args(&callee.argument_types())?;
                    restrict_reference_arguments(&callee, &mut args)?;
                    if callee.pinvoke.is_some() {
                        let libraries = native_libraries.as_mut().ok_or_else(|| {
                            Fault::new("native imports require trusted run_with_native execution")
                        })?;
                        // SAFETY: a native library session is only supplied by run_with_native,
                        // whose caller accepts the native ABI and memory safety contract.
                        let value = unsafe { libraries.invoke(&callee, args, memory)? };
                        expect(&value, &callee.returns)?;
                        frame.stack.push(value.on_stack());
                    } else if callee.is_internal_call() {
                        let binding = crate::native::bind(&callee)?;
                        if matches!(binding, crate::native::Binding::Reflection(_)) {
                            arrays_used = true;
                        }
                        let value = binding.invoke(
                            args,
                            module,
                            &limits,
                            output,
                            options.console.as_deref(),
                        )?;
                        expect(&value, &callee.returns)?;
                        frame.stack.push(value);
                    } else {
                        if frames.len() >= limits.frames {
                            return Err(Fault::new("frame limit exceeded"));
                        }
                        frames.push(Frame::new(callee, args)?);
                    }
                }
                Op::Return => {
                    let value = frame.pop()?;
                    let value = value.for_storage(&function.returns)?;
                    assigned_reference(&value)?;
                    frame.check_reference_return(&value)?;
                    for (output, conditional) in &frame.outputs {
                        if !conditional || value == Value::Boolean(true) {
                            output.assigned()?;
                        }
                    }
                    if !frame.stack.is_empty() {
                        return Err(Fault::new("ret requires exactly one value"));
                    }
                    let value = if frame.constructing {
                        frame.args[0].borrow().get().map_err(|_| {
                            Fault::new("constructor returned without initializing its receiver")
                        })?
                    } else {
                        value
                    };
                    for pointer in &frame.allocations {
                        memory.release_local(pointer)?;
                    }
                    frames.pop();
                    if let Some(caller) = frames.last_mut() {
                        caller.stack.push(value.on_stack());
                    } else {
                        return Ok(Some(value));
                    }
                }
                Op::AllocateArray(ty) | Op::NewArray(ty) | Op::CreateArray(ty) => {
                    arrays_used = true;
                    let initial = if matches!(op, Op::CreateArray(_)) {
                        frame.pop()?.for_storage(ty)?
                    } else if matches!(op, Op::AllocateArray(_)) {
                        Value::Uninitialized(ty.clone())
                    } else {
                        crate::initialization::default_value(module, ty)?
                    };
                    let length = crate::arrays::index(frame.pop()?).map_err(|_| {
                        Fault::new("array length must be a non-negative Int32 or native integer")
                    })?;
                    let value = crate::arrays::create(ty.clone(), length, initial, &limits)?;
                    if matches!(op, Op::NewArray(_) | Op::AllocateArray(_)) {
                        if heap.len() >= limits.heap_objects {
                            return Err(Fault::new("heap object limit exceeded"));
                        }
                        let index = heap.allocate(value)?;
                        frame.stack.push(Value::SlotReference(heap.address(index)?));
                    } else {
                        frame.stack.push(value);
                    }
                }
                Op::ArrayLength => {
                    let length = match frame.pop()? {
                        Value::SlotReference(reference) => reference.array_length()?,
                        Value::Array { elements, .. } => elements.len(),
                        _ => return Err(Fault::new("ldlen requires array")),
                    };
                    frame.stack.push(Value::UIntPtr(length));
                }
                Op::ArrayElement(ty) | Op::ArrayAddress(ty) | Op::StoreArrayElement(ty) => {
                    let stored = if matches!(op, Op::StoreArrayElement(_)) {
                        Some(frame.pop()?)
                    } else {
                        None
                    };
                    let index = crate::arrays::index(frame.pop()?)?;
                    match frame.pop()? {
                        Value::SlotReference(reference) => {
                            if matches!(op, Op::ArrayAddress(_))
                                && matches!(ty, Type::ByRef(_) | Type::ReadOnlyByRef(_))
                            {
                                return Err(Fault::new(
                                    "nested managed references are not supported",
                                ));
                            }
                            let address = reference.element(index, ty)?;
                            if matches!(op, Op::ArrayElement(_)) {
                                frame.stack.push(address.read()?.on_stack());
                            } else if let Some(value) = stored {
                                address.write(value)?;
                            } else {
                                frame.stack.push(Value::SlotReference(address));
                            }
                        }
                        Value::Array { element, elements } if matches!(op, Op::ArrayElement(_)) => {
                            if element != *ty {
                                return Err(Fault::new("array element type mismatch"));
                            }
                            frame.stack.push(
                                elements
                                    .get(index)
                                    .ok_or_else(|| Fault::new("array index out of range"))?
                                    .initialized()?
                                    .clone()
                                    .on_stack(),
                            );
                        }
                        _ => {
                            return Err(Fault::new(
                                "array mutation/address requires managed array reference",
                            ));
                        }
                    }
                }
                Op::New(ty) => {
                    crate::access::check_construction(module, &function, ty)?;
                    let definitions = module.instantiated_fields(ty)?;
                    let types: Vec<_> = definitions.iter().map(|f| f.ty.clone()).collect();
                    let fields = frame.args(&types)?;
                    for field in &fields {
                        field.ensure_heap_references()?;
                    }
                    frame.stack.push(Value::Object {
                        ty: ty.clone(),
                        fields,
                    });
                }
                Op::Field(i) => {
                    let Value::Object { ty, fields } = frame.pop()? else {
                        return Err(Fault::new(
                            "ldfld requires object value (use ldobj for a reference)",
                        ));
                    };
                    crate::access::check_field(module, &function, &ty, *i)?;
                    frame.stack.push(
                        fields
                            .get(*i)
                            .ok_or_else(|| Fault::new("field index out of range"))?
                            .clone()
                            .on_stack(),
                    );
                }
                Op::SetField(i) => {
                    let value = frame.pop()?;
                    let Value::Object { ty, mut fields } = frame.pop()? else {
                        return Err(Fault::new("stfld requires object value"));
                    };
                    crate::access::check_field(module, &function, &ty, *i)?;
                    let field = fields
                        .get_mut(*i)
                        .ok_or_else(|| Fault::new("field index out of range"))?;
                    value.ensure_heap_references()?;
                    *field = value.for_storage(&field.ty())?;
                    frame.stack.push(Value::Object { ty, fields });
                }
                Op::SizeOf(ty) | Op::AlignOf(ty) => {
                    let layout = crate::memory::layout(module, ty)?;
                    let value = if matches!(op, Op::SizeOf(_)) {
                        layout.size
                    } else {
                        layout.alignment
                    };
                    frame.stack.push(Value::Int32(
                        i32::try_from(value)
                            .map_err(|_| Fault::new("layout exceeds Int32 range"))?,
                    ));
                }
                Op::AllocateLocal => {
                    let size = match frame.pop()? {
                        Value::Int32(n) => usize::try_from(n).ok(),
                        Value::IntPtr(n) => usize::try_from(n).ok(),
                        Value::UIntPtr(n) => Some(n),
                        _ => return Err(Fault::new("localloc size requires integer")),
                    }
                    .ok_or_else(|| Fault::new("negative localloc size"))?;
                    if !frame.stack.is_empty() {
                        return Err(Fault::new(
                            "localloc requires only its size on the evaluation stack",
                        ));
                    }
                    let pointer = memory.allocate_local(
                        size,
                        limits.pointer_bytes,
                        limits.pointer_allocations,
                    )?;
                    frame.allocations.push(pointer.clone());
                    frame.stack.push(Value::Pointer(pointer));
                }
                Op::Allocate(ty) => {
                    let count = match frame.pop()? {
                        Value::Int32(n) => usize::try_from(n).ok(),
                        Value::IntPtr(n) => usize::try_from(n).ok(),
                        Value::UIntPtr(n) => Some(n),
                        _ => return Err(Fault::new("allocation count requires integer")),
                    }
                    .ok_or_else(|| Fault::new("negative allocation count"))?;
                    let layout = crate::memory::layout(module, ty)?;
                    let pointer = memory.allocate(
                        ty.clone(),
                        &layout,
                        count,
                        limits.pointer_bytes,
                        limits.pointer_allocations,
                    )?;
                    frame.stack.push(Value::Pointer(pointer));
                }
                Op::Free => {
                    memory.free(&frame.pointer()?)?;
                    frame.stack.push(Value::Void);
                }
                Op::NullPointer(ty) => frame
                    .stack
                    .push(Value::Pointer(crate::memory::Pointer::null(ty.clone()))),
                Op::PointerCast(ty) => {
                    let mut pointer = frame.pointer()?;
                    pointer.target = ty.clone();
                    frame.stack.push(Value::Pointer(pointer));
                }
                Op::PointerAdd => {
                    let offset = match frame.pop()? {
                        Value::Int32(n) => n as isize,
                        Value::IntPtr(n) => n,
                        _ => return Err(Fault::new("pointer offset requires Int32 or IntPtr")),
                    };
                    let pointer = frame.pointer()?;
                    frame
                        .stack
                        .push(Value::Pointer(memory.offset(&pointer, offset)?));
                }
                Op::FieldAddress(index) => {
                    if let Some(Value::SlotReference(reference)) = frame.stack.last() {
                        crate::access::check_field(module, &function, reference.target(), *index)?;
                        let fields = module.instantiated_fields(reference.target())?;
                        let target = fields
                            .get(*index)
                            .ok_or_else(|| Fault::new("field index out of range"))?
                            .ty
                            .clone();
                        let reference = reference.field(*index, target)?;
                        frame.pop()?;
                        frame.stack.push(Value::SlotReference(reference));
                        return Ok(None);
                    }
                    let pointer = frame.pointer()?;
                    if !matches!(pointer.target, Type::Named(_) | Type::Constructed { .. }) {
                        return Err(Fault::new("ldflda requires pointer to record"));
                    }
                    crate::access::check_field(module, &function, &pointer.target, *index)?;
                    let layout = crate::memory::layout(module, &pointer.target)?;
                    frame
                        .stack
                        .push(Value::Pointer(memory.field(&pointer, &layout, *index)?));
                }
                Op::CopyObject(ty) | Op::InitializeObject(ty) => {
                    if let (Op::InitializeObject(_), Some(Value::SlotReference(reference))) =
                        (op, frame.stack.last())
                    {
                        if reference.target() != ty {
                            return Err(Fault::new("memory operation pointer type mismatch"));
                        }
                        // Build the complete default before publishing it or fulfilling outputs.
                        let value = crate::initialization::default_value(module, ty)?;
                        reference.write(value)?;
                        frame.pop()?;
                        return Ok(None);
                    }
                    let source = if matches!(op, Op::CopyObject(_)) {
                        Some(frame.pointer()?)
                    } else {
                        None
                    };
                    let destination = frame.pointer()?;
                    if destination.target != *ty || source.as_ref().is_some_and(|p| p.target != *ty)
                    {
                        return Err(Fault::new("memory operation pointer type mismatch"));
                    }
                    let layout = crate::memory::layout(module, ty)?;
                    if let Some(source) = source {
                        let value = memory.read(&source, &layout)?;
                        memory.write(&destination, &layout, &value)?;
                    } else {
                        memory.fill(&destination, &layout, 0)?;
                    }
                }
                Op::CopyBlock | Op::InitializeBlock => {
                    let size = match frame.pop()? {
                        Value::Int32(n) => usize::try_from(n).ok(),
                        Value::IntPtr(n) => usize::try_from(n).ok(),
                        Value::UIntPtr(n) => Some(n),
                        _ => return Err(Fault::new("block size requires integer")),
                    }
                    .ok_or_else(|| Fault::new("negative block size"))?;
                    let layout = crate::memory::Layout {
                        size,
                        alignment: 1,
                        fields: vec![],
                    };
                    if matches!(op, Op::CopyBlock) {
                        let source = frame.pointer()?;
                        let destination = frame.pointer()?;
                        memory.copy_block(&destination, &source, &layout)?;
                    } else {
                        let Value::Int32(value) = frame.pop()? else {
                            return Err(Fault::new("initblk fill requires Int32"));
                        };
                        let destination = frame.pointer()?;
                        memory.fill(&destination, &layout, value as u8)?;
                    }
                }
                Op::LoadObject(_)
                | Op::LoadIndirectInt32
                | Op::LoadIndirectInt8
                | Op::LoadIndirectUInt8
                | Op::LoadIndirectInt16
                | Op::LoadIndirectUInt16
                | Op::LoadIndirectUInt32
                | Op::LoadIndirectInt64
                | Op::LoadIndirectNative
                | Op::LoadIndirectFloat32
                | Op::LoadIndirectFloat64 => {
                    if let (Op::LoadObject(ty), Some(Value::SlotReference(reference))) =
                        (op, frame.stack.last())
                    {
                        if reference.target() != ty {
                            return Err(Fault::new("managed reference load type mismatch"));
                        }
                        if access_alignment.is_some() {
                            return Err(Fault::new("unaligned is not valid on slot references"));
                        }
                        let value = reference.read()?.on_stack();
                        frame.pop()?;
                        frame.stack.push(value);
                        return Ok(None);
                    }
                    let mut pointer = frame.pointer()?;
                    let ty = if let Op::LoadObject(ty) = op {
                        if pointer.target != *ty {
                            return Err(Fault::new("memory load pointer type mismatch"));
                        }
                        ty.clone()
                    } else {
                        crate::numeric::indirect_type(op, &pointer.target)?
                    };
                    pointer.target = ty.clone();
                    let mut layout = crate::memory::layout(module, &ty)?;
                    if let Some(alignment) = access_alignment {
                        layout.alignment = layout.alignment.min(alignment);
                    }
                    frame.stack.push(memory.read(&pointer, &layout)?.on_stack());
                }
                Op::StoreObject(_)
                | Op::StoreIndirectInt32
                | Op::StoreIndirectInt8
                | Op::StoreIndirectInt16
                | Op::StoreIndirectInt64
                | Op::StoreIndirectNative
                | Op::StoreIndirectFloat32
                | Op::StoreIndirectFloat64 => {
                    let value = frame.pop()?;
                    if let (Op::StoreObject(ty), Some(Value::SlotReference(reference))) =
                        (op, frame.stack.last())
                    {
                        if reference.target() != ty {
                            return Err(Fault::new("managed reference store type mismatch"));
                        }
                        if access_alignment.is_some() {
                            return Err(Fault::new("unaligned is not valid on slot references"));
                        }
                        reference.write(value)?;
                        frame.pop()?;
                        return Ok(None);
                    }
                    let pointer = frame.pointer()?;
                    if let Op::StoreObject(ty) = op {
                        if pointer.target != *ty {
                            return Err(Fault::new("memory store pointer type mismatch"));
                        }
                    } else {
                        crate::numeric::indirect_type(op, &pointer.target)?;
                    }
                    let ty = &pointer.target;
                    let value = value
                        .for_storage(ty)
                        .map_err(|_| Fault::new("memory store type mismatch"))?;
                    let mut layout = crate::memory::layout(module, ty)?;
                    if let Some(alignment) = access_alignment {
                        layout.alignment = layout.alignment.min(alignment);
                    }
                    memory.write(&pointer, &layout, &value)?;
                }
                Op::HeapNew => {
                    if heap.len() >= limits.heap_objects {
                        return Err(Fault::new("heap object limit exceeded"));
                    }
                    let value = frame.pop()?;
                    let target = value.ty();
                    if matches!(&target, Type::ByRef(_) | Type::ReadOnlyByRef(_)) {
                        return Err(Fault::new(
                            "managed references cannot escape into heap storage",
                        ));
                    }
                    let index = heap.allocate(value)?;
                    frame.stack.push(Value::SlotReference(heap.address(index)?));
                }
                Op::Fault(message) => return Err(Fault::new(message)),
            }
            Ok(None)
        })();
        match step {
            Ok(Some(value)) => {
                let mut roots = vec![];
                crate::gc::trace(&value, &mut roots);
                heap.collect(roots, crate::CollectionReason::ExecutionCompleted)?;
                return Ok(value);
            }
            Err(mut fault) => {
                fault.function = Some(context);
                fault.instruction = Some(pc);
                return Err(fault);
            }
            Ok(None) => (),
        }
        if frames.iter().any(|f| f.stack.len() > limits.stack) {
            return Err(Fault::new("evaluation stack limit exceeded"));
        }
    }
    if let Some(frame) = frames.last_mut() {
        frame.trace_pc = frame.pc;
    }
    Err(Fault::new("instruction limit exceeded"))
}

fn source_point(
    function: &crate::metadata::Function,
    pc: usize,
) -> Option<&crate::metadata::SequencePoint> {
    function
        .sequence_points
        .iter()
        .rev()
        .find(|p| p.instruction <= pc)
}
fn debug_text(text: &str) -> String {
    text.chars()
        .take(256)
        .flat_map(char::escape_debug)
        .collect()
}
fn debug_value(
    module: &Module,
    frames: &[Frame],
    value: &Value,
    depth: usize,
    budget: &mut usize,
) -> crate::debugger::DebugValue {
    use crate::debugger::DebugValue;
    if *budget == 0 || depth >= 8 {
        return DebugValue {
            value: "…".into(),
            truncated: true,
            ..Default::default()
        };
    }
    *budget -= 1;
    let mut result = DebugValue {
        ty: debug_text(
            &crate::type_identity::signature_name(&value.ty())
                .unwrap_or_else(|_| "<unknown>".into()),
        ),
        ..Default::default()
    };
    match value {
        Value::Object { ty, fields } => {
            result.value = format!("record ({} fields)", fields.len());
            let names = module.instantiated_fields(ty).unwrap_or_default();
            for (i, field) in fields.iter().take(64).enumerate() {
                result.children.push((
                    names
                        .get(i)
                        .map_or_else(|| i.to_string(), |f| debug_text(&f.name)),
                    debug_value(module, frames, field, depth + 1, budget),
                ));
            }
            result.truncated = fields.len() > 64;
        }
        Value::Array { elements, .. } => {
            result.value = format!("array length={}", elements.len());
            for (i, element) in elements.iter().take(64).enumerate() {
                result.children.push((
                    i.to_string(),
                    debug_value(module, frames, element, depth + 1, budget),
                ));
            }
            result.truncated = elements.len() > 64;
        }
        Value::Erased(payload) => {
            result.value = "erased value".into();
            result.children.push((
                "payload".into(),
                debug_value(module, frames, payload, depth + 1, budget),
            ));
        }
        Value::SlotReference(reference)
        | Value::SlotInterface {
            receiver: reference,
            ..
        } => {
            let root = if let Some(id) = reference.allocation_id() {
                format!("heap#{id}")
            } else {
                frames
                    .iter()
                    .enumerate()
                    .find_map(|(frame, f)| {
                        f.args
                            .iter()
                            .enumerate()
                            .find(|(_, c)| reference.addresses(c))
                            .map(|(i, _)| format!("frame#{frame}.arg#{i}"))
                            .or_else(|| {
                                f.locals
                                    .iter()
                                    .enumerate()
                                    .find(|(_, c)| reference.addresses(c))
                                    .map(|(i, _)| format!("frame#{frame}.local#{i}"))
                            })
                    })
                    .unwrap_or_else(|| "expired frame".into())
            };
            result.value = format!(
                "{}&{root} path={:?}{}",
                if reference.is_readonly() {
                    "readonly "
                } else {
                    ""
                },
                reference.debug_path(),
                if matches!(value, Value::SlotInterface { .. }) {
                    " (interface view)"
                } else {
                    ""
                }
            );
        }
        Value::Pointer(pointer)
        | Value::InterfaceRef {
            receiver: pointer, ..
        } => {
            result.value = if pointer.address == 0 {
                "null native pointer".into()
            } else {
                format!(
                    "native allocation={:?} offset={} (not dereferenced)",
                    pointer.allocation, pointer.offset
                )
            };
        }
        Value::RuntimeTypeHandle(handle) => result.value = debug_text(&handle.name),
        Value::String(text) | Value::Error(text) => {
            result.value = debug_text(text);
            result.truncated = text.chars().count() > 256;
        }
        _ => result.value = format!("{value:?}"),
    }
    result
}
fn debug_snapshot(
    module: &Module,
    frames: &[Frame],
    heap: &crate::ManagedHeap,
    memory: &crate::memory::PointerHeap,
    output: &[String],
    fault: bool,
) -> crate::debugger::DebugSnapshot {
    let mut snapshot = crate::debugger::DebugSnapshot::default();
    let mut budget = 8192;
    for (index, frame) in frames.iter().enumerate().rev().take(64) {
        let pc = if fault || index + 1 != frames.len() {
            frame.trace_pc
        } else {
            frame.pc
        };
        let slot = |cell: &crate::slots::Cell, budget: &mut usize| {
            let slot = cell.borrow();
            slot.inspect().map_or_else(
                || crate::debugger::DebugValue {
                    ty: debug_text(
                        &crate::type_identity::signature_name(slot.inspect_type())
                            .unwrap_or_else(|_| "<unknown>".into()),
                    ),
                    value: "uninitialized".into(),
                    ..Default::default()
                },
                |v| debug_value(module, frames, v, 0, budget),
            )
        };
        let arguments = frame
            .args
            .iter()
            .enumerate()
            .take(128)
            .map(|(i, cell)| {
                let name = if i == 0 && frame.function.instance {
                    "this".into()
                } else {
                    frame
                        .function
                        .parameter_names
                        .get(i - usize::from(frame.function.instance))
                        .and_then(|n| n.clone())
                        .unwrap_or_else(|| format!("arg#{i}"))
                };
                (name, slot(cell, &mut budget))
            })
            .collect();
        let locals = frame
            .locals
            .iter()
            .enumerate()
            .take(128)
            .map(|(i, cell)| {
                (
                    format!(
                        "local#{i} {}",
                        frame
                            .function
                            .local_names
                            .get(i)
                            .and_then(|n| n.as_deref())
                            .unwrap_or("")
                    ),
                    slot(cell, &mut budget),
                )
            })
            .collect();
        snapshot.frames.push(crate::debugger::DebugFrame {
            index,
            function: frame.function.name.clone(),
            instruction: pc,
            operation: frame
                .function
                .body
                .get(pc)
                .map(|op| debug_text(&format!("{op:?}")))
                .unwrap_or_else(|| "<end>".into()),
            source: source_point(&frame.function, pc).cloned(),
            arguments,
            locals,
            evaluation_stack: frame
                .stack
                .iter()
                .take(128)
                .map(|v| debug_value(module, frames, v, 0, &mut budget))
                .collect(),
        });
        snapshot.truncated |=
            frame.args.len() > 128 || frame.locals.len() > 128 || frame.stack.len() > 128;
    }
    for (id, cell) in heap.debug_cells().take(256) {
        if let Some(value) = cell.borrow().inspect() {
            snapshot
                .heap
                .push((*id, debug_value(module, frames, value, 0, &mut budget)));
        }
    }
    snapshot.native = memory.debug_allocations();
    snapshot.truncated |=
        frames.len() > 64 || heap.len() > 256 || memory.live_allocations() > 128 || budget == 0;
    let stats = heap.statistics();
    snapshot.gc = serde_json::json!({ "allocated": stats.allocated_objects, "live": stats.live_objects,
        "peak": stats.peak_objects, "collections": stats.collections, "reclaimed": stats.reclaimed_objects });
    snapshot.output = output
        .iter()
        .rev()
        .take(32)
        .rev()
        .map(|s| debug_text(s))
        .collect();
    snapshot
}

fn validate_attribute(
    module: &Module,
    attribute: &crate::metadata::CustomAttribute,
) -> Result<(), Fault> {
    let target = &attribute.constructor;
    let owner = target
        .owner
        .as_ref()
        .ok_or_else(|| Fault::new("attribute requires a constructor owner"))?;
    // Attribute metadata is closed even when attached to an open generic definition.
    record_fields(module, owner, 0)?;
    let name = match owner {
        Type::Named(name) => name,
        Type::Constructed { definition, .. } => definition,
        _ => return Err(Fault::new("attribute owner must be a record type")),
    };
    if !target.instance || !target.parameters.is_empty() || target.name != format!("{name}..ctor") {
        return Err(Fault::new(
            "marker attribute requires instance Type::.ctor()",
        ));
    }
    let constructor = resolve(module, target)?;
    if constructor.returns != Type::Void {
        return Err(Fault::new("attribute constructor must return Void"));
    }
    Ok(())
}

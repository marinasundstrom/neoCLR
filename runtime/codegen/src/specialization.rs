//! Bounded type-generic specialization before explicit closed-world selection.
//! Closed value/reference shapes and static/instance method instantiations.
use neoclr::metadata::{Function, FunctionRef, Instruction as Op, Type};
use serde_json::{Value, json};
type Error = Box<dyn std::error::Error>;
use crate::limits::{FUNCTION_CLONES as MAX_FUNCTION_CLONES, TYPES as MAX_SPECIALIZED_TYPES};
#[derive(Clone, PartialEq, Eq)]
struct Instance {
    source: usize,
    types: Vec<Type>,
    methods: Vec<Type>,
    row: usize,
}
#[derive(Clone)]
struct Shape {
    source: usize,
    arguments: Vec<Type>,
    row: usize,
    name: String,
}
// Admit closed nongeneric nominal interface bounds and ordinary runtime categories.
fn check_method_constraints(
    module: &neoclr::Module,
    function: &Function,
    arguments: &[Type],
) -> Result<(), Error> {
    for constraint in &function.generic_constraints {
        let argument = arguments
            .get(constraint.parameter as usize)
            .ok_or("generic method constraint argument missing")?;
        match &constraint.kind {
            category @ (neoclr::metadata::ConstraintKind::ReferenceType
            | neoclr::metadata::ConstraintKind::ValueType
            | neoclr::metadata::ConstraintKind::DefaultConstructor
            | neoclr::metadata::ConstraintKind::FunctionType) => {
                neoclr::native_metadata::validate_method_constraint(module, argument, category)
                    .map_err(|error| format!("method {} category: {error}", function.name))?;
            }
            neoclr::metadata::ConstraintKind::TypeBound(bound @ Type::Named(_))
                if module.type_definition(bound).is_some_and(|t| {
                    t.representation == neoclr::metadata::Representation::Interface
                        && t.generic_parameters.is_empty()
                }) =>
            {
                neoclr::native_metadata::validate_self_interface_bound(module, argument, bound)
                    .map_err(|e| {
                        format!(
                            "method {} bound {bound:?} for {argument:?}: {e}",
                            function.name
                        )
                    })?;
            }
            bound @ neoclr::metadata::ConstraintKind::TypeBound(Type::Named(_)) => {
                neoclr::native_metadata::validate_method_constraint(module, argument, bound)
                    .map_err(|error| format!("method {} class bound: {error}", function.name))?;
            }
            _ => return Err("unsupported generic method constraint".into()),
        }
    }
    Ok(())
}
// Follow the exact closed base chain. Owner parameter ordinals can change at
// every edge, so matching open signatures (or reusing child arguments) is unsafe.
fn closed_virtual_base_contracts(
    input: &neoclr::Module,
    original: &Function,
    arguments: &[Type],
) -> Result<Vec<FunctionRef>, Error> {
    if !super::selection::class_contract(input, original) || !original.is_override {
        return Ok(vec![]);
    }
    let method = super::selection::closed_signature(original, arguments)?;
    let mut owner = method.owner.clone().ok_or("override requires owner")?;
    for _ in 0..64 {
        let definition = input
            .type_definition(&owner)
            .ok_or("override owner unavailable")?;
        let Some(base) = &definition.base else {
            return Ok(vec![]);
        };
        let owner_arguments = match &owner {
            Type::Constructed { arguments, .. } | Type::Scoped { arguments, .. } => {
                arguments.as_slice()
            }
            _ => &[],
        };
        owner = base
            .substitute_type_parameters(owner_arguments)
            .map_err(|e| e.to_string())?;
        let base_arguments = match &owner {
            Type::Constructed { arguments, .. } | Type::Scoped { arguments, .. } => {
                arguments.as_slice()
            }
            _ => &[],
        };
        for candidate in &input.functions {
            if !candidate.instance
                || !candidate.is_virtual
                || candidate.owner.as_ref().and_then(Type::definition_name)
                    != owner.definition_name()
                || candidate.name.rsplit('.').next() != method.name.rsplit('.').next()
            {
                continue;
            }
            let closed = super::selection::closed_signature(candidate, base_arguments)?;
            if closed.parameters == method.parameters && closed.returns == method.returns {
                return Ok(vec![FunctionRef {
                    definition: closed.definition,
                    name: closed.name,
                    owner: closed.owner,
                    instance: true,
                    generic_arguments: vec![],
                    parameters: closed.parameters,
                }]);
            }
        }
    }
    Err("class base nesting exceeds 64".into())
}

struct Specializer<'a> {
    conformance: &'a neoclr::Module,
    source: &'a neoclr::Module,
    functions_by_name: std::collections::HashMap<&'a str, Vec<usize>>,
    shapes: Vec<Shape>,
    type_clones: usize,
    type_depth: usize,
    instances: Vec<Instance>,
    clones: usize,
}
impl Specializer<'_> {
    fn type_index(&self, name: &str) -> Result<usize, Error> {
        let mut found = self
            .source
            .types
            .iter()
            .enumerate()
            .filter(|(_, t)| t.name == name);
        let (i, _) = found
            .next()
            .ok_or("specialization requires local type definitions")?;
        if found.next().is_some() {
            return Err("ambiguous generic type definition".into());
        }
        Ok(i)
    }
    fn close(&mut self, ty: &Type, arguments: &[Type]) -> Result<Type, Error> {
        let ty = ty
            .substitute_type_parameters(arguments)
            .map_err(|e| e.to_string())?;
        self.lower(&ty)
    }
    fn lower(&mut self, ty: &Type) -> Result<Type, Error> {
        if self.type_depth >= 128 {
            return Err("type dependency nesting exceeds 128".into());
        }
        self.type_depth += 1;
        let result = self.lower_inner(ty);
        self.type_depth -= 1;
        result
    }
    fn lower_inner(&mut self, ty: &Type) -> Result<Type, Error> {
        let (name, arguments) = match ty {
            Type::Single
            | Type::Double
            | Type::Int32
            | Type::Byte
            | Type::SByte
            | Type::Int16
            | Type::UInt16
            | Type::Boolean
            | Type::Void
            | Type::Value
            | Type::String
            | Type::Char
            | Type::UInt32
            | Type::Int64
            | Type::UInt64
            | Type::IntPtr
            | Type::UIntPtr
            | Type::RuntimeTypeHandle => {
                return Ok(ty.clone());
            }
            Type::Union(parts) => {
                return Ok(Type::Union(
                    parts
                        .iter()
                        .map(|t| self.lower(t))
                        .collect::<Result<_, _>>()?,
                ));
            }
            Type::Intersection(parts) => {
                return Ok(Type::Intersection(
                    parts
                        .iter()
                        .map(|t| self.lower(t))
                        .collect::<Result<_, _>>()?,
                ));
            }
            Type::Tuple(parts) => {
                let elements = parts
                    .iter()
                    .map(|t| self.lower(t))
                    .collect::<Result<_, _>>()?;
                return Ok(Type::Tuple(
                    neoclr::metadata::TupleSignature::new(elements, parts.names().to_vec())
                        .map_err(|e| e.to_string())?,
                ));
            }
            Type::Function(shape) => {
                let mut shape = shape.as_ref().clone();
                shape.parameters = shape
                    .parameters
                    .iter()
                    .map(|t| self.lower(t))
                    .collect::<Result<_, _>>()?;
                shape.returns = self.lower(&shape.returns)?;
                return Ok(Type::Function(Box::new(shape)));
            }
            Type::ByRef(t) => return Ok(Type::ByRef(Box::new(self.lower(t)?))),
            Type::Array(t) if matches!(**t, Type::Byte | Type::Char) => return Ok(ty.clone()),
            Type::ArrayRef(t)
                if matches!(**t, Type::String | Type::Char)
                    || super::selection::scalar_array_element(t) =>
            {
                if let Some(owner) = super::selection::array_owner(self.source, t) {
                    self.lower(&owner)?;
                }
                return Ok(ty.clone());
            }
            Type::ArrayRef(t)
                if matches!(
                    **t,
                    Type::Function(_)
                        | Type::ArrayRef(_)
                        | Type::Named(_)
                        | Type::Constructed { .. }
                        | Type::Tuple(_)
                        | Type::Union(_)
                        | Type::Intersection(_)
                ) =>
            {
                let lowered = self.lower(t)?;
                if (matches!(**t, Type::ArrayRef(_))
                    || self.source.type_definition(t).is_some_and(|t| {
                        (t.is_reference_type
                            && t.representation == neoclr::metadata::Representation::Record)
                            || t.representation == neoclr::metadata::Representation::Interface
                    }))
                    && let Some(owner) = super::selection::array_owner(self.source, t)
                {
                    self.lower(&owner)?;
                }
                return Ok(Type::ArrayRef(Box::new(lowered)));
            }
            Type::ArrayRef(t) if **t == Type::Byte => {
                if let Some(owner) = super::selection::byte_array_owner(self.source) {
                    self.lower(&owner)?;
                }
                return Ok(ty.clone());
            }
            Type::Named(name) => (name, vec![]),
            Type::Constructed {
                definition,
                arguments,
            } => (definition, arguments.clone()),
            _ => {
                return Err(format!(
                    "specialization requires closed reference-free local value types: {ty:?}"
                )
                .into());
            }
        };
        let i = self.type_index(name)?;
        let definition = &self.source.types[i];
        if definition
            .generic_constraints
            .iter()
            .any(|constraint| match &constraint.kind {
                neoclr::metadata::ConstraintKind::ReferenceType
                | neoclr::metadata::ConstraintKind::ValueType
                | neoclr::metadata::ConstraintKind::DefaultConstructor
                | neoclr::metadata::ConstraintKind::FunctionType => false,
                neoclr::metadata::ConstraintKind::TypeBound(bound) => {
                    !matches!(bound, Type::Named(_))
                        || !self.source.type_definition(bound).is_some_and(|owner| {
                            owner.generic_parameters.is_empty()
                                && (owner.representation
                                    == neoclr::metadata::Representation::Interface
                                    || (owner.is_reference_type
                                        && owner.representation
                                            == neoclr::metadata::Representation::Record))
                        })
                }
                _ => true,
            })
            || definition.generic_parameters.len() != arguments.len()
        {
            return Err(format!("unsupported generic type category, arity or constraints: {name}; reference={}, arity={}/{}, constraints={:?}", definition.is_reference_type, arguments.len(), definition.generic_parameters.len(), definition.generic_constraints).into());
        }
        // Validate argument shapes before allowing a duplicate instantiation through.
        // Bound argument nesting independently of recursive local record layout checks.
        for arg in &arguments {
            validate_argument(arg, 0)?;
        }
        if !definition.generic_constraints.is_empty() {
            // Check the original closed construction before discarding parameters
            // from its executable shape. This also validates nested/base arguments.
            self.source
                .instantiated_fields(ty)
                .map_err(|error| error.to_string())?;
        }
        if let Some(existing) = self
            .shapes
            .iter()
            .find(|v| v.source == i && v.arguments == arguments)
        {
            return Ok(Type::Named(existing.name.clone()));
        }
        if self.shapes.len() >= MAX_SPECIALIZED_TYPES {
            return Err(format!("specialized type count exceeds {MAX_SPECIALIZED_TYPES}").into());
        }
        let row = if self.shapes.iter().any(|v| v.source == i) {
            let row = self.source.types.len() + self.type_clones;
            self.type_clones += 1;
            row
        } else {
            i
        };
        let mut private_name = if row == i {
            name.clone()
        } else {
            format!("{name}$aot_shape_{row}")
        };
        if row != i {
            while self.source.types.iter().any(|t| t.name == private_name)
                || self.shapes.iter().any(|t| t.name == private_name)
            {
                private_name.push('_');
            }
        }
        self.shapes.push(Shape {
            source: i,
            arguments: arguments.clone(),
            row,
            name: private_name.clone(),
        });
        if let Some(owner) = &definition.declaring_type {
            if owner.module != self.source.name || owner.revision != self.source.revision {
                return Err("external lexical owner is unsupported".into());
            }
            let owner = self
                .source
                .types
                .get(owner.index as usize)
                .ok_or("invalid lexical owner")?;
            // CLI-style nested frames redeclare enclosing parameters first.
            // Close that owner before discovering dependent fields so every
            // emitted name and access edge refers to the same specialization.
            if !owner.generic_parameters.is_empty() {
                let count = owner.generic_parameters.len();
                if arguments.len() < count {
                    return Err(
                        "nested specialization requires redeclared enclosing parameters".into(),
                    );
                }
                let closed = self.lower(&Type::Constructed {
                    definition: owner.name.clone(),
                    arguments: arguments[..count].to_vec(),
                })?;
                let Type::Named(parent_name) = closed else {
                    unreachable!()
                };
                let suffix = private_name
                    .strip_prefix(&format!("{}.", owner.name))
                    .ok_or("invalid specialized lexical name")?;
                private_name = format!("{parent_name}.{suffix}");
                self.shapes
                    .iter_mut()
                    .find(|shape| shape.row == row)
                    .unwrap()
                    .name = private_name.clone();
            } else if owner.is_reference_type {
                // Keep lexical identity only. The value profile separately validates
                // empty static companions and rejects every executable use.
                let owner_index = self.type_index(&owner.name)?;
                if !self.shapes.iter().any(|v| v.source == owner_index)
                    && self.shapes.len() >= MAX_SPECIALIZED_TYPES
                {
                    return Err(
                        format!("specialized type count exceeds {MAX_SPECIALIZED_TYPES}").into(),
                    );
                }
                if !self.shapes.iter().any(|v| v.source == owner_index) {
                    self.shapes.push(Shape {
                        source: owner_index,
                        arguments: vec![],
                        row: owner_index,
                        name: owner.name.clone(),
                    });
                }
            } else {
                self.lower(&Type::Named(owner.name.clone()))?;
            }
        }
        for arg in &arguments {
            self.lower(arg)?;
        }
        for field in &definition.fields {
            self.close(&field.ty, &arguments)?;
        }
        for field in &definition.static_fields {
            self.close(&field.ty, &arguments)?;
        }
        if let Some(base) = &definition.base {
            self.close(base, &arguments)?;
        }
        for interface in executable_interface_views(self.source, ty)? {
            self.lower(&interface)?;
        }
        Ok(Type::Named(private_name))
    }
    fn owner(&mut self, ty: &Type, instance: bool) -> Result<Type, Error> {
        if !instance && let Type::Named(name) = ty {
            let i = self.type_index(name)?;
            if super::selection::static_owner(&self.source.types[i]) {
                if !self.shapes.iter().any(|v| v.source == i)
                    && self.shapes.len() >= MAX_SPECIALIZED_TYPES
                {
                    return Err(
                        format!("specialized type count exceeds {MAX_SPECIALIZED_TYPES}").into(),
                    );
                }
                if !self.shapes.iter().any(|v| v.source == i) {
                    self.shapes.push(Shape {
                        source: i,
                        arguments: vec![],
                        row: i,
                        name: name.clone(),
                    });
                }
                return Ok(ty.clone());
            }
        }
        self.lower(ty)
    }
    fn resolve(&mut self, target: &FunctionRef) -> Result<Instance, Error> {
        if target.generic_arguments.len() > 4 {
            return Err("generic method argument count exceeds four".into());
        }
        if target.generic_arguments.iter().any(|t| {
            !matches!(
                t,
                Type::Single
                    | Type::Double
                    | Type::Int32
                    | Type::Byte
                    | Type::SByte
                    | Type::Int16
                    | Type::UInt16
                    | Type::Boolean
                    | Type::Void
                    | Type::String
                    | Type::Char
                    | Type::UInt32
                    | Type::Int64
                    | Type::UInt64
                    | Type::Function(_)
                    | Type::Tuple(_)
                    | Type::Union(_)
                    | Type::Intersection(_)
                    | Type::Named(_)
                    | Type::Constructed { .. }
                    | Type::ArrayRef(_)
            )
        }) {
            return Err(format!(
                "generic method arguments require supported closed shapes: {} {:?}",
                target.name, target.generic_arguments
            )
            .into());
        }
        let arguments = match &target.owner {
            Some(Type::Constructed { arguments, .. }) => arguments.clone(),
            _ => vec![],
        };
        let mut candidates = vec![];
        for &i in self
            .functions_by_name
            .get(target.name.as_str())
            .into_iter()
            .flatten()
        {
            let f = &self.source.functions[i];
            if f.instance != target.instance
                || f.generic_parameters.len() != target.generic_arguments.len()
                || target
                    .definition
                    .as_ref()
                    .is_some_and(|id| Some(id) != f.definition.as_ref())
            {
                continue;
            }
            // Bind against the original nominal signatures before erasing generic shape.
            let owner = f
                .owner
                .as_ref()
                .map(|t| substitute(t, &arguments, &target.generic_arguments))
                .transpose()
                .map_err(|e| e.to_string())?;
            let parameters = f
                .parameters
                .iter()
                .map(|t| substitute(t, &arguments, &target.generic_arguments))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            if owner == target.owner && parameters == target.parameters {
                candidates.push(i);
            }
        }
        let [i] = candidates.as_slice() else {
            return Err(format!(
                "generic call identity/signature is missing or ambiguous: {target:?}"
            )
            .into());
        };
        let i = *i;
        check_method_constraints(
            self.conformance,
            &self.source.functions[i],
            &target.generic_arguments,
        )?;
        if let Some(instance) = self.instances.iter().find(|v| {
            v.source == i && v.types == arguments && v.methods == target.generic_arguments
        }) {
            return Ok(instance.clone());
        }
        if self.instances.len() >= crate::limits::FUNCTIONS
            || (self.clones >= MAX_FUNCTION_CLONES
                && (!target.generic_arguments.is_empty()
                    || self.instances.iter().any(|v| v.source == i)))
        {
            return Err(format!("method specialization exceeds {} selected functions or {} clones ({} functions, {} clones, {} types)", crate::limits::FUNCTIONS, MAX_FUNCTION_CLONES, self.instances.len(), self.clones, self.shapes.len()).into());
        }
        let row = if target.generic_arguments.is_empty()
            && !self.instances.iter().any(|v| v.source == i)
        {
            i
        } else {
            let row = self.source.functions.len() + self.clones;
            self.clones += 1;
            row
        };
        let instance = Instance {
            source: i,
            types: arguments,
            methods: target.generic_arguments.clone(),
            row,
        };
        self.instances.push(instance.clone());
        Ok(instance)
    }
    fn function(
        &mut self,
        f: &Function,
        arguments: &[Type],
        methods: &[Type],
        pending: &mut Vec<Instance>,
    ) -> Result<Function, Error> {
        if f.generic_parameters.len() != methods.len() || !f.generic_arguments.is_empty() {
            return Err("generic method arity or pre-instantiated bodies are unsupported".into());
        }
        check_method_constraints(self.conformance, f, methods)?;
        let mut result = f.clone();
        result.generic_constraints.clear();
        if super::selection::object_dispatch_contract(f) {
            result.body = super::selection::object_placeholder(f);
            result.locals.clear();
            result.local_names.clear();
        }
        result.generic_parameters.clear();
        if let Some(origin) = &mut result.origin {
            // Contracts were consumed on the source generic definition. The closed
            // executable body has no generic parameter slots to own these tokens.
            let tokens = std::mem::take(&mut origin.generic_parameter_tokens);
            result
                .custom_attributes
                .retain(|a| a.target_token.is_none_or(|t| !tokens.contains(&t)));
        }
        result.owner = f
            .owner
            .as_ref()
            .map(|t| {
                let closed = substitute(t, arguments, methods).map_err(|e| e.to_string())?;
                self.owner(&closed, f.instance)
            })
            .transpose()?;
        result.name = lowered_member_name(f, result.owner.as_ref());
        for t in result
            .parameters
            .iter_mut()
            .chain(&mut result.locals)
            .chain([&mut result.returns])
        {
            *t = self.lower(&substitute(t, arguments, methods).map_err(|e| e.to_string())?)?;
        }
        for target in &mut result.interface_implementations {
            let mut closed = target.clone();
            closed.owner = closed
                .owner
                .as_ref()
                .map(|t| substitute(t, arguments, methods))
                .transpose()
                .map_err(|e| e.to_string())?;
            closed.parameters = closed
                .parameters
                .iter()
                .map(|t| substitute(t, arguments, methods))
                .collect::<Result<_, _>>()
                .map_err(|e| e.to_string())?;
            let instance = self.resolve(&closed)?;
            let callee = &self.source.functions[instance.source];
            pending.push(instance.clone());
            target.definition = Some(neoclr::metadata::MemberId {
                module: self.source.name.clone(),
                revision: self.source.revision.clone(),
                index: instance.row as u32,
            });
            target.owner = closed
                .owner
                .as_ref()
                .map(|t| self.owner(t, closed.instance))
                .transpose()?;
            target.name = lowered_member_name(callee, target.owner.as_ref());
            target.parameters = closed
                .parameters
                .iter()
                .map(|t| self.lower(t))
                .collect::<Result<_, _>>()?;
        }
        for op in &mut result.body {
            if let Op::Construct(target) = op {
                let definition = target
                    .owner
                    .as_ref()
                    .and_then(|owner| self.source.type_definition(owner))
                    .and_then(|ty| ty.definition.as_ref());
                if definition.is_some_and(|id| {
                    self.source
                        .assemblies
                        .iter()
                        .any(|a| a.array_backing.as_ref() == Some(id))
                }) {
                    return Err(
                        "array backing requires array allocation, not class construction".into(),
                    );
                }
            }
            if let Op::BindFunction { function_type, .. } = op {
                *function_type = self.lower(
                    &substitute(function_type, arguments, methods).map_err(|e| e.to_string())?,
                )?;
            }
            if let Op::StaticField { owner, .. } | Op::SetStaticField { owner, .. } = op {
                *owner =
                    self.lower(&substitute(owner, arguments, methods).map_err(|e| e.to_string())?)?;
            }
            if let Op::CallSelf {
                borrowed,
                self_type,
                target,
            } = op
            {
                if *borrowed || target.instance {
                    return Err("instance Self specialization is unsupported".into());
                }
                let concrete =
                    substitute(self_type, arguments, methods).map_err(|e| e.to_string())?;
                let mut contract = target.clone();
                contract.owner = contract
                    .owner
                    .as_ref()
                    .map(|t| substitute(t, arguments, methods))
                    .transpose()
                    .map_err(|e| e.to_string())?;
                let implementation = neoclr::native_metadata::resolve_static_self_call(
                    self.conformance,
                    &concrete,
                    &contract,
                )
                .map_err(|e| format!("static Self call {contract:?} for {concrete:?}: {e}"))?;
                *op = Op::Call(FunctionRef {
                    definition: implementation.definition,
                    name: implementation.name,
                    owner: implementation.owner,
                    instance: false,
                    generic_arguments: vec![],
                    parameters: implementation.parameters,
                });
            }
            let virtual_call = matches!(op, Op::CallVirtual(_));
            match op {
                Op::Call(target)
                | Op::CallVirtual(target)
                | Op::Construct(target)
                | Op::BindFunction { target, .. } => {
                    let mut closed = target.clone();
                    closed.owner = target
                        .owner
                        .as_ref()
                        .map(|t| substitute(t, arguments, methods))
                        .transpose()
                        .map_err(|e| e.to_string())?;
                    closed.parameters = target
                        .parameters
                        .iter()
                        .map(|t| substitute(t, arguments, methods))
                        .collect::<Result<_, _>>()
                        .map_err(|e| e.to_string())?;
                    closed.generic_arguments = target
                        .generic_arguments
                        .iter()
                        .map(|t| substitute(t, arguments, methods))
                        .collect::<Result<_, _>>()
                        .map_err(|e| e.to_string())?;
                    if super::selection::callable_invoke(&closed).is_some() {
                        closed.owner = closed.owner.as_ref().map(|t| self.lower(t)).transpose()?;
                        closed.parameters = closed
                            .parameters
                            .iter()
                            .map(|t| self.lower(t))
                            .collect::<Result<_, _>>()?;
                        *target = closed;
                        continue;
                    }
                    let instance = self.resolve(&closed)?;
                    let callee = &self.source.functions[instance.source];
                    if virtual_call
                        && !super::selection::interface_contract(self.source, callee)
                        && !super::selection::class_contract(self.source, callee)
                        && !super::selection::object_dispatch_contract(callee)
                        && !super::selection::sealed_member(self.source, callee)
                        && !super::selection::inherited_object_member(self.source, callee)
                        && (callee.is_virtual || callee.is_abstract || callee.is_override)
                    {
                        return Err(format!("virtual calls requiring dispatch need a later specialization profile: {}", callee.name).into());
                    }
                    if !virtual_call && super::selection::object_dispatch_contract(callee) {
                        return Err("direct Object.ToString/Equals/GetHashCode calls require base-call lowering".into());
                    }
                    target.definition = Some(neoclr::metadata::MemberId {
                        module: self.source.name.clone(),
                        revision: self.source.revision.clone(),
                        index: instance.row as u32,
                    });
                    let method_clone = !instance.methods.is_empty();
                    // Use one private name suffix per argument vector so interface
                    // contracts and implementations retain the same specialized slot.
                    let method_key = self
                        .instances
                        .iter()
                        .find(|candidate| candidate.methods == instance.methods)
                        .unwrap()
                        .row;
                    let source_function = self.source.functions[instance.source].clone();
                    target.generic_arguments.clear();
                    pending.push(instance);
                    target.owner = closed
                        .owner
                        .as_ref()
                        .map(|t| self.owner(t, closed.instance))
                        .transpose()?;
                    target.name = lowered_member_name(&source_function, target.owner.as_ref());
                    if method_clone {
                        target.name = clone_name(&target.name, method_key);
                    }
                    target.parameters = closed
                        .parameters
                        .iter()
                        .map(|t| self.lower(t))
                        .collect::<Result<_, _>>()?;
                }
                Op::New(t)
                | Op::InitializeObject(t)
                | Op::LoadObject(t)
                | Op::StoreObject(t)
                | Op::PackValue(t)
                | Op::IsValue(t)
                | Op::LoadTypeToken(t)
                | Op::UnpackValue(t)
                | Op::BoxValue(t)
                | Op::UnboxAny(t)
                | Op::IsInstance(t)
                | Op::CastClass(t)
                | Op::NewArray(t)
                | Op::ReserveArray(t)
                | Op::ArrayElement(t)
                | Op::StoreArrayElement(t)
                | Op::ArrayAddress(t) => {
                    *t =
                        self.lower(&substitute(t, arguments, methods).map_err(|e| e.to_string())?)?
                }
                _ => (), // Ordinary selected-body admission still rejects unsupported IL.
            }
        }
        Ok(result)
    }
}
fn substitute(t: &Type, types: &[Type], methods: &[Type]) -> Result<Type, neoclr::Fault> {
    t.substitute_method_parameters(methods)?
        .substitute_type_parameters(types)
}
fn lowered_member_name(f: &Function, owner: Option<&Type>) -> String {
    let name = |t: &Type| match t {
        Type::Named(n) | Type::Constructed { definition: n, .. } => Some(n.clone()),
        _ => None,
    };
    if let (Some(old), Some(new)) = (f.owner.as_ref().and_then(name), owner.and_then(name))
        && let Some(member) = f.name.strip_prefix(&format!("{old}."))
    {
        return format!("{new}.{member}");
    }
    f.name.clone()
}
fn clone_name(name: &str, row: usize) -> String {
    format!("{name}$aot_method_{row}")
}

// Self-dependent inherited views belong to concrete owners. Flatten their verified
// closure there; an interface declaration alone cannot choose a Self argument.
fn executable_interface_views(input: &neoclr::Module, owner: &Type) -> Result<Vec<Type>, Error> {
    Ok(neoclr::native_metadata::interface_views(input, owner)
        .map_err(|error| error.to_string())?
        .into_iter()
        .filter(|view| view != owner && !view.contains_self())
        .collect())
}

fn validate_argument(ty: &Type, depth: usize) -> Result<(), Error> {
    if depth > 16 {
        return Err("generic argument nesting exceeds 16".into());
    }
    match ty {
        Type::Single
        | Type::Double
        | Type::Int32
        | Type::UInt32
        | Type::Byte
        | Type::SByte
        | Type::Int16
        | Type::UInt16
        | Type::Boolean
        | Type::Void
        | Type::String
        | Type::Char
        | Type::Int64
        | Type::UInt64
        | Type::IntPtr
        | Type::UIntPtr
        | Type::RuntimeTypeHandle
        | Type::Named(_) => Ok(()),
        Type::ArrayRef(element) => validate_argument(element, depth + 1),
        Type::Function(shape) => {
            for t in shape.parameters.iter().chain([&shape.returns]) {
                validate_argument(t, depth + 1)?;
            }
            Ok(())
        }
        Type::Tuple(elements) => {
            for element in elements.iter() {
                validate_argument(element, depth + 1)?;
            }
            Ok(())
        }
        Type::Union(arguments)
        | Type::Intersection(arguments)
        | Type::Constructed { arguments, .. } => {
            for arg in arguments {
                validate_argument(arg, depth + 1)?;
            }
            Ok(())
        }
        _ => Err(format!("unsupported closed generic argument: {ty:?}").into()),
    }
}

pub fn expand(input: &neoclr::Module, root: &str) -> Result<(neoclr::Module, Value), Error> {
    expand_with_host_roots(input, root, &[])
}
pub fn expand_with_host_roots(
    input: &neoclr::Module,
    root: &str,
    host_roots: &[usize],
) -> Result<(neoclr::Module, Value), Error> {
    expand_with_conformance(input, input, root, host_roots)
}
pub(crate) fn expand_with_conformance(
    input: &neoclr::Module,
    conformance: &neoclr::Module,
    root: &str,
    host_roots: &[usize],
) -> Result<(neoclr::Module, Value), Error> {
    if input.types.len() > crate::limits::INPUT_TYPES
        || input.functions.len() > crate::limits::INPUT_FUNCTIONS
    {
        return Err("specialization input exceeds metadata limits".into());
    }
    // Only the immutable source participates in lookup reuse. The growing
    // expanded inventory is separate, as is any original conformance proof.
    input.with_metadata_lookups(|input| expand_validated(input, conformance, root, host_roots))
}

fn expand_validated(
    input: &neoclr::Module,
    conformance: &neoclr::Module,
    root: &str,
    host_roots: &[usize],
) -> Result<(neoclr::Module, Value), Error> {
    let roots: Vec<_> = input
        .functions
        .iter()
        .enumerate()
        .filter(|(_, f)| f.name == root)
        .map(|(i, _)| i)
        .collect();
    let [root] = roots.as_slice() else {
        return Err("specialization requires one root name".into());
    };
    let root_instance = Instance {
        source: *root,
        types: vec![],
        methods: vec![],
        row: *root,
    };
    // Preserve source order within each overload set. Index only the immutable
    // source inventory; generated instances retain their separate identity checks.
    let mut functions_by_name: std::collections::HashMap<&str, Vec<usize>> =
        std::collections::HashMap::new();
    for (index, function) in input.functions.iter().enumerate() {
        functions_by_name
            .entry(function.name.as_str())
            .or_default()
            .push(index);
    }
    let mut context = Specializer {
        conformance,
        source: input,
        functions_by_name,
        shapes: vec![],
        type_clones: 0,
        type_depth: 0,
        instances: vec![root_instance.clone()],
        clones: 0,
    };
    let mut pending = vec![root_instance];
    for &index in host_roots {
        let f = input
            .functions
            .get(index)
            .ok_or("host root index out of range")?;
        if !f.generic_parameters.is_empty()
            || f.owner.as_ref().is_some_and(|owner| {
                input
                    .type_definition(owner)
                    .is_none_or(|t| !t.generic_parameters.is_empty())
            })
        {
            return Err("host roots require closed nongeneric declarations".into());
        }
        if context
            .instances
            .iter()
            .any(|instance| instance.source == index)
        {
            continue;
        }
        if context.instances.len() >= crate::limits::FUNCTIONS {
            return Err(format!(
                "method specialization exceeds {} selected functions while admitting host roots",
                crate::limits::FUNCTIONS
            )
            .into());
        }
        let instance = Instance {
            source: index,
            types: vec![],
            methods: vec![],
            row: index,
        };
        context.instances.push(instance.clone());
        pending.push(instance);
    }
    let mut visited = std::collections::HashSet::new();
    let mut expanded = input.clone();
    loop {
        while let Some(instance) = pending.pop() {
            if !visited.insert(instance.row) {
                continue;
            }
            for reference in closed_virtual_base_contracts(
                input,
                &input.functions[instance.source],
                &instance.types,
            )? {
                let inherited = context.resolve(&reference)?;
                if !visited.contains(&inherited.row) {
                    pending.push(inherited);
                }
            }
            let mut function = context.function(
                &input.functions[instance.source],
                &instance.types,
                &instance.methods,
                &mut pending,
            )?;
            neoclr::native_metadata::retain_interface_slot(&mut function, input, instance.source);
            if !instance.methods.is_empty() {
                let method_key = context
                    .instances
                    .iter()
                    .find(|candidate| candidate.methods == instance.methods)
                    .unwrap()
                    .row;
                function.name = clone_name(&function.name, method_key);
            }
            if instance.row != instance.source
                && let Some(origin) = &mut function.origin
            {
                // Source tokens identify templates, not each private native body. Keep
                // access/owner facts, assign an unused private token, omit parameter
                // descriptors (custom attributes are removed during selection).
                let last = input
                    .functions
                    .iter()
                    .filter_map(|f| f.origin.as_ref())
                    .map(|o| o.token & 0x00ff_ffff)
                    .max()
                    .unwrap_or(0);
                let rid = last + (instance.row - input.functions.len()) as u32 + 1;
                if rid > 0x00ff_ffff {
                    return Err("private method token range exhausted".into());
                }
                origin.token = 0x0600_0000 | rid;
                origin.parameter_tokens.fill(0);
            }
            function.definition = Some(neoclr::metadata::MemberId {
                module: input.name.clone(),
                revision: input.revision.clone(),
                index: instance.row as u32,
            });
            // Reserve rows in discovery order even when the worklist is processed in reverse.
            while expanded.functions.len() < input.functions.len() + context.clones {
                expanded.functions.push(input.functions[*root].clone());
            }
            expanded.functions[instance.row] = function;
        }
        // Initializers are implicit runtime roots for every retained closed owner.
        for shape in context.shapes.clone() {
            let definition = &input.types[shape.source];
            let owner = if shape.arguments.is_empty() {
                Type::Named(definition.name.clone())
            } else {
                Type::Constructed {
                    definition: definition.name.clone(),
                    arguments: shape.arguments.clone(),
                }
            };
            if let Some(source) = crate::type_initialization::initializer(
                input,
                &crate::type_initialization::owner_type(definition),
            ) {
                let f = &input.functions[source];
                let instance = context.resolve(&FunctionRef {
                    definition: f.definition.clone(),
                    name: f.name.clone(),
                    owner: Some(owner),
                    instance: false,
                    generic_arguments: vec![],
                    parameters: vec![],
                })?;
                if !visited.contains(&instance.row) {
                    pending.push(instance);
                }
            }
        }
        // Recover closed source contracts before erasing their owner arguments.
        // Dispatch may discover new constructors and further closed interfaces.
        let mut constructed = vec![];
        let mut boxed_values = vec![];
        let mut contracts = vec![];
        for instance in &context.instances {
            let original = &input.functions[instance.source];
            let display = super::selection::object_dispatch_contract(original);
            let class = super::selection::class_contract(input, original);
            if super::selection::interface_contract(input, original) || display || class {
                if !original.instance
                    || original.receiver_byref
                    || (!display && !class && !original.body.is_empty())
                    || (!original.generic_parameters.is_empty() && (display || class))
                {
                    return Err(
                        "interface dispatch requires a bodyless instance contract; class virtual methods must be nongeneric"
                            .into(),
                    );
                }
                let mut contract = super::selection::closed_signature(original, &instance.types)?;
                contract.parameters = contract
                    .parameters
                    .iter()
                    .map(|ty| ty.substitute_method_parameters(&instance.methods))
                    .collect::<Result<_, _>>()
                    .map_err(|e| e.to_string())?;
                contract.returns = contract
                    .returns
                    .substitute_method_parameters(&instance.methods)
                    .map_err(|e| e.to_string())?;
                contract.generic_arguments = instance.methods.clone();
                contracts.push(contract);
            }
            if display {
                continue;
            }
            if let Type::ArrayRef(element) =
                substitute(&original.returns, &instance.types, &instance.methods)
                    .map_err(|e| e.to_string())?
                && let Some(owner) = super::selection::array_owner(input, &element)
                && !constructed.contains(&owner)
            {
                constructed.push(owner);
            }
            for op in &original.body {
                if let Op::BoxValue(ty) = op {
                    let owner = substitute(ty, &instance.types, &instance.methods)
                        .map_err(|e| e.to_string())?;
                    if super::selection::projected_primitive(&owner)
                        || input.type_definition(&owner).is_some_and(|t| {
                            !t.is_reference_type
                                && t.representation == neoclr::metadata::Representation::Record
                        })
                    {
                        if !constructed.contains(&owner) {
                            constructed.push(owner.clone());
                        }
                        if !boxed_values.contains(&owner) {
                            boxed_values.push(owner);
                        }
                    }
                }
                if let Op::NewArray(element) | Op::ReserveArray(element) = op {
                    let element = substitute(element, &instance.types, &instance.methods)
                        .map_err(|e| e.to_string())?;
                    if let Some(owner) =
                        super::selection::array_owner(input, &element).filter(|_| {
                            matches!(element, Type::Byte | Type::String | Type::Char)
                                || super::selection::scalar_array_element(&element)
                                || input.type_definition(&element).is_some_and(|t| {
                                    (t.is_reference_type
                                        && t.representation
                                            == neoclr::metadata::Representation::Record)
                                        || t.representation
                                            == neoclr::metadata::Representation::Interface
                                })
                        })
                        && !constructed.contains(&owner)
                    {
                        constructed.push(owner);
                    }
                }
                if let Op::Construct(target) = op {
                    let owner = substitute(
                        target.owner.as_ref().ok_or("constructor requires owner")?,
                        &instance.types,
                        &instance.methods,
                    )
                    .map_err(|e| e.to_string())?;
                    if !constructed.contains(&owner) {
                        constructed.push(owner);
                    }
                }
            }
        }
        for contract in contracts {
            if super::selection::interface_contract(input, &contract)
                && super::selection::implements_interface(
                    input,
                    &Type::String,
                    contract.owner.as_ref().unwrap(),
                )
            {
                let (_, reference) =
                    super::selection::implicit_implementation(input, &Type::String, &contract)?;
                let instance = context.resolve(&reference)?;
                if !visited.contains(&instance.row) {
                    pending.push(instance);
                }
            }
            // Keep conformance for descriptor/signature-only classes as well as
            // executable receivers. Their getters can themselves need closed shapes.
            let mut receivers = constructed.clone();
            if !super::selection::object_dispatch_contract(&contract) {
                for shape in &context.shapes {
                    let ty = &input.types[shape.source];
                    if !ty.is_reference_type
                        || ty.is_abstract
                        || ty.representation != neoclr::metadata::Representation::Record
                    {
                        continue;
                    }
                    let owner = if shape.arguments.is_empty() {
                        Type::Named(ty.name.clone())
                    } else {
                        Type::Constructed {
                            definition: ty.name.clone(),
                            arguments: shape.arguments.clone(),
                        }
                    };
                    if !receivers.contains(&owner) {
                        receivers.push(owner);
                    }
                }
            }
            for owner in &receivers {
                if *owner == Type::String {
                    continue;
                }
                if !super::selection::object_dispatch_contract(&contract)
                    && !super::selection::implements_interface(
                        input,
                        owner,
                        contract.owner.as_ref().unwrap(),
                    )
                {
                    continue;
                }
                // Scalar interface implementations can call generic helpers too. Discover
                // them before selection so their complete bodies are specialized.
                if super::selection::projected_primitive(owner) {
                    if super::selection::interface_contract(input, &contract) {
                        let (_, reference) =
                            super::selection::implicit_implementation(input, owner, &contract)?;
                        let instance = context.resolve(&reference)?;
                        if !visited.contains(&instance.row) {
                            pending.push(instance);
                        }
                    }
                    continue;
                }
                let definition = input
                    .type_definition(owner)
                    .ok_or("constructed interface implementor requires local definition")?;
                // Discover boxed value dispatch before erasing closed generic arguments.
                let value_display = (super::selection::object_dispatch_contract(&contract)
                    || super::selection::interface_contract(input, &contract))
                    && boxed_values.contains(owner);
                // A value constructor alone does not produce an interface receiver.
                if !definition.is_reference_type && !value_display {
                    continue;
                }
                if (!definition.is_reference_type && !value_display)
                    || definition.representation != neoclr::metadata::Representation::Record
                {
                    return Err("interface dispatch requires constructed classes".into());
                }
                if contract.owner.as_ref().is_some_and(|target| {
                    neoclr::native_metadata::interface_has_variance(input, target)
                }) {
                    let view = neoclr::native_metadata::interface_view(
                        input,
                        owner,
                        contract.owner.as_ref().unwrap(),
                    )
                    .map_err(|e| e.to_string())?;
                    if Some(&view) != contract.owner.as_ref() {
                        let template = input
                            .functions
                            .iter()
                            .find(|f| {
                                f.definition == contract.definition && f.name == contract.name
                            })
                            .ok_or("missing original variant contract")?;
                        let arguments = match &view {
                            Type::Constructed { arguments, .. }
                            | Type::Scoped { arguments, .. } => arguments.as_slice(),
                            _ => &[],
                        };
                        let actual = super::selection::closed_signature(template, arguments)?;
                        let peer = context.resolve(&FunctionRef {
                            definition: actual.definition,
                            name: actual.name,
                            owner: actual.owner,
                            instance: actual.instance,
                            generic_arguments: vec![],
                            parameters: actual.parameters,
                        })?;
                        if !visited.contains(&peer.row) {
                            pending.push(peer);
                        }
                    }
                }
                let (_, reference) = if super::selection::object_dispatch_contract(&contract) {
                    let Some(target) =
                        super::selection::object_value_override(input, owner, &contract)?
                    else {
                        continue;
                    };
                    target
                } else if super::selection::class_contract(input, &contract) {
                    super::selection::class_implementation(input, owner, &contract)?
                } else {
                    super::selection::implicit_implementation(input, owner, &contract)?
                };
                let instance = context.resolve(&reference)?;
                if !visited.contains(&instance.row) {
                    pending.push(instance);
                }
            }
        }
        if pending.is_empty() {
            break;
        }
    }
    // Materialize each private shape after discovery. Original metadata is verified
    // before this pass; copied access/readonly facts remain active in the projection.
    let mut at = 0;
    let mut field_rid = input
        .types
        .iter()
        .filter_map(|t| t.origin.as_ref())
        .flat_map(|o| &o.field_tokens)
        .map(|t| t & 0x00ff_ffff)
        .max()
        .unwrap_or(0);
    let mut property_rid = input
        .types
        .iter()
        .filter_map(|t| t.origin.as_ref())
        .flat_map(|o| &o.property_tokens)
        .map(|t| t & 0x00ff_ffff)
        .max()
        .unwrap_or(0);
    let type_rid = input
        .types
        .iter()
        .filter_map(|t| t.origin.as_ref())
        .map(|o| o.token & 0x00ff_ffff)
        .max()
        .unwrap_or(0);
    while at < context.shapes.len() {
        let shape = context.shapes[at].clone();
        at += 1;
        let mut t = input.types[shape.source].clone();
        t.name = shape.name.clone();
        if let Some(owner_id) = &t.declaring_type {
            let owner = &input.types[owner_id.index as usize];
            let count = owner.generic_parameters.len();
            if count != 0 {
                let parent = context
                    .shapes
                    .iter()
                    .find(|candidate| {
                        candidate.source == owner_id.index as usize
                            && candidate.arguments == shape.arguments[..count]
                    })
                    .ok_or("missing specialized lexical owner")?;
                t.declaring_type = Some(neoclr::metadata::TypeDefId {
                    module: input.name.clone(),
                    revision: input.revision.clone(),
                    index: parent.row as u32,
                });
            }
        }
        let source_interface = if shape.arguments.is_empty() {
            Type::Named(input.types[shape.source].name.clone())
        } else {
            Type::Constructed {
                definition: input.types[shape.source].name.clone(),
                arguments: shape.arguments.clone(),
            }
        };
        neoclr::native_metadata::retain_string_view(&mut t, input, &source_interface);
        let native_arguments = shape
            .arguments
            .iter()
            .map(|argument| context.lower(argument))
            .collect::<Result<Vec<_>, _>>()?;
        neoclr::native_metadata::retain_memory_view_shape(
            &mut t,
            input,
            shape.source,
            &native_arguments,
        )
        .map_err(|e| e.to_string())?;
        neoclr::native_metadata::retain_array_shape(&mut t, input, shape.source, &native_arguments)
            .map_err(|e| e.to_string())?;
        neoclr::native_metadata::retain_interface_shape(
            &mut t,
            input,
            shape.source,
            native_arguments,
        )
        .map_err(|e| e.to_string())?;
        t.generic_parameters.clear();
        if let Some(origin) = &mut t.origin {
            let tokens = std::mem::take(&mut origin.generic_parameter_tokens);
            t.custom_attributes
                .retain(|a| a.target_token.is_none_or(|t| !tokens.contains(&t)));
        }
        t.generic_variance.clear();
        t.generic_constraints.clear();
        for field in &mut t.fields {
            field.ty = context.close(&field.ty, &shape.arguments)?;
        }
        for field in &mut t.static_fields {
            field.ty = context.close(&field.ty, &shape.arguments)?;
        }
        t.base = t
            .base
            .as_ref()
            .map(|t| context.close(t, &shape.arguments))
            .transpose()?;
        t.implements = executable_interface_views(input, &source_interface)?
            .iter()
            .map(|t| context.lower(t))
            .collect::<Result<_, _>>()?;
        // Keep construction authority attached to the exact specialized setter.
        // Do not discover methods just because their property exists.
        let mut properties = vec![];
        let mut property_tokens = vec![];
        for (index, property) in t.properties.iter().enumerate().filter(|(_, p)| p.init_only) {
            let setter = property
                .setter
                .as_ref()
                .ok_or("init property lacks setter")?;
            let source = super::selection::resolve(input, setter)?;
            let Some(instance) = context.instances.iter().find(|instance| {
                instance.source == source
                    && instance.types == shape.arguments
                    && instance.methods.is_empty()
                    && visited.contains(&instance.row)
            }) else {
                continue;
            };
            let method = &expanded.functions[instance.row];
            let mut target = setter.clone();
            target.name = method.name.clone();
            target.owner = method.owner.clone();
            target.parameters = method.parameters.clone();
            target.definition = method.definition.clone();
            target.generic_arguments.clear();
            let mut property = property.clone();
            property.ty = method
                .parameters
                .last()
                .ok_or("init setter lacks value")?
                .clone();
            property.parameters = method.parameters[..method.parameters.len() - 1].to_vec();
            property.getter = None;
            property.setter = Some(target);
            properties.push(property);
            if let Some(origin) = &t.origin {
                if shape.row == shape.source {
                    property_tokens.push(origin.property_tokens[index]);
                } else {
                    property_rid += 1;
                    if property_rid > 0x00ff_ffff {
                        return Err("private property token range exhausted".into());
                    }
                    property_tokens.push(0x1700_0000 | property_rid);
                }
            }
        }
        t.properties = properties;
        t.definition = Some(neoclr::metadata::TypeDefId {
            module: input.name.clone(),
            revision: input.revision.clone(),
            index: shape.row as u32,
        });
        if let Some(origin) = &mut t.origin {
            origin.property_tokens = property_tokens;
            if shape.row != shape.source {
                let rid = type_rid + (shape.row - input.types.len()) as u32 + 1;
                if rid > 0x00ff_ffff {
                    return Err("private type token range exhausted".into());
                }
                origin.token = 0x0200_0000 | rid;
                for token in &mut origin.field_tokens {
                    field_rid += 1;
                    if field_rid > 0x00ff_ffff {
                        return Err("private field token range exhausted".into());
                    }
                    *token = 0x0400_0000 | field_rid;
                }
            }
        }
        while expanded.types.len() < input.types.len() + context.type_clones {
            expanded.types.push(input.types[shape.source].clone());
        }
        expanded.types[shape.row] = t;
    }
    for shape in &context.shapes {
        if let Some(parent) = &expanded.types[shape.row].declaring_type {
            let token = expanded.types[parent.index as usize]
                .origin
                .as_ref()
                .map(|origin| origin.token);
            if let Some(origin) = &mut expanded.types[shape.row].origin {
                origin.declaring_type_token = token;
            }
        }
    }
    // The source backing identifies the generic declaration, not whichever closed
    // shape happened to reuse its row first. This private profile only projects
    // Byte views; carry that exact instantiation through row erasure. The original
    // module and its general array-backing contract remain unchanged.
    for assembly in &mut expanded.assemblies {
        if let Some(backing) = &assembly.array_backing {
            let source = input
                .types
                .iter()
                .position(|t| t.definition.as_ref() == Some(backing))
                .ok_or("array backing requires a verified source definition")?;
            assembly.array_backing = context
                .shapes
                .iter()
                .find(|shape| shape.source == source && shape.arguments == [Type::Byte])
                .and_then(|shape| expanded.types[shape.row].definition.clone());
        }
    }
    // Intrinsic storage does not consume nominal shapes. Preserve its verified,
    // reached closed interface views through the same argument-erasure path.
    for intrinsic in std::iter::once(&Type::String).chain(super::selection::SCALAR_PRIMITIVES) {
        let Some(original) = input.type_definition(intrinsic) else {
            continue;
        };
        let Some(projected) = expanded.types.iter_mut().find(|t| t.name == original.name) else {
            continue;
        };
        projected.implements = context
            .shapes
            .iter()
            .filter_map(|shape| {
                let original = &input.types[shape.source];
                if original.representation != neoclr::metadata::Representation::Interface {
                    return None;
                }
                let target = if shape.arguments.is_empty() {
                    Type::Named(original.name.clone())
                } else {
                    Type::Constructed {
                        definition: original.name.clone(),
                        arguments: shape.arguments.clone(),
                    }
                };
                super::selection::implements_interface(input, intrinsic, &target)
                    .then(|| Type::Named(shape.name.clone()))
            })
            .collect();
    }
    let reference_backings: Vec<_> = context
        .shapes
        .iter()
        .filter(|shape| {
            input.assemblies.iter().any(|a| {
                a.array_backing
                    .as_ref()
                    .is_some_and(|id| input.types[shape.source].definition.as_ref() == Some(id))
            }) && shape.arguments.len() == 1
                && (matches!(shape.arguments[0], Type::String | Type::Char)
                    || super::selection::scalar_array_element(&shape.arguments[0])
                    || input.type_definition(&shape.arguments[0]).is_some_and(|t| {
                        (t.is_reference_type
                            && t.representation == neoclr::metadata::Representation::Record)
                            || t.representation == neoclr::metadata::Representation::Interface
                    }))
        })
        .map(|shape| shape.row)
        .collect();
    let report = json!({"referenceArrayBackings":reference_backings,"policy":format!("up to {} closed value/reference/interface shapes, {} function clones and {} selected functions; closed generic methods and lexical owners; reference/value/new and nongeneric nominal class/interface type constraints; function-kind and nongeneric nominal interface method constraints", MAX_SPECIALIZED_TYPES, MAX_FUNCTION_CLONES, crate::limits::FUNCTIONS),
        "typeCount": context.shapes.len(), "functionCount": context.instances.len(), "functionCloneCount": context.clones,
        "methods": context.instances.iter().filter(|v| !v.methods.is_empty() || !v.types.is_empty()).map(|v| json!({"sourceIndex":v.source,"expandedIndex":v.row,"definition":input.functions[v.source].definition.clone().unwrap_or(neoclr::metadata::MemberId { module: input.name.clone(), revision: input.revision.clone(), index: v.source as u32 }),"name":input.functions[v.source].name,"sourceOrigin":input.functions[v.source].origin,"arguments":v.methods,"typeArguments":v.types})).collect::<Vec<_>>(),
        "types": context.shapes.iter().filter(|v| !v.arguments.is_empty()).map(|v| json!({"sourceIndex":v.source,"expandedIndex":v.row,"definition":input.types[v.source].definition,"name":input.types[v.source].name,"compiledName":v.name,"arguments":v.arguments})).collect::<Vec<_>>()});
    Ok((expanded, report))
}

// Restore source identities in diagnostics; appended rows exist only inside codegen.
pub fn restore_methods(report: &mut Value, specialization: &Value) {
    for key in ["functions", "excludedFunctions"] {
        for row in report[key].as_array_mut().unwrap() {
            if let Some(method) = specialization["methods"]
                .as_array()
                .unwrap()
                .iter()
                .find(|m| m["expandedIndex"] == row["sourceIndex"])
            {
                row["expandedIndex"] = row["sourceIndex"].clone();
                row["sourceIndex"] = method["sourceIndex"].clone();
                row["definition"] = method["definition"].clone();
                row["compiledName"] = row["name"].clone();
                row["name"] = method["name"].clone();
                row["methodArguments"] = method["arguments"].clone();
                row["typeArguments"] = method["typeArguments"].clone();
            }
        }
    }
    for key in ["types", "excludedTypes"] {
        for row in report[key].as_array_mut().unwrap() {
            if let Some(shape) = specialization["types"]
                .as_array()
                .unwrap()
                .iter()
                .find(|t| t["expandedIndex"] == row["sourceIndex"])
            {
                row["expandedIndex"] = row["sourceIndex"].clone();
                row["sourceIndex"] = shape["sourceIndex"].clone();
                row["definition"] = shape["definition"].clone();
                row["compiledName"] = row["name"].clone();
                row["name"] = shape["name"].clone();
                row["typeArguments"] = shape["arguments"].clone();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn closed_execution_shapes_drop_source_generic_parameter_slots() {
        use serde_json::json;
        let source = neoclr::assemble(".module GenericOrigins\n.type Holder<T>\n.end\n.function Echo<T>(T value) -> T\nldarg value\nret\n.end\n.function Main() -> Int32\n.local Holder<Int32> holder\nldc.i4 42\ncall Echo<Int32>(Int32)\nret\n.end").unwrap();
        let mut value = serde_json::to_value(source).unwrap();
        value["assemblies"] = json!([{"name":"GenericOrigins","full_name":"GenericOrigins","modules":["GenericOrigins.dll"],"references":[]}]);
        value["types"][0]["origin"] = json!({"assembly":"GenericOrigins","module":"GenericOrigins.dll","name":"Holder`1","token":0x02000001,"generic_parameter_tokens":[0x2a000001]});
        value["functions"][0]["origin"] = json!({"assembly":"GenericOrigins","module":"GenericOrigins.dll","name":"Echo","token":0x06000001,"parameter_tokens":[0],"generic_parameter_tokens":[0x2a000002]});
        let input = serde_json::from_value(value).unwrap();
        let (selected, _) = crate::selection::prepare(&input, "Main").unwrap();
        neoclr::verify(&selected).unwrap();
        assert!(
            selected
                .types
                .iter()
                .filter_map(|t| t.origin.as_ref())
                .all(|o| o.generic_parameter_tokens.is_empty())
        );
        assert!(
            selected
                .functions
                .iter()
                .filter_map(|f| f.origin.as_ref())
                .all(|o| o.generic_parameter_tokens.is_empty())
        );
    }

    #[test]
    #[ignore = "focused specialization timing probe; run explicitly with --nocapture"]
    fn specialization_lookup_timing() {
        let mut text = String::from(".module Lookup\n");
        for i in 0..4096 {
            text.push_str(&format!(
                ".function F{i}(Int32 value) -> Int32\nldarg value\n"
            ));
            if i < 1023 {
                text.push_str(&format!("call F{}(Int32)\n", i + 1));
            }
            text.push_str("ret\n.end\n");
        }
        let source = neoclr::assemble(&text).unwrap();
        for _ in 0..5 {
            let started = std::time::Instant::now();
            let (expanded, _) = super::expand(&source, "F0").unwrap();
            assert_eq!(expanded.functions.len(), 4096);
            println!(
                "specialization_lookup_seconds={:.6}",
                started.elapsed().as_secs_f64()
            );
        }
    }

    #[test]
    fn boxed_scalar_interface_body_specializes_generic_helpers() {
        let source = neoclr::assemble(
            r#"
.module System
.type class System.Object
.end
.interface Format
.method instance Render() -> Int32
.end
.end
.type System.Int32
.implements Format
.method instance byref Render() -> Int32
ldc.i4 42
call Echo<Int32>(Int32)
ret
.end
.end
.function Echo<T>(T value) -> T
ldarg value
ret
.end
.function Main() -> Int32
ldc.i4 1
box Int32
castclass Format
callvirt instance Format::Render()
ret
.end
"#,
        )
        .unwrap();
        // Selection must discover the scalar interface body before specializing
        // its generic helper. This isolated core fixture is not an application.
        super::super::selection::prepare(&source, "Main").unwrap();
    }

    #[test]
    fn memory_view_shapes_retain_exact_elements_without_admitting_native_layout() {
        use neoclr::metadata::{MemoryViewKind, Type};
        let source = neoclr::assemble(
            r#"
.module System
.type System.Slice<T>
.memoryview slice
.end
.type System.ReadOnlySlice<T>
.memoryview read_only_slice
.end
.type System.Memory<T>
.memoryview memory
.end
.type System.ReadOnlyMemory<T>
.memoryview read_only_memory
.end
.type class Element
.end
.function Main() -> Int32
.local System.Slice<Int32> writable
.local System.Slice<Byte> bytes
.local System.ReadOnlySlice<Byte> readonly
.local System.Memory<Element> retained
.local System.ReadOnlyMemory<Element> retainedReadonly
ldc.i4 0
ret
.end
"#,
        )
        .unwrap();
        neoclr::verify(&source).unwrap();
        let (expanded, _) = expand(&source, "Main").unwrap();
        let shapes: Vec<_> = expanded
            .types
            .iter()
            .filter_map(|t| {
                neoclr::native_metadata::memory_view_element(t).map(|element| {
                    assert!(t.generic_parameters.is_empty());
                    (t.memory_view.unwrap(), element.clone())
                })
            })
            .collect();
        assert_eq!(shapes.len(), 5);
        assert!(shapes.contains(&(MemoryViewKind::Slice, Type::Int32)));
        assert!(shapes.contains(&(MemoryViewKind::Slice, Type::Byte)));
        assert!(shapes.contains(&(MemoryViewKind::ReadOnlySlice, Type::Byte)));
        let (_, element) = shapes
            .iter()
            .find(|(kind, _)| *kind == MemoryViewKind::Memory)
            .unwrap();
        assert!(shapes.contains(&(MemoryViewKind::ReadOnlyMemory, element.clone())));
        assert!(matches!(element, Type::Named(_)));
        assert!(expanded.type_definition(element).is_some());

        assert!(
            crate::selection::select(&expanded, "Main")
                .unwrap_err()
                .to_string()
                .contains("native ABI")
        );
    }

    #[test]
    fn nominal_self_inheritance_closes_views_at_the_implementing_owner() {
        let text = include_str!("../../../docs/validation/numerics-interfaces/StaticSelf.neoil")
            .replace(
                ".interface Number\n",
                ".interface Witness<T>\n.end\n.interface Number\n.implements Witness<Self>\n",
            );
        let source = neoclr::assemble(&text).unwrap();
        let original = serde_json::to_value(&source).unwrap();
        let (expanded, _) = super::expand(&source, "Main").unwrap();
        let count = expanded.types.iter().find(|t| t.name == "Count").unwrap();
        assert!(count.implements.iter().any(|view| {
            view.definition_name()
                .is_some_and(|n| n.starts_with("Witness"))
        }));
        assert!(
            expanded
                .types
                .iter()
                .all(|t| t.implements.iter().all(|view| !view.contains_self()))
        );
        assert_eq!(serde_json::to_value(&source).unwrap(), original);
        let (selected, _) = super::super::selection::select(&expanded, "Main").unwrap();
        assert_eq!(
            neoclr::LoadedProgram::new(&selected)
                .unwrap()
                .run(neoclr::Limits::default())
                .unwrap()
                .value,
            neoclr::Value::Int32(42)
        );
    }

    #[test]
    fn nominal_static_self_methods_specialize_without_erasing_conformance() {
        let source = neoclr::assemble(include_str!(
            "../../../docs/validation/numerics-interfaces/StaticSelf.neoil"
        ))
        .unwrap();
        let original = serde_json::to_value(&source).unwrap();
        let (expanded, report) = super::expand(&source, "Main").unwrap();
        let (linear, linear_report) =
            super::expand_validated(&source, &source, "Main", &[]).unwrap();
        assert_eq!(
            serde_json::to_value(&expanded).unwrap(),
            serde_json::to_value(&linear).unwrap()
        );
        assert_eq!(report, linear_report);
        let (selected, _) = super::super::selection::select(&expanded, "Main").unwrap();
        let program = neoclr::LoadedProgram::new(&selected).unwrap();
        program.verify().unwrap();
        assert_eq!(
            program.run(neoclr::Limits::default()).unwrap().value,
            neoclr::Value::Int32(42)
        );
        assert_eq!(serde_json::to_value(&source).unwrap(), original);
        assert!(selected.functions.iter().all(|f| {
            f.generic_constraints.is_empty()
                && !f
                    .body
                    .iter()
                    .any(|op| matches!(op, super::Op::CallSelf { .. }))
        }));
        // Native linking keeps value conformance as proof, not executable interface views.
        let mut projected = source.clone();
        projected
            .types
            .iter_mut()
            .find(|t| t.name == "Count")
            .unwrap()
            .implements
            .clear();
        let (expanded, report) =
            super::expand_with_conformance(&projected, &source, "Main", &[]).unwrap();
        let (linear, linear_report) =
            super::expand_validated(&projected, &source, "Main", &[]).unwrap();
        assert_eq!(
            serde_json::to_value(&expanded).unwrap(),
            serde_json::to_value(&linear).unwrap()
        );
        assert_eq!(report, linear_report);
        let (selected, _) = super::super::selection::select(&expanded, "Main").unwrap();
        assert_eq!(
            neoclr::LoadedProgram::new(&selected)
                .unwrap()
                .run(neoclr::Limits::default())
                .unwrap()
                .value,
            neoclr::Value::Int32(42)
        );
        // Mutate linked metadata directly: do not rely on source-language diagnostics.
        for mutation in 0..4 {
            let mut invalid = source.clone();
            match mutation {
                0 => invalid
                    .types
                    .iter_mut()
                    .find(|t| t.name == "Count")
                    .unwrap()
                    .implements
                    .clear(),
                1 => {
                    invalid
                        .functions
                        .iter_mut()
                        .find(|f| f.name == "Count.Add")
                        .unwrap()
                        .visibility = neoclr::metadata::Visibility::Private
                }
                2 => {
                    invalid
                        .functions
                        .iter_mut()
                        .find(|f| f.name == "Count.Add")
                        .unwrap()
                        .returns = super::Type::Int32
                }
                _ => {
                    invalid
                        .functions
                        .iter_mut()
                        .find(|f| f.name == "Sum")
                        .unwrap()
                        .generic_constraints[0]
                        .kind = neoclr::metadata::ConstraintKind::TypeBound(super::Type::Named(
                        "Missing".into(),
                    ))
                }
            }
            assert_eq!(
                super::expand(&invalid, "Main").unwrap_err().to_string(),
                super::expand_validated(&invalid, &invalid, "Main", &[])
                    .unwrap_err()
                    .to_string(),
                "mutation {mutation} changed diagnostics"
            );
        }
    }

    #[test]
    fn function_kind_constraint_preserves_typed_calls_after_specialization() {
        let source = neoclr::assemble(include_str!(
            "../../../docs/validation/function-kind-constraint/Main.neoil"
        ))
        .unwrap();
        let original = serde_json::to_value(&source).unwrap();
        let (expanded, _) = super::expand(&source, "Main").unwrap();
        let (selected, _) = super::super::selection::select(&expanded, "Main").unwrap();
        let program = neoclr::LoadedProgram::new(&selected).unwrap();
        program.verify().unwrap();
        assert_eq!(
            program.run(neoclr::Limits::default()).unwrap().value,
            neoclr::Value::Int32(42)
        );
        assert_eq!(serde_json::to_value(&source).unwrap(), original);
        assert!(
            selected
                .functions
                .iter()
                .all(|f| f.generic_constraints.is_empty())
        );
        let constrained = source.functions.iter().find(|f| f.name == "Keep").unwrap();
        assert!(
            super::check_method_constraints(&source, constrained, &[super::Type::Int32]).is_err()
        );
        assert!(
            super::check_method_constraints(
                &source,
                constrained,
                &[super::Type::MethodTypeParameter(0)]
            )
            .is_err()
        );
        assert!(super::check_method_constraints(&source, constrained, &[]).is_err());
        let mut invalid = neoclr::assemble(".module Invalid\n.entry Main\n.function Keep<F>(F) -> F\nldarg 0\nret\n.end\n.function Main() -> Int32\nldc.i4 42\ncall Keep<Int32>(Int32)\nret\n.end").unwrap();
        invalid.functions[0].generic_constraints = constrained.generic_constraints.clone();
        assert!(
            super::expand(&invalid, "Main")
                .unwrap_err()
                .to_string()
                .contains("constraint")
        );
    }

    #[test]
    fn method_class_bounds_validate_before_specialization() {
        let source = neoclr::assemble(".module Bounds\n.entry Main\n.type class Base\n.end\n.type class Child\n.extends Base\n.end\n.function Answer<T>() -> Int32\n.constraint T Base\nldc.i4 42\nret\n.end\n.function Main() -> Int32\ncall Answer<Child>()\nret\n.end").unwrap();
        let (expanded, _) = super::expand(&source, "Main").unwrap();
        let (selected, _) = super::super::selection::select(&expanded, "Main").unwrap();
        assert_eq!(
            neoclr::LoadedProgram::new(&selected)
                .unwrap()
                .run(neoclr::Limits::default())
                .unwrap()
                .value,
            neoclr::Value::Int32(42)
        );
        let method = source
            .functions
            .iter()
            .find(|f| f.name == "Answer")
            .unwrap();
        assert!(super::check_method_constraints(&source, method, &[super::Type::Int32]).is_err());
        assert!(
            super::check_method_constraints(
                &source,
                method,
                &[super::Type::MethodTypeParameter(0)]
            )
            .is_err()
        );
    }

    #[test]
    fn method_categories_use_runtime_constraint_validation() {
        use neoclr::metadata::{ConstraintKind, GenericConstraint};
        let source = neoclr::assemble(".module Categories\n.entry Main\n.function Keep<T>(T) -> T\nldarg 0\nret\n.end\n.function Main() -> Int32\nldc.i4 42\ncall Keep<Int32>(Int32)\nret\n.end").unwrap();
        for (kind, accepted) in [
            (ConstraintKind::ValueType, true),
            (ConstraintKind::DefaultConstructor, true),
            (ConstraintKind::ReferenceType, false),
        ] {
            let mut module = source.clone();
            module.functions[0].generic_constraints =
                vec![GenericConstraint { parameter: 0, kind }];
            let result = super::expand(&module, "Main");
            assert_eq!(result.is_ok(), accepted);
            if let Ok((expanded, _)) = result {
                let (selected, _) = super::super::selection::select(&expanded, "Main").unwrap();
                assert_eq!(
                    neoclr::LoadedProgram::new(&selected)
                        .unwrap()
                        .run(neoclr::Limits::default())
                        .unwrap()
                        .value,
                    neoclr::Value::Int32(42)
                );
            }
            assert!(
                super::check_method_constraints(
                    &module,
                    &module.functions[0],
                    &[super::Type::MethodTypeParameter(0)]
                )
                .is_err()
            );
            assert!(
                super::check_method_constraints(
                    &module,
                    &module.functions[0],
                    &[super::Type::Named("Missing".into())]
                )
                .is_err()
            );
        }
    }

    #[test]
    fn special_type_constraints_are_checked_before_specialization() {
        let source = neoclr::assemble(include_str!(
            "../../../docs/validation/special-type-constraints/Main.neoil"
        ))
        .unwrap();
        let original = serde_json::to_value(&source).unwrap();
        neoclr::verify(&source).unwrap();
        let (expanded, _) = super::expand(&source, "Main").unwrap();
        let (selected, _) = super::super::selection::select(&expanded, "Main").unwrap();
        let program = neoclr::LoadedProgram::new(&selected).unwrap();
        program.verify().unwrap();
        assert_eq!(
            program.run(neoclr::Limits::default()).unwrap().value,
            neoclr::Value::Int32(42)
        );
        assert_eq!(serde_json::to_value(&source).unwrap(), original);
        assert!(
            selected
                .types
                .iter()
                .all(|t| t.generic_constraints.is_empty())
        );
        // The value box must not satisfy reference requirements, and a reference
        // argument must not satisfy value requirements. Constructor visibility
        // and abstractness are checked against the original declaration.
        for case in 0..4 {
            let mut invalid = source.clone();
            match case {
                0 => {
                    invalid
                        .types
                        .iter_mut()
                        .find(|t| t.name == "ValueBox")
                        .unwrap()
                        .generic_constraints[0]
                        .kind = neoclr::metadata::ConstraintKind::ReferenceType
                }
                1 => {
                    invalid
                        .types
                        .iter_mut()
                        .find(|t| t.name == "ReferenceBox")
                        .unwrap()
                        .generic_constraints[0]
                        .kind = neoclr::metadata::ConstraintKind::ValueType
                }
                2 => {
                    invalid
                        .functions
                        .iter_mut()
                        .find(|f| f.name.ends_with("..ctor"))
                        .unwrap()
                        .visibility = neoclr::metadata::Visibility::Private
                }
                _ => {
                    invalid
                        .types
                        .iter_mut()
                        .find(|t| t.name == "Payload")
                        .unwrap()
                        .is_abstract = true
                }
            }
            assert!(
                super::expand(&invalid, "Main")
                    .unwrap_err()
                    .to_string()
                    .contains("violates")
            );
        }
    }

    #[test]
    fn nominal_interface_owner_bounds_are_checked_before_specialization() {
        let text = include_str!("../../../docs/validation/special-type-constraints/Main.neoil")
            .replace(
                ".type class Payload",
                ".interface Failure\n.end\n.type class Payload\n.implements Failure",
            )
            .replace(
                ".constraint T reference new",
                ".constraint T reference new Failure",
            );
        let source = neoclr::assemble(&text).unwrap();
        let (expanded, _) = super::expand(&source, "Main").unwrap();
        let (selected, _) = super::super::selection::select(&expanded, "Main").unwrap();
        let program = neoclr::LoadedProgram::new(&selected).unwrap();
        program.verify().unwrap();
        assert_eq!(
            program.run(neoclr::Limits::default()).unwrap().value,
            neoclr::Value::Int32(42)
        );
        let mut invalid = source.clone();
        invalid
            .types
            .iter_mut()
            .find(|t| t.name == "Payload")
            .unwrap()
            .implements
            .clear();
        assert!(super::expand(&invalid, "Main").is_err());
    }

    #[test]
    fn nominal_class_constraints_are_checked_before_specialization() {
        let text = include_str!("../../../docs/validation/special-type-constraints/Main.neoil")
            .replace(
                ".constraint T reference new",
                ".constraint T reference new Payload",
            );
        let source = neoclr::assemble(&(text + "\n.type class Other\n.end\n")).unwrap();
        let original = serde_json::to_value(&source).unwrap();
        let (expanded, _) = super::expand(&source, "Main").unwrap();
        let (selected, _) = super::super::selection::select(&expanded, "Main").unwrap();
        let program = neoclr::LoadedProgram::new(&selected).unwrap();
        program.verify().unwrap();
        assert_eq!(
            program.run(neoclr::Limits::default()).unwrap().value,
            neoclr::Value::Int32(42)
        );
        assert_eq!(serde_json::to_value(&source).unwrap(), original);
        let mut invalid = source.clone();
        let owner = invalid
            .types
            .iter_mut()
            .find(|t| t.name == "ReferenceBox")
            .unwrap();
        let bound = owner
            .generic_constraints
            .iter_mut()
            .find(|c| matches!(c.kind, neoclr::metadata::ConstraintKind::TypeBound(_)))
            .unwrap();
        bound.kind =
            neoclr::metadata::ConstraintKind::TypeBound(super::Type::Named("Other".into()));
        assert!(
            super::expand(&invalid, "Main")
                .unwrap_err()
                .to_string()
                .contains("violates")
        );
    }

    #[test]
    fn generic_class_override_without_base_call_retains_closed_slot() {
        let source = neoclr::assemble(
            r#"
.module GenericVirtual
.entry Main
.type class Base<T>
.method instance virtual Echo(T) -> T
ldarg 1
ret
.end
.end
.type class Derived<Unused,T>
.extends Base<T>
.field Value T
.method instance override Echo(T) -> T
ldarg 0
ldfld Derived<Unused,T>::Value
ret
.end
.end
.function Main() -> Int32
ldc.i4 42
newobj Derived<Boolean,Int32>
ldc.i4 99
callvirt instance Derived<Boolean,Int32>::Echo(Int32)
ret
.end
"#,
        )
        .unwrap();
        let (expanded, _) = expand(&source, "Main").unwrap();
        let (selected, _) = super::super::selection::select(&expanded, "Main").unwrap();
        let program = neoclr::LoadedProgram::new(&selected).unwrap();
        program.verify().unwrap();
        assert_eq!(
            program.run(neoclr::Limits::default()).unwrap().value,
            neoclr::Value::Int32(42)
        );
    }

    #[test]
    fn variant_interfaces_preserve_dispatch_and_identity_after_specialization() {
        let input = neoclr::assemble(include_str!(
            "../../../docs/validation/interface-variance/Main.neoil"
        ))
        .unwrap();
        let (expanded, _) = super::expand(&input, "Main").unwrap();
        let (selected, _) = super::super::selection::select(&expanded, "Main").unwrap();
        let program = neoclr::LoadedProgram::new(&selected).unwrap();
        program.verify().unwrap();
        let outcome = program.run(neoclr::Limits::default()).unwrap();
        assert_eq!(outcome.value, neoclr::Value::Int32(42));
    }

    #[test]
    fn generic_interface_calls_with_identical_erased_signatures_remain_distinct() {
        let source = include_str!("../../../examples/generic-interfaces.neoil")
            .replace("Apply<T>(T value) -> T", "Apply<T>() -> Int32")
            .replace("ldarg value", "ldc.i4 21")
            .replace("ldc.i4 42\ncallvirt instance Echo::Apply<Int32>(Int32)",
                "callvirt instance Echo::Apply<Int32>()\nnewobj Identity\ncastclass Echo\ncallvirt instance Echo::Apply<String>()\nadd");
        let input = neoclr::assemble(&source).unwrap();
        let (selected, _) = super::super::selection::prepare(&input, "Main").unwrap();
        let program = neoclr::LoadedProgram::new(&selected).unwrap();
        program.verify().unwrap();
        assert_eq!(
            program.run(neoclr::Limits::default()).unwrap().value,
            neoclr::Value::Int32(42)
        );
    }

    #[test]
    fn generic_interface_dispatch_closes_method_state() {
        let input =
            neoclr::assemble(include_str!("../../../examples/generic-interfaces.neoil")).unwrap();
        let (selected, _) = super::super::selection::prepare(&input, "Main").unwrap();
        let program = neoclr::LoadedProgram::new(&selected).unwrap();
        program.verify().unwrap();
        assert_eq!(
            program.run(neoclr::Limits::default()).unwrap().value,
            neoclr::Value::Int32(42)
        );
    }

    #[test]
    fn variant_interface_dispatch_closes_generic_implementations() {
        let input = neoclr::assemble(
            r#".module GenericVariance
.entry Main
.type class Base
.end
.type class Derived
.extends Base
.end
.interface Producer<T>
.variance out
.method instance Get() -> T
.end
.end
.type class Box<T>
.implements Producer<T>
.field Item T
.method instance Get() -> T
ldarg this
ldfld 0
ret
.end
.end
.function Main() -> Int32
newobj Derived
newobj Box<Derived>
castclass Producer<Base>
callvirt instance Producer<Base>::Get()
pop
ldc.i4 42
ret
.end"#,
        )
        .unwrap();
        let (selected, _) = super::super::selection::prepare(&input, "Main").unwrap();
        let program = neoclr::LoadedProgram::new(&selected).unwrap();
        program.verify().unwrap();
        assert_eq!(
            program.run(neoclr::Limits::default()).unwrap().value,
            neoclr::Value::Int32(42)
        );
        // Serialized artifacts cannot install the native compiler's trusted proof.
        let mut json = serde_json::to_value(&input).unwrap();
        json["types"][0]["native_interface"] = serde_json::json!({"arguments": []});
        assert!(serde_json::from_value::<neoclr::Module>(json).is_err());
    }

    use super::*;

    #[test]
    fn instance_methods_specialize_owner_and_method_parameters_independently() {
        let input = neoclr::assemble(
            r#"
.module GenericOwner
.entry Main
.type class Box<T>
.method instance .ctor() -> void
ret
.end
.method instance Choose<U>(T first, U second) -> U
ldarg 2
ret
.end
.end
.function Main() -> Int32
newobj instance Box<Int32>::.ctor()
ldc.i4 1
ldstr "method"
call instance Box<Int32>::Choose<String>(Int32,String)
pop
newobj instance Box<String>::.ctor()
ldstr "owner"
ldc.i4 42
call instance Box<String>::Choose<Int32>(String,Int32)
ret
.end
"#,
        )
        .unwrap();
        neoclr::verify(&input).unwrap();
        let (expanded, _) = expand(&input, "Main").unwrap();
        let (selected, _) = super::super::selection::select(&expanded, "Main").unwrap();
        neoclr::verify(&selected).unwrap();
        assert_eq!(
            neoclr::LoadedProgram::new(&selected)
                .unwrap()
                .run(neoclr::Limits::default())
                .unwrap()
                .value,
            neoclr::Value::Int32(42)
        );
    }

    #[test]
    fn nested_generic_specializations_preserve_each_lexical_owner() {
        let input = neoclr::assemble(
            r#"
.module NestedGeneric
.entry Main
.type class Owner<T>
.field private value T
.method instance .ctor(T value) -> void
ldarg 0
ldarg 1
stfld Owner<T>::value
ret
.end
.type class Frame<T,U>
.method static Read(Owner<T> owner, U other) -> T
ldarg 0
ldfld Owner<T>::value
ret
.end
.end
.end
.function Main() -> Int32
ldstr "owner"
newobj instance Owner<String>::.ctor(String)
ldc.i4 1
call Owner.Frame<String,Int32>::Read(Owner<String>,Int32)
pop
ldc.i4 42
newobj instance Owner<Int32>::.ctor(Int32)
ldstr "other"
call Owner.Frame<Int32,String>::Read(Owner<Int32>,String)
ret
.end
"#,
        )
        .unwrap();
        neoclr::verify(&input).unwrap();
        let (expanded, _) = expand(&input, "Main").unwrap();
        let (selected, _) = super::super::selection::select(&expanded, "Main").unwrap();
        neoclr::verify(&selected).unwrap();
        assert_eq!(
            neoclr::LoadedProgram::new(&selected)
                .unwrap()
                .run(neoclr::Limits::default())
                .unwrap()
                .value,
            neoclr::Value::Int32(42)
        );
    }

    #[test]
    fn variant_selection_ignores_unrelated_explicit_source_templates() {
        let source = neoclr::assemble(
            r#"
    .module UnrelatedMapping
    .entry Main
    .type class System.Object
    .end
    .interface Producer<T>
    .variance out
    .method instance Get() -> T
    .end
    .end
    .interface Other<T>
    .method instance Count() -> Int32
    .end
    .end
    .type class Source
    .implements Producer<String>
    .implements Other<Int32>
    .method private instance Count() -> Int32
    .override instance Other<Int32>::Count()
    ldc.i4 1
    ret
    .end
    .method instance Get() -> String
    ldstr "text"
    ret
    .end
    .end
    .function Main() -> Int32
    newobj Source
    castclass Producer<System.Object>
    callvirt instance Producer<System.Object>::Get()
    pop
    ldc.i4 42
    ret
    .end
    "#,
        )
        .unwrap();
        let (expanded, _) = expand(&source, "Main").unwrap();
        let (selected, _) = super::super::selection::select(&expanded, "Main").unwrap();
        let program = neoclr::LoadedProgram::new(&selected).unwrap();
        program.verify().unwrap();
        assert_eq!(
            program.run(neoclr::Limits::default()).unwrap().value,
            neoclr::Value::Int32(42)
        );
    }

    #[test]
    fn signature_only_classes_retain_closed_interface_conformance() {
        let input = neoclr::assemble(
            ".module SignatureConformance
.entry Main
.interface Readable<T>
.method instance Read() -> T
.end
.end
.type class Descriptor<T>
.implements Readable<T>
.field Value T
.method instance Read() -> T
ldarg 0
ldfld Descriptor<T>::Value
ret
.end
.method instance Unused() -> Int32
ldc.i4 99
ret
.end
.end
.function Main() -> Int32
.local Descriptor<Int32> descriptor
ldloca descriptor
initobj Descriptor<Int32>
ldc.bool false
brfalse Done
ldloc descriptor
castclass Readable<Int32>
callvirt instance Readable<Int32>::Read()
ret
Done:
ldc.i4 42
ret
.end",
        )
        .unwrap();
        neoclr::verify(&input).unwrap();
        let (expanded, _) = expand(&input, "Main").unwrap();
        let (selected, report) = super::super::selection::select(&expanded, "Main").unwrap();
        neoclr::verify(&selected).unwrap();
        assert!(
            selected
                .functions
                .iter()
                .any(|f| f.name == "Descriptor.Read")
        );
        assert!(
            !selected
                .functions
                .iter()
                .any(|f| f.name.ends_with(".Unused"))
        );
        assert!(
            report["interfaceDispatch"]
                .as_array()
                .unwrap()
                .iter()
                .all(|r| r["targets"].as_array().unwrap().is_empty())
        );
        assert_eq!(
            neoclr::LoadedProgram::new(&selected)
                .unwrap()
                .run(neoclr::Limits::default())
                .unwrap()
                .value,
            neoclr::Value::Int32(42)
        );
    }

    #[test]
    fn specialized_interface_dispatch_ignores_unselected_open_overload() {
        let input = neoclr::assemble(
            ".module SignatureConformance
.entry Main
.interface Readable<T>
.method instance Read() -> T
.end
.end
.type class Descriptor<T>
.implements Readable<T>
.field Value T
.method instance Read() -> T
ldarg 0
ldfld Descriptor<T>::Value
ret
.end
.method instance Read(Int32 unused) -> T
ldarg 0
ldfld Descriptor<T>::Value
ret
.end
.end
.function Main() -> Int32
.local Descriptor<Int32> descriptor
ldloca descriptor
initobj Descriptor<Int32>
ldc.bool false
brfalse Done
ldloc descriptor
castclass Readable<Int32>
callvirt instance Readable<Int32>::Read()
ret
Done:
ldc.i4 42
ret
.end",
        )
        .unwrap();
        neoclr::verify(&input).unwrap();
        let (expanded, _) = expand(&input, "Main").unwrap();
        let (selected, report) = super::super::selection::select(&expanded, "Main").unwrap();
        neoclr::verify(&selected).unwrap();
        assert!(
            selected
                .functions
                .iter()
                .any(|f| f.name == "Descriptor.Read")
        );
        assert!(
            !selected
                .functions
                .iter()
                .any(|f| f.name == "Descriptor.Read" && !f.parameters.is_empty())
        );
        assert!(
            report["interfaceDispatch"]
                .as_array()
                .unwrap()
                .iter()
                .all(|r| r["targets"].as_array().unwrap().is_empty())
        );
        assert_eq!(
            neoclr::LoadedProgram::new(&selected)
                .unwrap()
                .run(neoclr::Limits::default())
                .unwrap()
                .value,
            neoclr::Value::Int32(42)
        );
    }

    #[test]
    fn deeply_nested_type_dependencies_fail_without_overflowing_the_host_stack() {
        let mut source =
            String::from(".module Deep\n.function Main(T0 value) -> Int32\nldc.i4 0\nret\n.end\n");
        for index in 0..140 {
            source += &format!(".type class T{index}\n");
            if index < 139 {
                source += &format!(".field Next T{}\n", index + 1);
            }
            source += ".end\n";
        }
        let input = neoclr::assemble(&source).unwrap();
        assert!(
            expand(&input, "Main")
                .err()
                .unwrap()
                .to_string()
                .contains("type dependency nesting exceeds 128")
        );
    }
    #[test]
    fn host_roots_share_selected_function_budget_and_ignore_duplicates() {
        let mut source = String::from(".module HostBudget\n");
        for index in 0..=crate::limits::FUNCTIONS {
            source += &format!(".function F{index}() -> Int32\nldc.i4 42\nret\n.end\n");
        }
        let input = neoclr::assemble(&source).unwrap();
        let mut roots: Vec<_> = (0..crate::limits::FUNCTIONS).collect();
        // Repeated roots, including the entry point, do not consume more slots.
        roots.extend(0..crate::limits::FUNCTIONS);
        let (_, report) = expand_with_host_roots(&input, "F0", &roots).unwrap();
        assert_eq!(report["functionCount"], crate::limits::FUNCTIONS);
        roots.push(crate::limits::FUNCTIONS);
        assert!(
            expand_with_host_roots(&input, "F0", &roots)
                .err()
                .expect("unique host roots must obey the selected-function budget")
                .to_string()
                .contains("selected functions")
        );
        // A call discovered after admitting host roots uses the same budget.
        let mut input = input;
        input.functions[0].body = vec![
            Op::Call(FunctionRef {
                definition: None,
                name: format!("F{}", crate::limits::FUNCTIONS),
                owner: None,
                instance: false,
                generic_arguments: vec![],
                parameters: vec![],
            }),
            Op::Return,
        ];
        roots.pop();
        assert!(
            expand_with_host_roots(&input, "F0", &roots)
                .err()
                .expect("discovered calls must share the host-root budget")
                .to_string()
                .contains("selected functions")
        );
    }

    #[test]
    fn selected_function_budget_accepts_boundary_and_rejects_overflow() {
        for count in [crate::limits::FUNCTIONS, crate::limits::FUNCTIONS + 1] {
            let mut source = String::from(".module FunctionBudget\n.function Main() -> Int32\n");
            for index in 1..count {
                source += &format!("call F{index}()\npop\n");
            }
            source += "ldc.i4 42\nret\n.end\n";
            for index in 1..count {
                source += &format!(".function F{index}() -> Int32\nldc.i4 0\nret\n.end\n");
            }
            let input = neoclr::assemble(&source).unwrap();
            let specialized = expand(&input, "Main");
            let selected = super::super::selection::select(&input, "Main");
            if count == crate::limits::FUNCTIONS {
                assert_eq!(specialized.unwrap().1["functionCount"], count);
                let (module, _) = selected.unwrap();
                assert_eq!(module.functions.len(), count);
                assert!(
                    !super::super::compiler::compile(&module, "Main", false, None)
                        .unwrap()
                        .is_empty()
                );
            } else {
                assert!(
                    specialized
                        .err()
                        .unwrap()
                        .to_string()
                        .contains("method specialization exceeds")
                );
                assert!(
                    selected
                        .err()
                        .unwrap()
                        .to_string()
                        .contains("selected functions exceed")
                );
            }
        }
    }

    #[test]
    fn input_function_inventory_remains_bounded() {
        let mut input = neoclr::assemble(
            ".module InputBudget\n.function Main() -> Int32\nldc.i4 0\nret\n.end\n",
        )
        .unwrap();
        input.functions.resize(
            crate::limits::INPUT_FUNCTIONS + 1,
            input.functions[0].clone(),
        );
        assert!(
            expand(&input, "Main")
                .err()
                .unwrap()
                .to_string()
                .contains("specialization input exceeds metadata limits")
        );
        assert!(
            super::super::selection::select(&input, "Main")
                .err()
                .unwrap()
                .to_string()
                .contains("closed-world input inventory exceeds bounds")
        );
    }

    #[test]
    fn closed_type_budget_includes_nongeneric_dependency_shapes() {
        for count in [128, MAX_SPECIALIZED_TYPES, MAX_SPECIALIZED_TYPES + 1] {
            let mut source = String::from(
                ".module Shapes\n.function Main(T0 value) -> Int32\nldc.i4 0\nret\n.end\n",
            );
            for index in 0..count {
                source += &format!(".type class T{index}\n");
                for child in [index * 2 + 1, index * 2 + 2] {
                    if child < count {
                        source += &format!(".field Child{child} T{child}\n");
                    }
                }
                source += ".end\n";
            }
            let input = neoclr::assemble(&source).unwrap();
            let result = expand(&input, "Main");
            if count <= MAX_SPECIALIZED_TYPES {
                let (_, report) = result.unwrap();
                assert_eq!(report["typeCount"], count);
                assert_eq!(report["functionCount"], 1);
                assert_eq!(report["functionCloneCount"], 0);
            } else {
                // Source inventory has a separate, larger bound. This graph is
                // admitted as input but exceeds the executable shape budget.
                assert_eq!(
                    result.err().unwrap().to_string(),
                    format!("specialized type count exceeds {MAX_SPECIALIZED_TYPES}")
                );
            }
        }
    }
    #[test]
    fn generic_shape_expansion_has_its_own_bound_before_selection() {
        for count in [MAX_SPECIALIZED_TYPES / 2, MAX_SPECIALIZED_TYPES / 2 + 1] {
            let mut source =
                String::from(".module ExpansionBudget\n.type Box<T>\n.field Value T\n.end\n");
            for index in 0..count {
                source += &format!(".type Leaf{index}\n.end\n");
            }
            source += ".function Main() -> Int32\n";
            for index in 0..count {
                source += &format!(".local Box<Leaf{index}> item{index}\n");
            }
            source += "ldc.i4 0\nret\n.end\n";
            let result = expand(&neoclr::assemble(&source).unwrap(), "Main");
            if count * 2 == MAX_SPECIALIZED_TYPES {
                assert_eq!(result.unwrap().1["typeCount"], MAX_SPECIALIZED_TYPES);
            } else {
                assert!(result.err().unwrap().to_string().contains(&format!(
                    "specialized type count exceeds {MAX_SPECIALIZED_TYPES}"
                )));
            }
        }
    }
    #[test]
    fn init_accessor_authority_survives_specialization_and_selection() {
        let source = ".module InitAuthority
.entry Main
.type class Counter<T>
.field Number T
.property instance Value() -> T
.get instance Counter<T>::Read()
.set instance Counter<T>::Write(T)
.end
.property instance Unused() -> T
.set instance Counter<T>::Unused(T)
.end
.method instance .ctor(T value) -> void
ldarg 0
ldarg value
stfld Counter<T>::Number
ret
.end
.method instance Read() -> T
ldarg 0
ldfld Counter<T>::Number
ret
.end
.method instance Write(T value) -> Void
ldarg 0
ldarg value
stfld Counter<T>::Number
ldvoid
ret
.end
.method instance Unused(T value) -> Void
ldarg 0
ldarg value
stfld Counter<T>::Number
ldvoid
ret
.end
.end
.function Main() -> Int32
ldc.i4 1
newobj instance Counter<Int32>::.ctor(Int32)
dup
ldc.i4 42
call instance Counter<Int32>::Write(Int32)
pop
call instance Counter<Int32>::Read()
ldc.i8 1
newobj instance Counter<Int64>::.ctor(Int64)
dup
ldc.i8 2
call instance Counter<Int64>::Write(Int64)
pop
call instance Counter<Int64>::Read()
pop
ret
.end";
        let mut value = serde_json::to_value(neoclr::assemble(source).unwrap()).unwrap();
        value["assemblies"] = json!([{"name":"InitAuthority", "full_name":"InitAuthority", "modules":["InitAuthority.dll"], "references":[]}]);
        value["types"][0]["origin"] = json!({"assembly":"InitAuthority", "module":"InitAuthority.dll", "name":"Counter", "token":0x02000001,
            "field_tokens":[0x04000001], "field_readonly":[true], "property_tokens":[0x17000001, 0x17000002]});
        value["types"][0]["properties"][0]["init_only"] = json!(true);
        value["types"][0]["properties"][1]["init_only"] = json!(true);
        let input = serde_json::from_value(value).unwrap();
        let (selected, _) = crate::selection::prepare(&input, "Main").unwrap();
        neoclr::verify(&selected).unwrap();
        assert_eq!(
            neoclr::run(&selected, neoclr::Limits::default())
                .unwrap()
                .value,
            neoclr::Value::Int32(42)
        );
        assert_eq!(selected.types.len(), 2);
        assert!(
            selected
                .types
                .iter()
                .all(|t| t.properties.len() == 1 && t.properties[0].init_only)
        );
        assert!(!selected.functions.iter().any(|f| f.name.contains("Unused")));
        let mut unmarked = selected.clone();
        for t in &mut unmarked.types {
            t.properties[0].init_only = false;
        }
        assert!(
            neoclr::verify(&unmarked)
                .unwrap_err()
                .message
                .contains("readonly")
        );
    }
}

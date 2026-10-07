//! Bounded type-generic specialization before explicit closed-world selection.
//! One shape per value definition; bounded primitive static method instantiations.
use neoclr::metadata::{Function, FunctionRef, Instruction as Op, Type};
use serde_json::{Value, json};
use std::collections::BTreeMap;
type Error = Box<dyn std::error::Error>;
#[derive(Clone, PartialEq, Eq)]
struct Instance {
    source: usize,
    types: Vec<Type>,
    methods: Vec<Type>,
    row: usize,
}
struct Specializer<'a> {
    source: &'a neoclr::Module,
    shapes: BTreeMap<usize, Vec<Type>>,
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
        let (name, arguments) = match ty {
            Type::Int32 | Type::Byte | Type::Boolean | Type::Void | Type::Value | Type::String => {
                return Ok(ty.clone());
            }
            Type::ByRef(t) => return Ok(Type::ByRef(Box::new(self.lower(t)?))),
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
        if (definition.is_reference_type && !definition.generic_parameters.is_empty())
            || !definition.generic_constraints.is_empty()
            || definition.generic_parameters.len() != arguments.len()
        {
            return Err(format!("unsupported generic type category, arity or constraints: {name}; reference={}, arity={}/{}, constraints={:?}", definition.is_reference_type, arguments.len(), definition.generic_parameters.len(), definition.generic_constraints).into());
        }
        // Validate argument shapes before allowing a duplicate instantiation through.
        // Bound argument nesting independently of recursive local record layout checks.
        for arg in &arguments {
            validate_argument(arg, 0)?;
        }
        if let Some(existing) = self.shapes.get(&i) {
            if existing != &arguments {
                return Err(
                    "multiple closed instantiations of one type require a later profile".into(),
                );
            }
            return Ok(Type::Named(name.clone()));
        }
        if self.shapes.len() >= 32 {
            return Err("specialized type count exceeds 32".into());
        }
        self.shapes.insert(i, arguments.clone());
        for arg in &arguments {
            self.lower(arg)?;
        }
        for field in &definition.fields {
            self.close(&field.ty, &arguments)?;
        }
        for base in definition.base.iter().chain(&definition.implements) {
            self.close(base, &arguments)?;
        }
        if let Some(owner) = &definition.declaring_type {
            if owner.module != self.source.name || owner.revision != self.source.revision {
                return Err("external lexical owner is unsupported".into());
            }
            let owner = self
                .source
                .types
                .get(owner.index as usize)
                .ok_or("invalid lexical owner")?;
            // Selecting a closed generic lexical owner needs its own context contract.
            if !owner.generic_parameters.is_empty() {
                return Err("generic lexical owners require a later profile".into());
            }
            if owner.is_reference_type {
                // Keep lexical identity only. The value profile separately validates
                // empty static companions and rejects every executable use.
                let owner_index = self.type_index(&owner.name)?;
                if !self.shapes.contains_key(&owner_index) && self.shapes.len() >= 32 {
                    return Err("specialized type count exceeds 32".into());
                }
                self.shapes.insert(owner_index, vec![]);
            } else {
                self.lower(&Type::Named(owner.name.clone()))?;
            }
        }
        Ok(Type::Named(name.clone()))
    }
    fn owner(&mut self, ty: &Type, instance: bool) -> Result<Type, Error> {
        if !instance {
            if let Type::Named(name) = ty {
                let i = self.type_index(name)?;
                if super::selection::static_owner(&self.source.types[i]) {
                    if !self.shapes.contains_key(&i) && self.shapes.len() >= 32 {
                        return Err("specialized type count exceeds 32".into());
                    }
                    self.shapes.insert(i, vec![]);
                    return Ok(ty.clone());
                }
            }
        }
        self.lower(ty)
    }
    fn resolve(&mut self, target: &FunctionRef) -> Result<Instance, Error> {
        if target.generic_arguments.len() > 4 {
            return Err("generic method argument count exceeds four".into());
        }
        if target
            .generic_arguments
            .iter()
            .any(|t| !matches!(t, Type::Int32 | Type::Byte | Type::Boolean | Type::Void))
        {
            return Err(
                "generic method arguments require primitive Int32/Byte/Boolean/Void shapes".into(),
            );
        }
        let arguments = match &target.owner {
            Some(Type::Constructed { arguments, .. }) => arguments.clone(),
            _ => vec![],
        };
        let mut candidates = vec![];
        for (i, f) in self.source.functions.iter().enumerate() {
            if f.name != target.name
                || f.instance != target.instance
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
        let f = &self.source.functions[i];
        if !target.generic_arguments.is_empty() && (f.instance || !arguments.is_empty()) {
            return Err("generic methods require static functions on nongeneric owners".into());
        }
        if let Some(instance) = self.instances.iter().find(|v| {
            v.source == i && v.types == arguments && v.methods == target.generic_arguments
        }) {
            return Ok(instance.clone());
        }
        if self.instances.len() >= 128
            || (!target.generic_arguments.is_empty() && self.clones >= 32)
        {
            return Err("method specialization exceeds 128 selected functions or 32 clones".into());
        }
        let row = if target.generic_arguments.is_empty() {
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
        if f.generic_parameters.len() != methods.len()
            || !f.generic_constraints.is_empty()
            || !f.generic_arguments.is_empty()
        {
            return Err(
                "generic method arity, constraints or pre-instantiated bodies are unsupported"
                    .into(),
            );
        }
        let mut result = f.clone();
        result.generic_parameters.clear();
        result.owner = f
            .owner
            .as_ref()
            .map(|t| {
                let closed = substitute(t, arguments, methods).map_err(|e| e.to_string())?;
                self.owner(&closed, f.instance)
            })
            .transpose()?;
        for t in result
            .parameters
            .iter_mut()
            .chain(&mut result.locals)
            .chain([&mut result.returns])
        {
            *t = self.lower(&substitute(t, arguments, methods).map_err(|e| e.to_string())?)?;
        }
        for op in &mut result.body {
            match op {
                Op::Call(target) | Op::Construct(target) => {
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
                    let instance = self.resolve(&closed)?;
                    target.definition = Some(neoclr::metadata::MemberId {
                        module: self.source.name.clone(),
                        revision: self.source.revision.clone(),
                        index: instance.row as u32,
                    });
                    if !instance.methods.is_empty() {
                        target.name =
                            clone_name(&self.source.functions[instance.source], instance.row);
                    }
                    target.generic_arguments.clear();
                    pending.push(instance);
                    target.owner = closed
                        .owner
                        .as_ref()
                        .map(|t| self.owner(t, closed.instance))
                        .transpose()?;
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
                | Op::UnpackValue(t) => {
                    *t =
                        self.lower(&substitute(t, arguments, methods).map_err(|e| e.to_string())?)?
                }
                Op::CallVirtual(_) => {
                    return Err("virtual calls require a later specialization profile".into());
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
fn clone_name(f: &Function, row: usize) -> String {
    format!("{}$aot_method_{row}", f.name)
}

fn validate_argument(ty: &Type, depth: usize) -> Result<(), Error> {
    if depth > 16 {
        return Err("generic argument nesting exceeds 16".into());
    }
    match ty {
        Type::Int32 | Type::Byte | Type::Boolean | Type::String | Type::Named(_) => Ok(()),
        Type::Constructed { arguments, .. } => {
            for arg in arguments {
                validate_argument(arg, depth + 1)?;
            }
            Ok(())
        }
        _ => Err("generic arguments must be closed value types".into()),
    }
}

pub fn expand(input: &neoclr::Module, root: &str) -> Result<(neoclr::Module, Value), Error> {
    if input.types.len() > 1024 || input.functions.len() > 4096 {
        return Err("specialization input exceeds metadata limits".into());
    }
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
    let mut context = Specializer {
        source: input,
        shapes: BTreeMap::new(),
        instances: vec![root_instance.clone()],
        clones: 0,
    };
    let mut pending = vec![root_instance];
    let mut visited = std::collections::HashSet::new();
    let mut expanded = input.clone();
    while let Some(instance) = pending.pop() {
        if !visited.insert(instance.row) {
            continue;
        }
        let mut function = context.function(
            &input.functions[instance.source],
            &instance.types,
            &instance.methods,
            &mut pending,
        )?;
        if !instance.methods.is_empty() {
            function.name = clone_name(&function, instance.row);
            if let Some(origin) = &mut function.origin {
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
    let shapes = context.shapes.clone();
    for (&i, arguments) in &shapes {
        let t = &mut expanded.types[i];
        t.generic_parameters.clear();
        for field in &mut t.fields {
            field.ty = context.close(&field.ty, arguments)?;
        }
        t.base = t
            .base
            .as_ref()
            .map(|t| context.close(t, arguments))
            .transpose()?;
        t.implements = t
            .implements
            .iter()
            .map(|t| context.close(t, arguments))
            .collect::<Result<_, _>>()?;
        // Descriptive property types are removed by closed-world selection; no property
        // access can bypass selection of its accessor body.
        t.properties.clear();
        if let Some(origin) = &mut t.origin {
            origin.property_tokens.clear();
        }
    }
    let report = json!({"policy":"one shape per value definition; primitive static generic methods, at most 32 clones and 128 selected functions; no constraints",
        "methods": context.instances.iter().filter(|v| !v.methods.is_empty()).map(|v| json!({"sourceIndex":v.source,"expandedIndex":v.row,"definition":input.functions[v.source].definition.clone().unwrap_or(neoclr::metadata::MemberId { module: input.name.clone(), revision: input.revision.clone(), index: v.source as u32 }),"name":input.functions[v.source].name,"sourceOrigin":input.functions[v.source].origin,"arguments":v.methods})).collect::<Vec<_>>(),
        "types": shapes.iter().filter(|(_,args)| !args.is_empty()).map(|(i,args)| json!({"sourceIndex":i,"definition":input.types[*i].definition,"name":input.types[*i].name,"arguments":args})).collect::<Vec<_>>()});
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
            }
        }
    }
}

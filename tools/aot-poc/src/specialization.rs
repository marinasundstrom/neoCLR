//! Bounded type-generic specialization before explicit closed-world selection.
//! One closed instantiation per local type definition; no generic methods/constraints.
use neoclr::metadata::{Function, FunctionRef, Instruction as Op, Type};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
type Error = Box<dyn std::error::Error>;
struct Specializer<'a> {
    source: &'a neoclr::Module,
    shapes: BTreeMap<usize, Vec<Type>>,
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
            Type::Int32 | Type::Byte | Type::Boolean | Type::Void | Type::Value => {
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
        if definition.is_reference_type
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
    fn resolve(&self, target: &FunctionRef) -> Result<(usize, Vec<Type>), Error> {
        if !target.generic_arguments.is_empty() {
            return Err("generic methods are outside this specialization profile".into());
        }
        let arguments = match &target.owner {
            Some(Type::Constructed { arguments, .. }) => arguments.clone(),
            _ => vec![],
        };
        let mut candidates = vec![];
        for (i, f) in self.source.functions.iter().enumerate() {
            if f.name != target.name
                || f.instance != target.instance
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
                .map(|t| t.substitute_type_parameters(&arguments))
                .transpose()
                .map_err(|e| e.to_string())?;
            let parameters = f
                .parameters
                .iter()
                .map(|t| t.substitute_type_parameters(&arguments))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            if owner == target.owner && parameters == target.parameters {
                candidates.push(i);
            }
        }
        let [i] = candidates.as_slice() else {
            return Err("generic call identity/signature is missing or ambiguous".into());
        };
        Ok((*i, arguments))
    }
    fn function(
        &mut self,
        f: &Function,
        arguments: &[Type],
        pending: &mut Vec<(usize, Vec<Type>)>,
    ) -> Result<Function, Error> {
        if !f.generic_parameters.is_empty()
            || !f.generic_constraints.is_empty()
            || !f.generic_arguments.is_empty()
        {
            return Err("generic methods and constraints require a later profile".into());
        }
        let mut result = f.clone();
        result.owner = f
            .owner
            .as_ref()
            .map(|t| {
                let closed = t
                    .substitute_type_parameters(arguments)
                    .map_err(|e| e.to_string())?;
                self.owner(&closed, f.instance)
            })
            .transpose()?;
        for t in result
            .parameters
            .iter_mut()
            .chain(&mut result.locals)
            .chain([&mut result.returns])
        {
            *t = self.close(t, arguments)?;
        }
        for op in &mut result.body {
            match op {
                Op::Call(target) | Op::Construct(target) => {
                    let mut closed = target.clone();
                    closed.owner = target
                        .owner
                        .as_ref()
                        .map(|t| t.substitute_type_parameters(arguments))
                        .transpose()
                        .map_err(|e| e.to_string())?;
                    closed.parameters = target
                        .parameters
                        .iter()
                        .map(|t| t.substitute_type_parameters(arguments))
                        .collect::<Result<_, _>>()
                        .map_err(|e| e.to_string())?;
                    let instance = self.resolve(&closed)?;
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
                | Op::UnpackValue(t) => *t = self.close(t, arguments)?,
                Op::CallVirtual(_) => {
                    return Err("virtual calls require a later specialization profile".into());
                }
                _ => (), // Ordinary selected-body admission still rejects unsupported IL.
            }
        }
        Ok(result)
    }
}
fn validate_argument(ty: &Type, depth: usize) -> Result<(), Error> {
    if depth > 16 {
        return Err("generic argument nesting exceeds 16".into());
    }
    match ty {
        Type::Int32 | Type::Byte | Type::Boolean | Type::Named(_) => Ok(()),
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
    let mut context = Specializer {
        source: input,
        shapes: BTreeMap::new(),
    };
    let mut pending = vec![(*root, vec![])];
    let mut visited = BTreeSet::new();
    let mut expanded = input.clone();
    while let Some((i, arguments)) = pending.pop() {
        if !visited.insert(i) {
            continue;
        }
        if visited.len() > 128 {
            return Err("specialized function count exceeds 128".into());
        }
        expanded.functions[i] = context.function(&input.functions[i], &arguments, &mut pending)?;
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
    let report = json!({"policy":"one closed instantiation per local type definition; no generic methods or constraints",
        "types": shapes.iter().filter(|(_,args)| !args.is_empty()).map(|(i,args)| json!({"sourceIndex":i,"definition":input.types[*i].definition,"name":input.types[*i].name,"arguments":args})).collect::<Vec<_>>()});
    Ok((expanded, report))
}

//! Bounded inline value records. All managed addresses borrow active frame storage.
use super::{Error, flow};
use neoclr::metadata::{FunctionRef, Instruction as Op, Representation, Type};
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Ty {
    Int,
    Byte,
    SByte,
    Short,
    UShort,
    Bool,
    Unit,
    Erased,
    Character, // One validated extended grapheme, stored as immutable UTF-8 text.
    Literal, // Immutable image/explicit invocation-arena UTF-8; not a general managed String.
    Record(usize),
    Reference(usize),
    Interface(usize),
    ByteArray,
    Size,
    Wide,
    Address(Box<Ty>),
}
impl Ty {
    pub fn address(self) -> Self {
        Self::Address(Box::new(self))
    }
}
pub(super) struct Profile<'a> {
    pub input: &'a neoclr::Module,
    references: bool,
    object_base: Option<usize>,
    layouts: Vec<Vec<usize>>,
    widths: Vec<usize>,
    names: HashMap<&'a str, Vec<usize>>,
    pub args: Vec<Vec<Ty>>,
    pub locals: Vec<Vec<Ty>>,
    pub results: Vec<Option<Ty>>,
    pub dispatch: HashMap<usize, Vec<(usize, usize)>>,
}
pub(super) type Stacks = Vec<Option<Vec<Ty>>>;

// Private compilation tags, not runtime Type ordinals or a public/native ABI.
pub(super) fn erased_tag(ty: &Type) -> Result<i64, Error> {
    match ty {
        Type::Void => Ok(0),
        Type::Int32 => Ok(1),
        Type::Byte => Ok(2),
        Type::Boolean => Ok(3),
        _ => Err("erased payload requires Int32, Byte, Boolean or Void".into()),
    }
}

fn object_base_shape(t: &neoclr::metadata::TypeDef) -> bool {
    t.is_reference_type && t.is_abstract
        && t.representation == Representation::Record && t.fields.is_empty()
        && t.base.is_none() && t.generic_parameters.is_empty() && t.implements.is_empty()
}

impl<'a> Profile<'a> {
    pub fn new(input: &'a neoclr::Module, references: bool, object_base: Option<usize>) -> Result<Self, Error> {
        if input.name == "System" || input.types.len() > 64 || input.functions.len() > 128 {
            return Err(
                "value profile requires an application with at most 64 types and 128 functions"
                    .into(),
            );
        }
        if object_base.is_some_and(|i| !references || input.types.get(i).is_none_or(|t| !object_base_shape(t))) {
            return Err("invalid private Object base projection".into());
        }
        for (index, t) in input.types.iter().enumerate() {
            if references && t.representation == Representation::Interface {
                if !t.fields.is_empty() || t.base.is_some() || !t.implements.is_empty()
                    || !t.generic_parameters.is_empty() || !t.generic_constraints.is_empty()
                    || t.enum_info.is_some() || t.packing.is_some() || t.minimum_size.is_some() {
                    return Err("interface views require fieldless nongeneric contracts without inheritance".into());
                }
                continue;
            }
            if !t.implements.is_empty() && (!references || !t.is_reference_type || t.implements.iter().any(|interface| {
                !matches!(interface, Type::Named(_)) || input.type_definition(interface)
                    .is_none_or(|t| t.representation != Representation::Interface || !t.generic_parameters.is_empty())
            })) {
                return Err(format!("{}: only reference-arena nongeneric interface views are supported", t.name).into());
            }
            let static_owner = crate::selection::static_owner(t);
            let root = references && object_base == Some(index);
            let root_base = references && t.is_reference_type && t.base.as_ref()
                .is_some_and(|base| object_base.is_some_and(|i| *base == Type::Named(input.types[i].name.clone())));
            if (t.is_reference_type && !static_owner && !references)
                || t.representation != Representation::Record
                || t.enum_info.is_some()
                || (t.base.is_some() && !root_base)
                || (t.is_abstract && !static_owner && !root)
                || !t.generic_parameters.is_empty()
                || !t.generic_constraints.is_empty()
                || t.packing.is_some()
                || t.minimum_size.is_some()
                || t.fields.len() > 16
                || t.fields.iter().any(|f| {
                    f.deferred
                        || !(matches!(f.ty, Type::Int32 | Type::Byte | Type::SByte | Type::Int16 | Type::UInt16 | Type::UInt32 | Type::Int64 | Type::UInt64 | Type::IntPtr | Type::UIntPtr | Type::Boolean | Type::String | Type::Named(_))
                            || (references && matches!(&f.ty, Type::ArrayRef(t) if **t == Type::Byte)))
                })
            {
                return Err(format!("{}: value profile requires nongeneric records with at most sixteen Int32/small-integer/Boolean/String/local-record fields", t.name).into());
            }
        }
        let mut p = Self {
            input,
            references,
            object_base,
            layouts: vec![vec![]; input.types.len()],
            widths: vec![0; input.types.len()],
            names: HashMap::new(),
            args: vec![],
            locals: vec![],
            results: vec![],
            dispatch: HashMap::new(),
        };
        // Compute bounded inline layouts before using any storage or call signatures.
        // A visiting node is an illegal inline cycle, including otherwise unused types.
        fn layout(p: &mut Profile<'_>, i: usize, states: &mut [u8]) -> Result<(), Error> {
            if states[i] == 1 {
                return Err("recursive inline value layout is unsupported".into());
            }
            if states[i] == 2 {
                return Ok(());
            }
            states[i] = 1;
            let fields = p.input.types[i].fields.clone();
            let mut width = 0;
            for field in fields {
                let ty = p.stored(&field.ty)?;
                if let Ty::Record(child) = ty {
                    layout(p, child, states)?;
                }
                p.layouts[i].push(width);
                width += p.lanes(&ty);
                if width > 16 {
                    return Err("value layout exceeds sixteen flattened lanes".into());
                }
            }
            p.widths[i] = width.max(1);
            states[i] = 2;
            Ok(())
        }
        let mut states = vec![0; input.types.len()];
        for i in 0..input.types.len() {
            layout(&mut p, i, &mut states)?;
        }
        for (i, f) in input.functions.iter().enumerate() {
            p.names.entry(&f.name).or_default().push(i);
            let interface = references && crate::selection::interface_contract(input, f);
            if interface {
                let reached = (0..input.functions.len()).collect();
                p.dispatch.insert(i, crate::selection::dispatch_targets(input, i, &reached)?);
            }
            if ((f.is_virtual || f.is_abstract) && !interface)
                || f.is_override
                || f.impl_flags != 0
                || f.pinvoke.is_some()
                || !f.generic_parameters.is_empty()
                || !f.generic_arguments.is_empty()
                || !f.generic_constraints.is_empty()
                || !f.out_when_true.is_empty()
                || !f.readonly_parameters.is_empty()
                || f.receiver_readonly
                || !f.interface_implementations.is_empty()
                || f.body.len() > 8192
                || f.locals.len() > 1024
                || f.parameters.len() > 32
            {
                return Err(format!("{}: unsupported value member contract", f.name).into());
            }
            if f.out_parameters
                .iter()
                .any(|n| !matches!(f.parameters.get(*n), Some(Type::ByRef(_))))
                || f.out_parameters.windows(2).any(|pair| pair[0] >= pair[1])
            {
                return Err("output indices must be sorted, unique borrowed parameters".into());
            }
            let mut args = vec![];
            if let Some(owner) = &f.owner {
                let metadata_only = matches!(owner, Type::Named(name) if input.types.iter()
                    .any(|t| t.name == *name && crate::selection::static_owner(t)));
                if metadata_only && !f.instance && !f.receiver_byref {
                    // Static owner contributes identity/access only, with no receiver.
                } else {
                    let ty = p.ty(owner)?;
                    if !matches!(ty, Ty::Record(_) | Ty::Reference(_) | Ty::Interface(_)) {
                        return Err("value member requires a local record owner".into());
                    }
                    if f.instance {
                        match ty {
                            Ty::Reference(_) | Ty::Interface(_) if !f.receiver_byref => args.push(ty),
                            Ty::Record(_) if f.receiver_byref => args.push(ty.address()),
                            _ => return Err("receiver representation does not match value/reference owner".into()),
                        }
                    } else if f.receiver_byref {
                        return Err("static member cannot have a by-reference receiver".into());
                    }
                }
            } else if f.instance || f.receiver_byref {
                return Err("instance member requires a record owner".into());
            }
            for (index, t) in f.parameters.iter().enumerate() {
                if let Type::ByRef(target) = t {
                    if !f.out_parameters.contains(&index) {
                        return Err(
                            "explicit borrowed parameters require an output contract".into()
                        );
                    }
                    args.push(p.stored(target)?.address());
                } else {
                    args.push(p.stored(t)?);
                }
            }
            p.args.push(args);
            p.locals.push(
                f.locals
                    .iter()
                    .map(|t| p.stored(t))
                    .collect::<Result<_, _>>()?,
            );
            p.results.push(if f.no_result {
                if f.returns != Type::Void {
                    return Err("no-result member must return Void".into());
                }
                None
            } else {
                Some(p.ty(&f.returns)?)
            });
        }
        // Resolve all calls (even dead ones) and reject recursive native call graphs.
        let mut edges = vec![vec![]; input.functions.len()];
        for (i, f) in input.functions.iter().enumerate() {
            for op in &f.body {
                if let Op::Call(target) | Op::CallVirtual(target) | Op::Construct(target) = op {
                    let callee = p.callee(target)?;
                    if p.dispatch.contains_key(&callee) && !matches!(op, Op::CallVirtual(_)) {
                        return Err("interface contracts require callvirt".into());
                    }
                    if matches!(op, Op::CallVirtual(_)) &&
                        (!references || !input.functions[callee].instance ||
                         !matches!(p.args[callee].first(), Some(Ty::Reference(_) | Ty::Interface(_)))) {
                        return Err("callvirt requires an admitted nonvirtual reference member".into());
                    }
                    if matches!(op, Op::Construct(_)) {
                        let c = &input.functions[callee];
                        if !c.instance || !c.name.ends_with("..ctor") || !c.no_result {
                            return Err("newobj.ctor requires a no-result value constructor".into());
                        }
                    }
                    edges[i].push(callee);
                }
            }
        }
        for (&contract, targets) in &p.dispatch {
            edges[contract].extend(targets.iter().map(|(_, target)| *target));
        }
        fn visit(i: usize, edges: &[Vec<usize>], states: &mut [u8]) -> Result<(), Error> {
            if states[i] == 1 {
                return Err("recursive calls require a native stack-budget contract".into());
            }
            if states[i] == 2 {
                return Ok(());
            }
            states[i] = 1;
            for &next in &edges[i] {
                visit(next, edges, states)?;
            }
            states[i] = 2;
            Ok(())
        }
        let mut states = vec![0; edges.len()];
        for i in 0..edges.len() {
            visit(i, &edges, &mut states)?;
        }
        Ok(p)
    }
    pub fn ty(&self, t: &Type) -> Result<Ty, Error> {
        Ok(match t {
            Type::Int32 | Type::UInt32 => Ty::Int,
            Type::Byte => Ty::Byte,
            Type::SByte => Ty::SByte,
            Type::Int16 => Ty::Short,
            Type::UInt16 => Ty::UShort,
            Type::Boolean => Ty::Bool,
            Type::Void => Ty::Unit,
            Type::Value => Ty::Erased,
            Type::String => Ty::Literal,
            Type::Char => Ty::Character,
            Type::IntPtr | Type::UIntPtr => Ty::Size,
            Type::Int64 | Type::UInt64 => Ty::Wide,
            Type::ArrayRef(t) if **t == Type::Byte && self.references => Ty::ByteArray,
            Type::Named(name) => {
                let i = self
                    .input
                    .types
                    .iter()
                    .position(|t| &t.name == name)
                    .ok_or("value profile requires a local named record")?;
                if crate::selection::static_owner(&self.input.types[i]) {
                    return Err("static owners cannot be used as values or instance receivers".into());
                }
                if self.input.types[i].representation == Representation::Interface && self.references { Ty::Interface(i) }
                else if self.input.types[i].is_reference_type { Ty::Reference(i) } else { Ty::Record(i) }
            }
            _ => {
                return Err(
                    "unsupported value type; references and generics require later profiles".into(),
                );
            }
        })
    }
    fn stored(&self, t: &Type) -> Result<Ty, Error> {
        let t = self.ty(t)?;
        if t == Ty::Unit {
            return Err("Void storage is outside the value profile".into());
        }
        Ok(t)
    }
    pub fn stack_type(t: &Ty) -> Ty {
        if matches!(t, Ty::Byte | Ty::SByte | Ty::Short | Ty::UShort) { Ty::Int } else { t.clone() }
    }
    pub fn narrow_lanes(&self, t: &Ty) -> Vec<Option<(u8, bool)>> {
        match t {
            Ty::Byte => vec![Some((8, false))],
            Ty::SByte => vec![Some((8, true))],
            Ty::Short => vec![Some((16, true))],
            Ty::UShort => vec![Some((16, false))],
            Ty::Record(i) if !self.input.types[*i].fields.is_empty() => self.input.types[*i]
                .fields
                .iter()
                .flat_map(|f| self.narrow_lanes(&self.ty(&f.ty).expect("admitted field")))
                .collect(),
            _ => vec![None; self.lanes(t)],
        }
    }
    pub fn pointer_lanes(&self, t: &Ty) -> Vec<bool> {
        match t {
            Ty::Literal | Ty::Character | Ty::Address(_) | Ty::Reference(_) | Ty::Interface(_) | Ty::ByteArray | Ty::Size | Ty::Wide => vec![true],
            Ty::Record(i) if !self.input.types[*i].fields.is_empty() => self.input.types[*i]
                .fields.iter().flat_map(|f| self.pointer_lanes(&self.ty(&f.ty).expect("admitted field"))).collect(),
            _ => vec![false; self.lanes(t)],
        }
    }
    pub fn lanes(&self, t: &Ty) -> usize {
        match t {
            Ty::Record(i) => self.widths[*i],
            Ty::Erased => 2,
            _ => 1,
        }
    }
    pub fn bytes(&self, t: &Ty) -> u32 {
        // Private slots use eight bytes per scalar lane so mixed pointer/Int32
        // records remain aligned. This is not an external aggregate ABI.
        self.lanes(t) as u32 * 8
    }
    fn reference_assignable(&self, actual: &Ty, expected: &Ty) -> bool {
        match (actual, expected) {
            (Ty::Reference(actual), Ty::Reference(expected)) if self.object_base == Some(*expected) =>
                self.input.types[*actual].base.as_ref().is_some_and(|t| *t == Type::Named(self.input.types[*expected].name.clone())),
            _ => false,
        }
    }
    pub fn cast_targets(&self, target: &Ty) -> Vec<usize> {
        match target {
            Ty::Reference(i) if self.object_base == Some(*i) => self.input.types.iter().enumerate()
                .filter(|(_, t)| t.is_reference_type && t.representation == Representation::Record && !t.is_abstract)
                .map(|(i, _)| i).collect(),
            Ty::Reference(i) => vec![*i],
            Ty::Interface(i) => {
                let target = Type::Named(self.input.types[*i].name.clone());
                self.input.types.iter().enumerate()
                    .filter(|(_, t)| t.is_reference_type && t.representation == Representation::Record && t.implements.contains(&target))
                    .map(|(i, _)| i).collect()
            }
            _ => unreachable!("admitted reference target"),
        }
    }
    pub fn object_bytes(&self, index: usize) -> u32 { 8 + self.widths[index] as u32 * 8 }
    pub fn field_offset(&self, t: &Ty, index: usize) -> usize {
        let t = if let Ty::Address(t) = t {
            t.as_ref()
        } else {
            t
        };
        let (Ty::Record(owner) | Ty::Reference(owner)) = t else {
            unreachable!("checked field owner")
        };
        self.layouts[*owner][index]
    }
    pub fn field(&self, t: &Ty, index: usize) -> Result<Ty, Error> {
        if matches!(t, Ty::Address(inner) if matches!(**inner, Ty::Reference(_))) {
            return Err("load a borrowed reference slot before accessing object fields".into());
        }
        let t = if let Ty::Address(t) = t {
            t.as_ref()
        } else {
            t
        };
        let (Ty::Record(owner) | Ty::Reference(owner)) = t else {
            return Err("field access requires a value record".into());
        };
        self.ty(&self.input.types[*owner]
            .fields
            .get(index)
            .ok_or("field index out of range")?
            .ty)
    }
    pub fn callee(&self, target: &FunctionRef) -> Result<usize, Error> {
        if !target.generic_arguments.is_empty() {
            return Err("generic value calls remain unsupported".into());
        }
        let candidates = self
            .names
            .get(target.name.as_str())
            .ok_or("value profile does not support external calls")?;
        let mut matches = candidates.iter().copied().filter(|&i| {
            let f = &self.input.functions[i];
            f.owner == target.owner
                && f.instance == target.instance
                && f.parameters == target.parameters
                && target
                    .definition
                    .as_ref()
                    .is_none_or(|id| Some(id) == f.definition.as_ref())
        });
        let i = matches
            .next()
            .ok_or("value member call identity/signature mismatch")?;
        if matches.next().is_some() {
            return Err("ambiguous value call requires a matching definition identity".into());
        }
        Ok(i)
    }
    pub fn root(&self, root: &str) -> Result<usize, Error> {
        let candidates = self.names.get(root).ok_or("root function not found")?;
        let [i] = candidates.as_slice() else {
            return Err("ambiguous value root name; use a uniquely named wrapper".into());
        };
        let i = *i;
        let f = &self.input.functions[i];
        if f.instance
            || self.results[i] != Some(Ty::Int)
            || !(self.args[i].is_empty() || self.args[i] == [Ty::Int])
        {
            return Err("value root must be static () -> Int32 or (Int32) -> Int32".into());
        }
        Ok(i)
    }
    // The interpreter checks callee output assignment dynamically. Native code has
    // no such runtime, so require a whole-slot write on every normal return path.
    fn verify_outputs(&self, i: usize, shapes: &Stacks) -> Result<(), Error> {
        let f = &self.input.functions[i];
        if f.out_parameters.is_empty() {
            return Ok(());
        }
        #[derive(Clone, PartialEq, Eq)]
        struct State {
            stack: Vec<Option<usize>>,
            assigned: Vec<bool>,
        }
        let mut states = vec![None; f.body.len()];
        states[0] = Some(State {
            stack: vec![],
            assigned: vec![false; f.parameters.len()],
        });
        let mut work = VecDeque::from([0]);
        let offset = usize::from(f.instance);
        while let Some(pc) = work.pop_front() {
            let mut state = states[pc].clone().unwrap();
            let op = &f.body[pc];
            let fail = || -> Error {
                format!(
                    "{} instruction {pc}: output requires a definite whole-slot assignment",
                    f.name
                )
                .into()
            };
            let readable = |origin: Option<usize>, assigned: &[bool]| -> Result<(), Error> {
                if origin.is_some_and(|n| !assigned[n]) {
                    Err(fail())
                } else {
                    Ok(())
                }
            };
            match op {
                Op::Arg(n) => state.stack.push(
                    n.checked_sub(offset)
                        .filter(|n| f.out_parameters.contains(n)),
                ),
                Op::ConvertInt32 | Op::ConvertUInt32 | Op::ConvertUInt8 | Op::ConvertInt8 | Op::ConvertInt16 | Op::ConvertUInt16 | Op::ConvertInt64 | Op::ConvertUInt64 | Op::ConvertNativeInt | Op::ConvertNativeUInt => (),
                Op::PackValue(_) | Op::IsValue(_) | Op::UnpackValue(_)
                | Op::IsInstance(_) | Op::CastClass(_) | Op::ReferenceIsNull | Op::NewArray(_) | Op::ArrayLength => {
                    readable(state.stack.pop().unwrap(), &state.assigned)?;
                    state.stack.push(None);
                }
                Op::Dup => state.stack.push(*state.stack.last().unwrap()),
                Op::StoreObject(_) => {
                    state.stack.pop();
                    if let Some(n) = state.stack.pop().unwrap() {
                        state.assigned[n] = true;
                    }
                }
                Op::InitializeObject(_) => {
                    if let Some(n) = state.stack.pop().unwrap() {
                        state.assigned[n] = true;
                    }
                }
                Op::Field(_) | Op::FieldAddress(_) | Op::LoadObject(_) => {
                    readable(state.stack.pop().unwrap(), &state.assigned)?;
                    state.stack.push(None);
                }
                Op::SetField(_) => {
                    state.stack.pop();
                    // Partial construction is deliberately outside this proof.
                    readable(state.stack.pop().unwrap(), &state.assigned)?;
                    let shape = shapes[pc].as_ref().unwrap();
                    if !matches!(shape[shape.len() - 2], Ty::Reference(_)) { state.stack.push(None); }
                }
                Op::ArrayElement(_) | Op::ArrayAddress(_) | Op::StoreArrayElement(_) => {
                    if matches!(op, Op::StoreArrayElement(_)) { state.stack.pop(); }
                    state.stack.pop();
                    readable(state.stack.pop().unwrap(), &state.assigned)?;
                    if !matches!(op, Op::StoreArrayElement(_)) { state.stack.push(None); }
                }
                Op::Call(target) | Op::CallVirtual(target) | Op::Construct(target) => {
                    let c = self.callee(target)?;
                    let callee = &self.input.functions[c];
                    let construct = matches!(op, Op::Construct(_));
                    let count = self.args[c].len() - usize::from(construct);
                    let passed = state.stack.split_off(state.stack.len() - count);
                    let receiver = usize::from(callee.instance && !construct);
                    // Check all input borrows before marking any output assigned.
                    for (n, origin) in passed.iter().enumerate() {
                        if n.checked_sub(receiver)
                            .is_none_or(|n| !callee.out_parameters.contains(&n))
                        {
                            readable(*origin, &state.assigned)?;
                        }
                    }
                    for &n in &callee.out_parameters {
                        if let Some(origin) = passed[n + receiver] {
                            state.assigned[origin] = true;
                        }
                    }
                    if construct || self.results[c].is_some() {
                        state.stack.push(None);
                    }
                }
                Op::Fault(_) => continue,
                Op::Return => {
                    if f.out_parameters.iter().any(|n| !state.assigned[*n]) {
                        return Err(fail());
                    }
                    continue;
                }
                Op::New(t) => {
                    let Ty::Record(owner) = self.ty(t)? else {
                        unreachable!()
                    };
                    state
                        .stack
                        .truncate(state.stack.len() - self.input.types[owner].fields.len());
                    state.stack.push(None);
                }
                Op::Pop
                | Op::Store(_)
                | Op::StoreArg(_)
                | Op::BranchTrue(_)
                | Op::BranchFalse(_) => {
                    state.stack.pop();
                }
                Op::Branch(_) => (),
                Op::Int(_)
                | Op::Int64(_)
                | Op::String(_)
                | Op::Bool(_)
                | Op::Void
                | Op::Load(_)
                | Op::LocalAddress(_)
                | Op::ArgumentAddress(_) => state.stack.push(None),
                _ => {
                    // All remaining admitted instructions are binary scalar operations.
                    state.stack.pop();
                    state.stack.pop();
                    if flow::comparison(op).is_none() {
                        state.stack.push(None);
                    }
                }
            }
            let successors = match op {
                Op::Branch(n) => vec![*n],
                Op::BranchTrue(n) | Op::BranchFalse(n) => vec![*n, pc + 1],
                _ if flow::comparison(op).is_some() => {
                    vec![flow::comparison(op).unwrap().0, pc + 1]
                }
                _ => vec![pc + 1],
            };
            for next in successors {
                debug_assert_eq!(state.stack.len(), shapes[next].as_ref().unwrap().len());
                if let Some(old) = &mut states[next] {
                    // Do not erase the identity of an output borrow at a CFG join.
                    if old.stack != state.stack {
                        return Err(fail());
                    }
                    let joined: Vec<_> = old
                        .assigned
                        .iter()
                        .zip(&state.assigned)
                        .map(|(a, b)| *a && *b)
                        .collect();
                    if joined != old.assigned {
                        old.assigned = joined;
                        work.push_back(next);
                    }
                } else {
                    states[next] = Some(state.clone());
                    work.push_back(next);
                }
            }
        }
        Ok(())
    }
    pub fn analyze(&self, i: usize) -> Result<Stacks, Error> {
        let f = &self.input.functions[i];
        let fail =
            |pc, message| -> Error { format!("{} instruction {pc}: {message}", f.name).into() };
        if self.dispatch.contains_key(&i) { return Ok(vec![]); }
        if f.body.is_empty() {
            return Err(fail(0, "empty body"));
        }
        // Admission is independent of reachability. Never silently trim unsupported IL.
        for (pc, op) in f.body.iter().enumerate() {
            match op {
                Op::Arg(n) | Op::ArgumentAddress(n) | Op::StoreArg(n)
                    if *n < self.args[i].len() =>
                {
                    if matches!(op, Op::ArgumentAddress(_) | Op::StoreArg(_))
                        && matches!(self.args[i][*n], Ty::Address(_))
                    {
                        return Err(fail(pc, "cannot address or replace a borrowed parameter"));
                    }
                }
                Op::Load(n) | Op::Store(n) | Op::LocalAddress(n) if *n < self.locals[i].len() => (),
                Op::Int(_)
                | Op::Int64(_)
                | Op::String(_)
                | Op::ConvertInt32
                | Op::ConvertUInt32
                | Op::ConvertInt64
                | Op::ConvertUInt64
                | Op::ConvertNativeInt
                | Op::ConvertNativeUInt
                | Op::ConvertUInt8
                | Op::ConvertInt8
                | Op::ConvertInt16
                | Op::ConvertUInt16
                | Op::Bool(_)
                | Op::Void
                | Op::Dup
                | Op::Pop
                | Op::Field(_)
                | Op::SetField(_)
                | Op::FieldAddress(_)
                | Op::Call(_)
                | Op::CallVirtual(_)
                | Op::Construct(_)
                | Op::Return
                | Op::Fault(_)
                | Op::Branch(_)
                | Op::BranchTrue(_)
                | Op::BranchFalse(_)
                | Op::Equal
                | Op::Greater
                | Op::Less
                | Op::GreaterUnsigned
                | Op::LessUnsigned
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
                | Op::Remainder
                | Op::RemainderUnsigned => (),
                Op::IsInstance(Type::String) | Op::CastClass(Type::String) | Op::ReferenceIsNull | Op::ReferenceEqual => (),
                Op::IsInstance(t) | Op::CastClass(t) if self.references && matches!(self.ty(t)?, Ty::Reference(_) | Ty::Interface(_)) => (),
                Op::NewArray(Type::Byte) | Op::ArrayElement(Type::Byte) | Op::StoreArrayElement(Type::Byte) | Op::ArrayAddress(Type::Byte) | Op::ArrayLength if self.references => (),
                Op::PackValue(t) | Op::IsValue(t) | Op::UnpackValue(t) => {
                    erased_tag(t)?;
                }
                Op::InitializeObject(Type::Value) => {
                    return Err(fail(pc, "Value has no default initialization"));
                }
                Op::InitializeObject(t) | Op::LoadObject(t) | Op::StoreObject(t) | Op::New(t) => {
                    self.stored(t)?;
                }
                _ if flow::comparison(op).is_some() => (),
                _ => return Err(fail(pc, "unsupported value instruction")),
            }
        }
        let mut stacks: Stacks = vec![None; f.body.len()];
        stacks[0] = Some(vec![]);
        let mut work = VecDeque::from([0]);
        while let Some(pc) = work.pop_front() {
            let mut stack = stacks[pc].as_ref().unwrap().clone();
            let op = &f.body[pc];
            let pop = |s: &mut Vec<Ty>| s.pop().ok_or_else(|| fail(pc, "stack underflow"));
            let take = |s: &mut Vec<Ty>, expected: &Ty| -> Result<(), Error> {
                let actual = pop(s)?;
                let expected = Self::stack_type(expected);
                if actual != expected && !self.reference_assignable(&actual, &expected) {
                    return Err(fail(pc, "value operand type mismatch"));
                }
                Ok(())
            };
            match op {
                Op::Int(_) => stack.push(Ty::Int),
                Op::Int64(_) => stack.push(Ty::Wide),
                Op::String(_) => stack.push(Ty::Literal),
                Op::ConvertInt32 | Op::ConvertUInt32 | Op::ConvertUInt8 | Op::ConvertInt8 | Op::ConvertInt16 | Op::ConvertUInt16 | Op::ConvertInt64 | Op::ConvertUInt64 | Op::ConvertNativeInt | Op::ConvertNativeUInt => {
                    if !matches!(pop(&mut stack)?, Ty::Int | Ty::Size | Ty::Wide) { return Err(fail(pc, "conversion requires an integer stack category")); }
                    stack.push(match op {
                        Op::ConvertInt64 | Op::ConvertUInt64 => Ty::Wide,
                        Op::ConvertNativeInt | Op::ConvertNativeUInt => Ty::Size,
                        _ => Ty::Int,
                    });
                }
                Op::Bool(_) => stack.push(Ty::Bool),
                Op::Void => stack.push(Ty::Unit),
                Op::PackValue(t) => {
                    take(&mut stack, &self.ty(t)?)?;
                    stack.push(Ty::Erased);
                }
                Op::IsValue(_) => {
                    take(&mut stack, &Ty::Erased)?;
                    stack.push(Ty::Bool);
                }
                Op::UnpackValue(t) => {
                    take(&mut stack, &Ty::Erased)?;
                    stack.push(Self::stack_type(&self.ty(t)?));
                }
                Op::Arg(n) => stack.push(Self::stack_type(&self.args[i][*n])),
                Op::Load(n) => stack.push(Self::stack_type(&self.locals[i][*n])),
                Op::ArgumentAddress(n) => stack.push(self.args[i][*n].clone().address()),
                Op::LocalAddress(n) => stack.push(self.locals[i][*n].clone().address()),
                Op::StoreArg(n) => take(&mut stack, &self.args[i][*n])?,
                Op::Store(n) => take(&mut stack, &self.locals[i][*n])?,
                Op::Dup => {
                    let t = pop(&mut stack)?;
                    stack.extend([t.clone(), t]);
                }
                Op::Pop => {
                    pop(&mut stack)?;
                }
                Op::InitializeObject(t) => take(&mut stack, &self.ty(t)?.address())?,
                Op::LoadObject(t) => {
                    let t = self.ty(t)?;
                    take(&mut stack, &t.clone().address())?;
                    stack.push(Self::stack_type(&t));
                }
                Op::StoreObject(t) => {
                    let t = self.ty(t)?;
                    take(&mut stack, &t)?;
                    take(&mut stack, &t.address())?;
                }
                Op::New(t) => {
                    let t = self.ty(t)?;
                    let Ty::Record(owner) = t else {
                        return Err(fail(pc, "reference allocation requires a constructor"));
                    };
                    for f in self.input.types[owner].fields.iter().rev() {
                        take(&mut stack, &self.ty(&f.ty)?)?;
                    }
                    stack.push(t);
                }
                Op::IsInstance(Type::String) | Op::CastClass(Type::String) => {
                    take(&mut stack, &Ty::Literal)?;
                    stack.push(Ty::Literal);
                }
                Op::IsInstance(t) | Op::CastClass(t) => {
                    if !matches!(pop(&mut stack)?, Ty::Reference(_) | Ty::Interface(_)) {
                        return Err(fail(pc, "reference casts require a class or interface view"));
                    }
                    stack.push(self.ty(t)?);
                }
                Op::ReferenceIsNull => {
                    if !matches!(pop(&mut stack)?, Ty::Literal | Ty::Reference(_) | Ty::Interface(_) | Ty::ByteArray) {
                        return Err(fail(pc, "null test requires text or reference"));
                    }
                    stack.push(Ty::Bool);
                }
                Op::ReferenceEqual => {
                    let ty = pop(&mut stack)?;
                    if !matches!(ty, Ty::Reference(_) | Ty::Interface(_) | Ty::ByteArray) { return Err(fail(pc, "identity requires same reference type")); }
                    take(&mut stack, &ty)?;
                    stack.push(Ty::Bool);
                }
                Op::NewArray(Type::Byte) => {
                    take(&mut stack, &Ty::Int)?;
                    stack.push(Ty::ByteArray);
                }
                Op::ArrayLength => {
                    take(&mut stack, &Ty::ByteArray)?;
                    stack.push(Ty::Size);
                }
                Op::ArrayElement(Type::Byte) | Op::ArrayAddress(Type::Byte) => {
                    take(&mut stack, &Ty::Int)?;
                    take(&mut stack, &Ty::ByteArray)?;
                    stack.push(if matches!(op, Op::ArrayAddress(_)) { Ty::Byte.address() } else { Ty::Int });
                }
                Op::StoreArrayElement(Type::Byte) => {
                    take(&mut stack, &Ty::Int)?;
                    take(&mut stack, &Ty::Int)?;
                    take(&mut stack, &Ty::ByteArray)?;
                }
                Op::Field(n) | Op::FieldAddress(n) => {
                    let owner = pop(&mut stack)?;
                    let t = self.field(&owner, *n)?;
                    if matches!(op, Op::FieldAddress(_)) {
                        if !matches!(owner, Ty::Address(_) | Ty::Reference(_)) {
                            return Err(fail(pc, "field address requires a borrowed record"));
                        }
                        stack.push(t.address());
                    } else {
                        stack.push(Self::stack_type(&t));
                    }
                }
                Op::SetField(n) => {
                    let value = pop(&mut stack)?;
                    let owner = pop(&mut stack)?;
                    if Self::stack_type(&self.field(&owner, *n)?) != value {
                        return Err(fail(pc, "field store type mismatch"));
                    }
                    if !matches!(owner, Ty::Reference(_)) {
                        stack.push(if matches!(owner, Ty::Address(_)) { Ty::Unit } else { owner });
                    }
                }
                Op::Call(target) | Op::CallVirtual(target) | Op::Construct(target) => {
                    let c = self.callee(target)?;
                    let construct = matches!(op, Op::Construct(_));
                    for t in self.args[c].iter().skip(usize::from(construct)).rev() {
                        take(&mut stack, t)?;
                    }
                    if construct {
                        stack.push(self.ty(target.owner.as_ref().unwrap())?);
                    } else if let Some(t) = &self.results[c] {
                        stack.push(Self::stack_type(t));
                    }
                }
                Op::Return => {
                    if let Some(t) = &self.results[i] {
                        take(&mut stack, t)?;
                    }
                    if !stack.is_empty() {
                        return Err(fail(pc, "nonempty return stack"));
                    }
                }
                Op::Fault(_) | Op::Branch(_) => (),
                Op::BranchTrue(_) | Op::BranchFalse(_) => {
                    if !matches!(pop(&mut stack)?, Ty::Int | Ty::Bool) {
                        return Err(fail(pc, "branch requires Int32 or Boolean"));
                    }
                }
                Op::Add | Op::Sub | Op::Mul => {
                    let t = pop(&mut stack)?;
                    if !matches!(t, Ty::Int | Ty::Wide) { return Err(fail(pc, "wrapping arithmetic requires Int32 or Int64")); }
                    take(&mut stack, &t)?;
                    stack.push(t);
                }
                Op::Equal => {
                    let t = pop(&mut stack)?;
                    if !matches!(t, Ty::Int | Ty::Bool) {
                        return Err(fail(pc, "equality requires Int32 or Boolean"));
                    }
                    take(&mut stack, &t)?;
                    stack.push(Ty::Bool);
                }
                _ => {
                    take(&mut stack, &Ty::Int)?;
                    take(&mut stack, &Ty::Int)?;
                    if flow::comparison(op).is_none() {
                        stack.push(
                            if matches!(
                                op,
                                Op::Greater | Op::Less | Op::GreaterUnsigned | Op::LessUnsigned
                            ) {
                                Ty::Bool
                            } else {
                                Ty::Int
                            },
                        );
                    }
                }
            }
            let successors = match op {
                Op::Return | Op::Fault(_) => vec![],
                Op::Branch(n) => vec![*n],
                Op::BranchTrue(n) | Op::BranchFalse(n) => vec![*n, pc + 1],
                _ if flow::comparison(op).is_some() => {
                    vec![flow::comparison(op).unwrap().0, pc + 1]
                }
                _ => vec![pc + 1],
            };
            for next in successors {
                if next >= stacks.len() {
                    return Err(fail(pc, "control flow leaves function body"));
                }
                if let Some(old) = &stacks[next] {
                    if old != &stack {
                        return Err(fail(next, "incompatible value stack at join"));
                    }
                } else {
                    stacks[next] = Some(stack.clone());
                    work.push_back(next);
                }
            }
        }
        self.verify_outputs(i, &stacks)?;
        Ok(stacks)
    }
}

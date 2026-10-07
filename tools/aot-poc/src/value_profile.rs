//! Bounded inline value records. All managed addresses borrow active frame storage.
use super::{Error, flow};
use neoclr::metadata::{FunctionRef, Instruction as Op, Representation, Type};
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Ty {
    Int,
    Byte,
    Bool,
    Unit,
    Erased,
    Literal, // Immutable image/explicit invocation-arena UTF-8; not a general managed String.
    Record(usize),
    Address(Box<Ty>),
}
impl Ty {
    pub fn address(self) -> Self {
        Self::Address(Box::new(self))
    }
}
pub(super) struct Profile<'a> {
    pub input: &'a neoclr::Module,
    layouts: Vec<Vec<usize>>,
    widths: Vec<usize>,
    names: HashMap<&'a str, Vec<usize>>,
    pub args: Vec<Vec<Ty>>,
    pub locals: Vec<Vec<Ty>>,
    pub results: Vec<Option<Ty>>,
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

impl<'a> Profile<'a> {
    pub fn new(input: &'a neoclr::Module) -> Result<Self, Error> {
        if input.name == "System" || input.types.len() > 32 || input.functions.len() > 128 {
            return Err(
                "value profile requires an application with at most 32 types and 128 functions"
                    .into(),
            );
        }
        for t in &input.types {
            if !t.implements.is_empty() {
                return Err(format!(
                    "{}: implemented interfaces require a later AOT profile",
                    t.name
                )
                .into());
            }
            let static_owner = crate::selection::static_owner(t);
            if (t.is_reference_type && !static_owner)
                || t.representation != Representation::Record
                || t.enum_info.is_some()
                || t.base.is_some()
                || (t.is_abstract && !static_owner)
                || !t.generic_parameters.is_empty()
                || !t.generic_constraints.is_empty()
                || t.packing.is_some()
                || t.minimum_size.is_some()
                || t.fields.len() > 8
                || t.fields.iter().any(|f| {
                    f.deferred
                        || !matches!(
                            f.ty,
                            Type::Int32 | Type::Byte | Type::Boolean | Type::Named(_)
                        )
                })
            {
                return Err(format!("{}: value profile requires nongeneric records with at most eight Int32/Byte/Boolean/local-record fields", t.name).into());
            }
        }
        let mut p = Self {
            input,
            layouts: vec![vec![]; input.types.len()],
            widths: vec![0; input.types.len()],
            names: HashMap::new(),
            args: vec![],
            locals: vec![],
            results: vec![],
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
                if width > 8 {
                    return Err("value layout exceeds eight flattened lanes".into());
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
            if f.is_virtual
                || f.is_override
                || f.is_abstract
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
                    if !matches!(ty, Ty::Record(_)) {
                        return Err("value member requires a local record owner".into());
                    }
                    if f.instance {
                        if !f.receiver_byref {
                            return Err("value member requires a by-reference receiver".into());
                        }
                        args.push(ty.address());
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
                if let Op::Call(target) | Op::Construct(target) = op {
                    let callee = p.callee(target)?;
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
            Type::Int32 => Ty::Int,
            Type::Byte => Ty::Byte,
            Type::Boolean => Ty::Bool,
            Type::Void => Ty::Unit,
            Type::Value => Ty::Erased,
            Type::String => Ty::Literal,
            Type::Named(name) => {
                let i = self
                    .input
                    .types
                    .iter()
                    .position(|t| &t.name == name)
                    .ok_or("value profile requires a local named record")?;
                if self.input.types[i].is_reference_type {
                    return Err(
                        "static owners cannot be used as values or instance receivers".into(),
                    );
                }
                Ty::Record(i)
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
        if *t == Ty::Byte { Ty::Int } else { t.clone() }
    }
    pub fn byte_lanes(&self, t: &Ty) -> Vec<bool> {
        match t {
            Ty::Byte => vec![true],
            Ty::Record(i) if !self.input.types[*i].fields.is_empty() => self.input.types[*i]
                .fields
                .iter()
                .flat_map(|f| self.byte_lanes(&self.ty(&f.ty).expect("admitted field")))
                .collect(),
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
        if matches!(t, Ty::Address(_) | Ty::Literal) {
            8
        } else {
            self.lanes(t) as u32 * 4
        }
    }
    pub fn field_offset(&self, t: &Ty, index: usize) -> usize {
        let t = if let Ty::Address(t) = t {
            t.as_ref()
        } else {
            t
        };
        let Ty::Record(owner) = t else {
            unreachable!("checked field owner")
        };
        self.layouts[*owner][index]
    }
    pub fn field(&self, t: &Ty, index: usize) -> Result<Ty, Error> {
        let t = if let Ty::Address(t) = t {
            t.as_ref()
        } else {
            t
        };
        let Ty::Record(owner) = t else {
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
                Op::ConvertInt32 | Op::ConvertUInt8 => (),
                Op::PackValue(_) | Op::IsValue(_) | Op::UnpackValue(_) => {
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
                    state.stack.push(None);
                }
                Op::Call(target) | Op::Construct(target) => {
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
                | Op::String(_)
                | Op::ConvertInt32
                | Op::ConvertUInt8
                | Op::Bool(_)
                | Op::Void
                | Op::Dup
                | Op::Pop
                | Op::Field(_)
                | Op::SetField(_)
                | Op::FieldAddress(_)
                | Op::Call(_)
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
                Op::PackValue(t) | Op::IsValue(t) | Op::UnpackValue(t) => {
                    erased_tag(t)?;
                }
                Op::InitializeObject(Type::String) => {
                    return Err(fail(pc, "String default requires a later ownership profile"));
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
                if pop(s)? != Self::stack_type(expected) {
                    return Err(fail(pc, "value operand type mismatch"));
                }
                Ok(())
            };
            match op {
                Op::Int(_) => stack.push(Ty::Int),
                Op::String(_) => stack.push(Ty::Literal),
                Op::ConvertInt32 | Op::ConvertUInt8 => {
                    take(&mut stack, &Ty::Int)?;
                    stack.push(Ty::Int);
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
                        return Err(fail(pc, "newobj requires a record"));
                    };
                    for f in self.input.types[owner].fields.iter().rev() {
                        take(&mut stack, &self.ty(&f.ty)?)?;
                    }
                    stack.push(t);
                }
                Op::Field(n) | Op::FieldAddress(n) => {
                    let owner = pop(&mut stack)?;
                    let t = self.field(&owner, *n)?;
                    if matches!(op, Op::FieldAddress(_)) {
                        if !matches!(owner, Ty::Address(_)) {
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
                    stack.push(if matches!(owner, Ty::Address(_)) {
                        Ty::Unit
                    } else {
                        owner
                    });
                }
                Op::Call(target) | Op::Construct(target) => {
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

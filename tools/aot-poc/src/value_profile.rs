//! Bounded inline value records. All managed addresses borrow active frame storage.
use super::{Error, flow};
use neoclr::metadata::{FunctionRef, Instruction as Op, Representation, Type};
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Ty {
    Int,
    Bool,
    Unit,
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
    names: HashMap<&'a str, usize>,
    pub args: Vec<Vec<Ty>>,
    pub locals: Vec<Vec<Ty>>,
    pub results: Vec<Option<Ty>>,
}
pub(super) type Stacks = Vec<Option<Vec<Ty>>>;

impl<'a> Profile<'a> {
    pub fn new(input: &'a neoclr::Module) -> Result<Self, Error> {
        if input.name == "System" || input.types.len() > 32 || input.functions.len() > 128 {
            return Err(
                "value profile requires an application with at most 32 types and 128 functions"
                    .into(),
            );
        }
        for t in &input.types {
            if t.is_reference_type
                || t.representation != Representation::Record
                || t.enum_info.is_some()
                || t.base.is_some()
                || !t.implements.is_empty()
                || t.is_abstract
                || !t.generic_parameters.is_empty()
                || !t.generic_constraints.is_empty()
                || t.packing.is_some()
                || t.minimum_size.is_some()
                || t.fields.len() > 8
                || t.fields.iter().any(|f| {
                    f.deferred || !matches!(f.ty, Type::Int32 | Type::Boolean | Type::Named(_))
                })
            {
                return Err(format!("{}: value profile requires nongeneric records with at most eight Int32/Boolean/local-record fields", t.name).into());
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
            if p.names.insert(&f.name, i).is_some() {
                return Err("value profile does not support overloaded names".into());
            }
            if f.is_virtual
                || f.is_override
                || f.is_abstract
                || f.impl_flags != 0
                || f.pinvoke.is_some()
                || !f.generic_parameters.is_empty()
                || !f.generic_arguments.is_empty()
                || !f.generic_constraints.is_empty()
                || !f.out_parameters.is_empty()
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
            let mut args = vec![];
            if let Some(owner) = &f.owner {
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
            } else if f.instance || f.receiver_byref {
                return Err("instance member requires a record owner".into());
            }
            for t in &f.parameters {
                args.push(p.stored(t)?);
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
            Type::Boolean => Ty::Bool,
            Type::Void => Ty::Unit,
            Type::Named(name) => Ty::Record(
                self.input
                    .types
                    .iter()
                    .position(|t| &t.name == name)
                    .ok_or("value profile requires a local named record")?,
            ),
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
    pub fn lanes(&self, t: &Ty) -> usize {
        match t {
            Ty::Record(i) => self.widths[*i],
            _ => 1,
        }
    }
    pub fn bytes(&self, t: &Ty) -> u32 {
        if matches!(t, Ty::Address(_)) {
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
        let i = *self
            .names
            .get(target.name.as_str())
            .ok_or("value profile does not support external calls")?;
        let f = &self.input.functions[i];
        if f.owner != target.owner
            || f.instance != target.instance
            || f.parameters != target.parameters
            || !target.generic_arguments.is_empty()
            || target
                .definition
                .as_ref()
                .is_some_and(|d| Some(d) != f.definition.as_ref())
        {
            return Err("value member call identity/signature mismatch".into());
        }
        Ok(i)
    }
    pub fn root(&self, root: &str) -> Result<usize, Error> {
        let i = *self.names.get(root).ok_or("root function not found")?;
        let f = &self.input.functions[i];
        if f.instance
            || self.results[i] != Some(Ty::Int)
            || !(self.args[i].is_empty() || self.args[i] == [Ty::Int])
        {
            return Err("value root must be static () -> Int32 or (Int32) -> Int32".into());
        }
        Ok(i)
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
                        return Err(fail(pc, "cannot address or replace a borrowed receiver"));
                    }
                }
                Op::Load(n) | Op::Store(n) | Op::LocalAddress(n) if *n < self.locals[i].len() => (),
                Op::Int(_)
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
                if &pop(s)? != expected {
                    return Err(fail(pc, "value operand type mismatch"));
                }
                Ok(())
            };
            match op {
                Op::Int(_) => stack.push(Ty::Int),
                Op::Bool(_) => stack.push(Ty::Bool),
                Op::Void => stack.push(Ty::Unit),
                Op::Arg(n) => stack.push(self.args[i][*n].clone()),
                Op::Load(n) => stack.push(self.locals[i][*n].clone()),
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
                    stack.push(t);
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
                        stack.push(t);
                    }
                }
                Op::SetField(n) => {
                    let value = pop(&mut stack)?;
                    let owner = pop(&mut stack)?;
                    if self.field(&owner, *n)? != value {
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
                        stack.push(t.clone());
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
                Op::Branch(_) => (),
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
                Op::Return => vec![],
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
        Ok(stacks)
    }
}

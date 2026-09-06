use crate::{
    Fault, Module, Value,
    metadata::{Case, Instruction as Op, Type},
};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub instructions: usize,
    pub frames: usize,
    pub stack: usize,
    pub heap_objects: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            instructions: 100_000,
            frames: 256,
            stack: 4096,
            heap_objects: 4096,
        }
    }
}

#[derive(Debug)]
pub struct Execution {
    pub value: Value,
    pub output: Vec<String>,
    /// Retained until the execution result is dropped; references index this arena.
    pub heap: Vec<Value>,
}

fn signature(name: &str) -> Option<(Vec<Type>, Type)> {
    Some(match name {
        "System.Int32.Parse" => (
            vec![Type::String],
            Type::Result(Box::new(Type::Int32), Box::new(Type::Error)),
        ),
        "System.Int32.Divide" => (
            vec![Type::Int32, Type::Int32],
            Type::Result(Box::new(Type::Int32), Box::new(Type::Error)),
        ),
        "System.Console.WriteLine" => (vec![Type::String], Type::Void),
        _ => return None,
    })
}

pub(crate) fn validate(module: &Module) -> Result<(), Fault> {
    if module.name.is_empty() {
        return Err(Fault::new("module name must not be empty"));
    }
    if module.format != 1 {
        return Err(Fault::new("unsupported module format (expected 1)"));
    }
    let mut names = HashSet::new();
    for def in &module.types {
        if def.name.is_empty() || !names.insert(&def.name) {
            return Err(Fault::new("empty or duplicate type name"));
        }
        let mut fields = HashSet::new();
        for field in &def.fields {
            if field.name.is_empty() || !fields.insert(&field.name) {
                return Err(Fault::new("empty or duplicate field name"));
            }
            check_type(&field.ty, module)?;
        }
    }
    names.clear();
    for function in &module.functions {
        if function.name.is_empty()
            || !names.insert(&function.name)
            || signature(&function.name).is_some()
        {
            return Err(Fault::new("empty, duplicate or reserved function name"));
        }
        for ty in function
            .parameters
            .iter()
            .chain(&function.locals)
            .chain([&function.returns])
        {
            check_type(ty, module)?;
        }
        if function.body.is_empty() {
            return Err(Fault::new("empty function body"));
        }
        for op in &function.body {
            match op {
                Op::Branch(i) | Op::BranchTrue(i) if *i >= function.body.len() => {
                    return Err(Fault::new("branch outside function"));
                }
                Op::Arg(i) if *i >= function.parameters.len() => {
                    return Err(Fault::new("argument index outside signature"));
                }
                Op::Load(i) | Op::Store(i) if *i >= function.locals.len() => {
                    return Err(Fault::new("local index outside signature"));
                }
                Op::Call(name)
                    if signature(name).is_none()
                        && !module.functions.iter().any(|f| &f.name == name) =>
                {
                    return Err(Fault::new(format!("unknown function {name}")));
                }
                Op::New(name) if !module.types.iter().any(|t| &t.name == name) => {
                    return Err(Fault::new(format!("unknown type {name}")));
                }
                Op::None(ty) | Op::Ok(ty) | Op::Err(ty) => check_type(ty, module)?,
                _ => (),
            }
        }
    }
    let entry = module
        .functions
        .iter()
        .find(|f| f.name == module.entry)
        .ok_or_else(|| Fault::new("entry function not found"))?;
    if !entry.parameters.is_empty() {
        return Err(Fault::new("entry must have no parameters"));
    }
    Ok(())
}

fn check_type(ty: &Type, module: &Module) -> Result<(), Fault> {
    match ty {
        Type::Named(name) if !module.types.iter().any(|t| &t.name == name) => {
            Err(Fault::new(format!("unknown type {name}")))
        }
        Type::Option(t) | Type::Ref(t) => check_type(t, module),
        Type::Result(t, e) => {
            check_type(t, module)?;
            check_type(e, module)
        }
        _ => Ok(()),
    }
}

struct Frame {
    function: usize,
    pc: usize,
    args: Vec<Value>,
    locals: Vec<Option<Value>>,
    stack: Vec<Value>,
}

impl Frame {
    fn new(function: usize, args: Vec<Value>, module: &Module) -> Self {
        Self {
            function,
            pc: 0,
            args,
            locals: vec![None; module.functions[function].locals.len()],
            stack: vec![],
        }
    }
    fn pop(&mut self) -> Result<Value, Fault> {
        self.stack
            .pop()
            .ok_or_else(|| Fault::new("evaluation stack underflow"))
    }
    fn int(&mut self) -> Result<i32, Fault> {
        match self.pop()? {
            Value::Int32(n) => Ok(n),
            _ => Err(Fault::new("expected Int32")),
        }
    }
    fn args(&mut self, types: &[Type]) -> Result<Vec<Value>, Fault> {
        if self.stack.len() < types.len() {
            return Err(Fault::new("not enough arguments"));
        }
        let args = self.stack.split_off(self.stack.len() - types.len());
        for (value, ty) in args.iter().zip(types) {
            expect(value, ty)?;
        }
        Ok(args)
    }
}

fn expect(value: &Value, ty: &Type) -> Result<(), Fault> {
    if value.ty() == *ty {
        Ok(())
    } else {
        Err(Fault::new(format!("expected {ty:?}, got {:?}", value.ty())))
    }
}

pub fn run(module: &Module, limits: Limits) -> Result<Execution, Fault> {
    validate(module)?;
    if limits.frames == 0 {
        return Err(Fault::new("frame limit exceeded"));
    }
    let entry = module
        .functions
        .iter()
        .position(|f| f.name == module.entry)
        .ok_or_else(|| Fault::new("missing entry"))?;
    let mut frames = vec![Frame::new(entry, vec![], module)];
    let mut heap: Vec<Value> = vec![];
    let mut output = vec![];
    for _ in 0..limits.instructions {
        let frame = frames
            .last_mut()
            .ok_or_else(|| Fault::new("missing frame"))?;
        let function = &module.functions[frame.function];
        let pc = frame.pc;
        let op = function.body.get(pc).ok_or_else(|| Fault {
            message: "function fell through without ret".into(),
            function: Some(function.name.clone()),
            instruction: Some(pc),
        })?;
        frame.pc += 1;
        let context = function.name.clone();
        // Host Result propagates terminal faults; there is no guest exception machinery.
        let step = (|| -> Result<Option<Value>, Fault> {
            let frame = frames
                .last_mut()
                .ok_or_else(|| Fault::new("missing frame"))?;
            match op {
                Op::Int(n) => frame.stack.push(Value::Int32(*n)),
                Op::Bool(b) => frame.stack.push(Value::Boolean(*b)),
                Op::String(s) => frame.stack.push(Value::String(s.clone())),
                Op::Void => frame.stack.push(Value::Void),
                Op::Error(s) => frame.stack.push(Value::Error(s.clone())),
                Op::Arg(i) => frame.stack.push(frame.args[*i].clone()),
                Op::Load(i) => frame.stack.push(
                    frame.locals[*i]
                        .clone()
                        .ok_or_else(|| Fault::new("read of uninitialized local"))?,
                ),
                Op::Store(i) => {
                    let value = frame.pop()?;
                    expect(&value, &function.locals[*i])?;
                    frame.locals[*i] = Some(value);
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
                Op::Add | Op::Sub | Op::Mul | Op::AddChecked | Op::SubChecked | Op::MulChecked => {
                    let right = frame.int()?;
                    let left = frame.int()?;
                    let value = match op {
                        Op::Add => Some(left.wrapping_add(right)),
                        Op::Sub => Some(left.wrapping_sub(right)),
                        Op::Mul => Some(left.wrapping_mul(right)),
                        Op::AddChecked => left.checked_add(right),
                        Op::SubChecked => left.checked_sub(right),
                        _ => left.checked_mul(right),
                    };
                    frame.stack.push(Value::Int32(
                        value.ok_or_else(|| Fault::new("Int32 overflow"))?,
                    ));
                }
                Op::Equal => {
                    let right = frame.pop()?;
                    let left = frame.pop()?;
                    expect(&right, &left.ty())?;
                    frame.stack.push(Value::Boolean(left == right));
                }
                Op::Less => {
                    let right = frame.int()?;
                    let left = frame.int()?;
                    frame.stack.push(Value::Boolean(left < right));
                }
                Op::Branch(i) => frame.pc = *i,
                Op::BranchTrue(i) => match frame.pop()? {
                    Value::Boolean(true) => frame.pc = *i,
                    Value::Boolean(false) => (),
                    _ => return Err(Fault::new("brtrue requires Boolean")),
                },
                Op::Call(name) => {
                    if let Some((parameters, _)) = signature(name) {
                        let args = frame.args(&parameters)?;
                        frame.stack.push(intrinsic(name, args, &mut output)?);
                    } else {
                        let index = module
                            .functions
                            .iter()
                            .position(|f| &f.name == name)
                            .ok_or_else(|| Fault::new("unknown function"))?;
                        let args = frame.args(&module.functions[index].parameters)?;
                        if frames.len() >= limits.frames {
                            return Err(Fault::new("frame limit exceeded"));
                        }
                        frames.push(Frame::new(index, args, module));
                    }
                }
                Op::Return => {
                    let value = frame.pop()?;
                    expect(&value, &function.returns)?;
                    if !frame.stack.is_empty() {
                        return Err(Fault::new("ret requires exactly one value"));
                    }
                    frames.pop();
                    if let Some(caller) = frames.last_mut() {
                        caller.stack.push(value);
                    } else {
                        return Ok(Some(value));
                    }
                }
                Op::New(name) => {
                    let def = module
                        .types
                        .iter()
                        .find(|t| &t.name == name)
                        .ok_or_else(|| Fault::new("unknown type"))?;
                    let types: Vec<_> = def.fields.iter().map(|f| f.ty.clone()).collect();
                    let fields = frame.args(&types)?;
                    frame.stack.push(Value::Object {
                        name: name.clone(),
                        fields,
                    });
                }
                Op::Field(i) => {
                    let Value::Object { fields, .. } = frame.pop()? else {
                        return Err(Fault::new(
                            "ldfld requires object value (use heap.load for a reference)",
                        ));
                    };
                    frame.stack.push(
                        fields
                            .get(*i)
                            .ok_or_else(|| Fault::new("field index out of range"))?
                            .clone(),
                    );
                }
                Op::SetField(i) => {
                    let value = frame.pop()?;
                    let Value::Object { name, mut fields } = frame.pop()? else {
                        return Err(Fault::new("stfld requires object value"));
                    };
                    let field = fields
                        .get_mut(*i)
                        .ok_or_else(|| Fault::new("field index out of range"))?;
                    expect(&value, &field.ty())?;
                    *field = value;
                    frame.stack.push(Value::Object { name, fields });
                }
                Op::HeapNew => {
                    if heap.len() >= limits.heap_objects {
                        return Err(Fault::new("heap object limit exceeded"));
                    }
                    let value = frame.pop()?;
                    let target = value.ty();
                    let index = heap.len();
                    heap.push(value);
                    frame.stack.push(Value::Reference { index, target });
                }
                Op::HeapLoad => {
                    let Value::Reference { index, .. } = frame.pop()? else {
                        return Err(Fault::new("heap.load requires Ref"));
                    };
                    frame.stack.push(
                        heap.get(index)
                            .ok_or_else(|| Fault::new("invalid reference"))?
                            .clone(),
                    );
                }
                Op::HeapStore => {
                    let value = frame.pop()?;
                    let Value::Reference { index, target } = frame.pop()? else {
                        return Err(Fault::new("heap.store requires Ref followed by value"));
                    };
                    expect(&value, &target)?;
                    *heap
                        .get_mut(index)
                        .ok_or_else(|| Fault::new("invalid reference"))? = value;
                    frame.stack.push(Value::Void);
                }
                Op::Some => {
                    let value = frame.pop()?;
                    frame.stack.push(Value::Union {
                        ty: Type::Option(Box::new(value.ty())),
                        case: Case::Some,
                        payload: Box::new(value),
                    });
                }
                Op::None(ty) => frame.stack.push(Value::Union {
                    ty: Type::Option(Box::new(ty.clone())),
                    case: Case::None,
                    payload: Box::new(Value::Void),
                }),
                Op::Ok(error) => {
                    let value = frame.pop()?;
                    let success = value.ty();
                    frame
                        .stack
                        .push(Value::result(value, success, error.clone(), Case::Ok));
                }
                Op::Err(success) => {
                    let value = frame.pop()?;
                    let error = value.ty();
                    frame
                        .stack
                        .push(Value::result(value, success.clone(), error, Case::Err));
                }
                Op::IsCase(wanted) | Op::LoadCase(wanted) => {
                    let Value::Union { ty, case, payload } = frame.pop()? else {
                        return Err(Fault::new("case instruction requires union"));
                    };
                    let valid = match ty {
                        Type::Option(_) => matches!(wanted, Case::Some | Case::None),
                        Type::Result(_, _) => matches!(wanted, Case::Ok | Case::Err),
                        _ => false,
                    };
                    if !valid {
                        return Err(Fault::new("case does not belong to union type"));
                    }
                    if matches!(op, Op::IsCase(_)) {
                        frame.stack.push(Value::Boolean(case == *wanted));
                    } else if case == *wanted {
                        frame.stack.push(*payload);
                    } else {
                        return Err(Fault::new("union case mismatch"));
                    }
                }
                Op::Fault(message) => return Err(Fault::new(message)),
            }
            Ok(None)
        })();
        match step {
            Ok(Some(value)) => {
                return Ok(Execution {
                    value,
                    output,
                    heap,
                });
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
    Err(Fault::new("instruction limit exceeded"))
}

fn intrinsic(name: &str, args: Vec<Value>, output: &mut Vec<String>) -> Result<Value, Fault> {
    match (name, args.as_slice()) {
        ("System.Int32.Parse", [Value::String(text)]) => Ok(match text.parse::<i32>() {
            Ok(n) => Value::result(Value::Int32(n), Type::Int32, Type::Error, Case::Ok),
            Err(_) => Value::result(
                Value::Error("InvalidInt32".into()),
                Type::Int32,
                Type::Error,
                Case::Err,
            ),
        }),
        ("System.Int32.Divide", [Value::Int32(a), Value::Int32(b)]) => {
            Ok(match a.checked_div(*b) {
                Some(n) => Value::result(Value::Int32(n), Type::Int32, Type::Error, Case::Ok),
                None => Value::result(
                    Value::Error(
                        if *b == 0 {
                            "DivisionByZero"
                        } else {
                            "Overflow"
                        }
                        .into(),
                    ),
                    Type::Int32,
                    Type::Error,
                    Case::Err,
                ),
            })
        }
        ("System.Console.WriteLine", [Value::String(text)]) => {
            output.push(text.clone());
            Ok(Value::Void)
        }
        _ => Err(Fault::new("invalid intrinsic invocation")),
    }
}

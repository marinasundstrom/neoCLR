//! Private String constructor lowering after original-scope verification.
//! Runtime String construction replaces a private receiver slot, not object fields.
use neoclr::metadata::{Instruction as Op, Type};
use serde_json::{Value, json};
type Error = Box<dyn std::error::Error>;

pub fn constructors(module: &mut neoclr::Module, report: &mut Value) -> Result<(), Error> {
    let rows: Vec<_> = module.functions.iter().enumerate()
        .filter(|(_, f)| f.owner == Some(Type::String) && f.instance && f.name.ends_with("..ctor"))
        .map(|(i, _)| i).collect();
    for &index in &rows {
        let f = &mut module.functions[index];
        if f.receiver_byref || f.receiver_readonly || !f.no_result || f.returns != Type::Void ||
            f.is_virtual || f.is_override || f.is_abstract || !f.interface_implementations.is_empty() {
            return Err("String constructor requires a verified ordinary receiver-slot body".into());
        }
        // Chained constructors need an explicit receiver-writeback contract.
        if f.body.iter().any(|op| matches!(op, Op::Call(t) if t.owner == Some(Type::String) && t.name.ends_with("..ctor"))) {
            return Err("chained String constructors require a later factory projection".into());
        }
        let receiver = f.locals.len();
        f.locals.push(Type::String);
        if !f.local_names.is_empty() { f.local_names.push(None); }
        let mut positions = Vec::with_capacity(f.body.len());
        let mut at = 2;
        for op in &f.body { positions.push(at); at += if matches!(op, Op::Return) { 2 } else { 1 }; }
        let mut body = vec![Op::String(String::new()), Op::Store(receiver)];
        for op in &f.body {
            let mut op = op.clone();
            match &mut op {
                Op::Arg(0) => op = Op::Load(receiver),
                Op::StoreArg(0) => op = Op::Store(receiver),
                Op::Arg(i) | Op::StoreArg(i) => *i -= 1,
                Op::Receiver { argument: true, .. } => return Err("String factory does not admit argument receiver operands".into()),
                Op::ArgumentAddress(_) => return Err("String factory does not admit argument-address operands".into()),
                Op::Return => body.push(Op::Load(receiver)),
                Op::Branch(i) | Op::BranchTrue(i) | Op::BranchFalse(i) | Op::BranchEqual(i) | Op::BranchNotEqual(i)
                | Op::BranchGreater(i) | Op::BranchGreaterUnsigned(i) | Op::BranchLess(i) | Op::BranchLessUnsigned(i)
                | Op::BranchGreaterEqual(i) | Op::BranchGreaterEqualUnsigned(i) | Op::BranchLessEqual(i) | Op::BranchLessEqualUnsigned(i) => *i = positions[*i],
                Op::Switch(targets) => for i in targets { *i = positions[*i]; },
                _ => {}
            }
            body.push(op);
        }
        f.body = body; f.owner = None; f.namespace = "System".into(); f.instance = false;
        f.no_result = false; f.returns = Type::String;
    }
    for f in &mut module.functions {
        for op in &mut f.body {
            if let Op::Construct(target) = op {
                if target.definition.as_ref().is_some_and(|id| rows.contains(&(id.index as usize))) {
                    target.owner = None; target.instance = false;
                    *op = Op::Call(target.clone());
                }
            }
        }
    }
    report["stringConstructorProjections"] = json!(rows.iter().map(|index| json!({
        "compiledIndex":index, "policy":"verified receiver-slot constructor to private String factory; original body with local receiver and relocated branches"
    })).collect::<Vec<_>>());
    Ok(())
}

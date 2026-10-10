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
        // Intrinsic String has receiver-slot storage, not an Object allocation.
        // Elide only the verified empty Object initializer, never an arbitrary
        // constructor body or a call elsewhere in the method.
        let skip = match module.functions[index].body.as_slice() {
            [Op::Arg(0), Op::Call(target), ..]
                if target.owner == Some(Type::Named("System.Object".into()))
                    && target.name == "System.Object..ctor" && target.instance
                    && target.parameters.is_empty() && target.generic_arguments.is_empty() => {
                let callee = super::selection::resolve(module, target)?;
                let base = &module.functions[callee];
                if !base.no_result || base.returns != Type::Void || base.impl_flags != 0
                    || !matches!(base.body.as_slice(), [Op::Return]) {
                    return Err("String factory requires an empty Object initializer".into());
                }
                2
            }
            _ => 0,
        };
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
        for (pc, op) in f.body.iter().enumerate() {
            positions.push(at);
            if pc >= skip { at += if matches!(op, Op::Return) { 2 } else { 1 }; }
        }
        let mut body = vec![Op::String(String::new()), Op::Store(receiver)];
        for op in f.body.iter().skip(skip) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use neoclr::metadata::FunctionRef;

    fn fixture() -> neoclr::Module {
        let mut module = neoclr::assemble(".module Factories\n.function Base() -> Void\nldvoid\nret\n.end\n.function Factory() -> Void\nldvoid\nret\n.end").unwrap();
        let base = &mut module.functions[0];
        base.name = "System.Object..ctor".into();
        base.owner = Some(Type::Named("System.Object".into()));
        base.instance = true;
        base.no_result = true;
        base.body = vec![Op::Return];
        let target = FunctionRef { definition: base.definition.clone(), name: base.name.clone(),
            owner: base.owner.clone(), instance: true, generic_arguments: vec![], parameters: vec![] };
        let factory = &mut module.functions[1];
        factory.name = "System.String..ctor".into();
        factory.owner = Some(Type::String);
        factory.instance = true;
        factory.no_result = true;
        factory.body = vec![Op::Arg(0), Op::Call(target), Op::Branch(3), Op::Return];
        module
    }

    #[test]
    fn string_factory_removes_empty_base_initializer_and_relocates_branches() {
        let mut module = fixture();
        constructors(&mut module, &mut json!({})).unwrap();
        let factory = &module.functions[1];
        assert!(matches!(factory.body.as_slice(), [Op::String(text), Op::Store(0),
            Op::Branch(3), Op::Load(0), Op::Return] if text.is_empty()));
        assert!(!factory.instance && factory.owner.is_none());
        assert_eq!(factory.returns, Type::String);
    }

    #[test]
    fn string_factory_does_not_discard_base_constructor_effects() {
        let mut module = fixture();
        module.functions[0].body = vec![Op::String("effect".into()), Op::Pop, Op::Return];
        assert!(constructors(&mut module, &mut json!({})).unwrap_err().to_string()
            .contains("empty Object initializer"));
    }
}

//! Closed enum metadata queries; construction uses the ordinary value and box paths.
use neoclr::metadata::{Instruction as Op, Type};
use serde_json::{Value, json};
type Error = Box<dyn std::error::Error>;

pub fn bind(
    input: &mut neoclr::Module,
    source: &neoclr::Module,
    report: &mut Value,
) -> Result<(), Error> {
    let queries: Vec<_> = report["functions"]
        .as_array()
        .ok_or("missing functions")?
        .iter()
        .filter(|row| {
            matches!(
                row["name"].as_str(),
                Some(
                    "neoCLR.Runtime.EnumCount"
                        | "neoCLR.Runtime.EnumName"
                        | "neoCLR.Runtime.EnumBits"
                        | "neoCLR.Runtime.EnumBox"
                )
            )
        })
        .cloned()
        .collect();
    if queries.is_empty() {
        return Ok(());
    }
    let object_index = report["objectBaseProjection"]["compiledIndex"]
        .as_u64()
        .ok_or("enum parsing requires Object projection")? as usize;
    let object = Type::Named(input.types[object_index].name.clone());
    let mut tokens = Vec::new();
    for op in input.functions.iter().flat_map(|f| &f.body) {
        if let Op::LoadTypeToken(ty) = op
            && !tokens.contains(ty)
        {
            tokens.push(ty.clone());
        }
    }
    let mut enums = Vec::new();
    for token in tokens {
        let semantic = super::reflection_metadata::semantic_token(input, report, &token)?;
        if let Some(info) = source
            .type_definition(&semantic)
            .and_then(|d| d.enum_info.as_ref())
        {
            if info.underlying != Type::Int32 {
                return Err("enum parsing requires supported Int32 storage".into());
            }
            enums.push((token, info.clone()));
        }
    }
    for row in queries {
        let index = row["compiledIndex"].as_u64().ok_or("missing query index")? as usize;
        let name = row["name"].as_str().ok_or("missing query name")?;
        let count = name.ends_with("EnumCount");
        let names = name.ends_with("EnumName");
        let boxed = name.ends_with("EnumBox");
        let f = &mut input.functions[index];
        let result = if names {
            Type::String
        } else if boxed {
            object.clone()
        } else {
            Type::Int32
        };
        let params = if count {
            vec![Type::RuntimeTypeHandle]
        } else {
            vec![Type::RuntimeTypeHandle, Type::Int32]
        };
        if f.name != name
            || f.owner.is_some()
            || f.instance
            || f.receiver_byref
            || f.receiver_readonly
            || f.parameters != params
            || f.returns != result
            || f.no_result
            || f.impl_flags != 0x1000
            || f.pinvoke.is_some()
            || !f.body.is_empty()
            || !f.locals.is_empty()
            || f.is_virtual
            || f.is_override
            || f.is_abstract
            || !f.generic_parameters.is_empty()
            || !f.generic_arguments.is_empty()
            || !f.generic_constraints.is_empty()
            || !f.interface_implementations.is_empty()
            || !f.out_parameters.is_empty()
            || !f.out_when_true.is_empty()
            || !f.readonly_parameters.is_empty()
        {
            return Err(
                format!("{name} requires exact reserved enum InternalCall signature").into(),
            );
        }
        let mut body = Vec::new();
        for (token, info) in &enums {
            let next = body.len() + 3;
            body.extend([
                Op::Arg(0),
                Op::LoadTypeToken(token.clone()),
                Op::Equal,
                Op::BranchFalse(0),
            ]);
            if count {
                body.extend([Op::Int(info.members.len() as i32), Op::Return]);
            } else if boxed {
                body.extend([
                    Op::Arg(1),
                    Op::New(token.clone()),
                    Op::BoxValue(token.clone()),
                    Op::Return,
                ]);
            } else {
                for (i, member) in info.members.iter().enumerate() {
                    let skip = body.len() + 6;
                    body.extend([
                        Op::Arg(1),
                        Op::Int(i as i32),
                        Op::Equal,
                        Op::BranchFalse(skip),
                        if names {
                            Op::String(member.name.clone())
                        } else {
                            Op::Int(member.value)
                        },
                        Op::Return,
                    ]);
                }
                body.push(Op::Fault("enum member index out of range".into()));
            }
            body[next] = Op::BranchFalse(body.len());
        }
        body.push(Op::Fault(
            "enum parsing requires a retained enum type".into(),
        ));
        f.impl_flags = 0;
        f.body = body;
        report["nativeBindings"]
            .as_array_mut()
            .ok_or("missing bindings")?
            .push(json!({
            "name":name,"compiledIndex":index,"definition":row["definition"],
            "implementation":"enum-metadata-closed-v1","enumCount":enums.len(),
            "policy":"source enum constants; ordinary value construction and boxing"}));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queries_require_exact_contract_and_fault_without_enum_tokens() {
        let source = neoclr::assemble(".module Queries\n.type class Root\n.end\n.function neoCLR.Runtime.EnumCount(RuntimeTypeHandle) -> Int32\n.methodimpl InternalCall\n.end\n").unwrap();
        let report = json!({"functions":[{"name":"neoCLR.Runtime.EnumCount","compiledIndex":0}],
            "objectBaseProjection":{"compiledIndex":0},"nativeBindings":[]});
        let mut input = source.clone();
        bind(&mut input, &source, &mut report.clone()).unwrap();
        assert!(matches!(&input.functions[0].body[..], [Op::Fault(_)]));
        for change in 0..5 {
            let mut input = source.clone();
            let f = &mut input.functions[0];
            match change {
                0 => f.parameters[0] = Type::Int32,
                1 => f.returns = Type::Boolean,
                2 => f.impl_flags = 0,
                3 => f.body = vec![Op::Int(0), Op::Return],
                _ => f.out_parameters = vec![0],
            }
            assert!(bind(&mut input, &source, &mut report.clone()).is_err());
        }
    }
}

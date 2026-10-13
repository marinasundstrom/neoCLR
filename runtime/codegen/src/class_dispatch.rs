//! Closed-world class dispatch uses ordinary typed IL thunks. Direct calls retain
//! the original body, so base calls never recurse through an override dispatcher.
type Error = Box<dyn std::error::Error>;
use neoclr::metadata::{FunctionRef, Instruction as Op, MemberId, Type};
use serde_json::{Value, json};

pub(super) fn project(module: &mut neoclr::Module, report: &mut Value) -> Result<(), Error> {
    let rows = report["classVirtualDispatch"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    for row in rows {
        let contract = row["contractCompiledIndex"]
            .as_u64()
            .ok_or("invalid class contract")? as usize;
        let mut thunk = module.functions[contract].clone();
        let index = module.functions.len();
        if index >= crate::limits::FUNCTIONS {
            return Err("class dispatch exceeds selected function limit".into());
        }
        thunk.name = format!(
            "{}.$aot_ClassDispatch_{index}",
            thunk
                .owner
                .as_ref()
                .and_then(Type::definition_name)
                .ok_or("class dispatch requires owner")?
        );
        thunk.definition = Some(MemberId {
            module: module.name.clone(),
            revision: module.revision.clone(),
            index: index as u32,
        });
        thunk.origin = None;
        thunk.is_virtual = false;
        thunk.is_override = false;
        thunk.is_abstract = false;
        thunk.custom_attributes.clear();
        thunk.interface_implementations.clear();
        thunk.locals.clear();
        thunk.local_names.clear();
        thunk.body.clear();
        let reference = FunctionRef {
            definition: thunk.definition.clone(),
            name: thunk.name.clone(),
            owner: thunk.owner.clone(),
            instance: true,
            generic_arguments: vec![],
            parameters: thunk.parameters.clone(),
        };
        let mut targets = row["targets"]
            .as_array()
            .ok_or("invalid class targets")?
            .clone();
        // isinst includes descendants: test the most derived allocation types first.
        targets.sort_by_key(|target| {
            std::cmp::Reverse(depth(
                module,
                target["typeCompiledIndex"].as_u64().unwrap() as usize,
            ))
        });
        for target in targets {
            let ty = target["typeCompiledIndex"]
                .as_u64()
                .ok_or("invalid class receiver")? as usize;
            let method = target["functionCompiledIndex"]
                .as_u64()
                .ok_or("invalid class implementation")? as usize;
            let implementation = &module.functions[method];
            let branch = thunk.body.len() + 3;
            thunk.body.extend([
                Op::Arg(0),
                Op::IsInstance(Type::Named(module.types[ty].name.clone())),
                Op::ReferenceIsNull,
                Op::BranchTrue(0),
            ]);
            thunk.body.push(Op::Arg(0));
            thunk.body.push(Op::CastClass(
                implementation
                    .owner
                    .clone()
                    .ok_or("class target requires owner")?,
            ));
            for argument in 0..thunk.parameters.len() {
                thunk.body.push(Op::Arg(argument + 1));
            }
            thunk.body.push(Op::Call(FunctionRef {
                definition: implementation.definition.clone(),
                name: implementation.name.clone(),
                owner: implementation.owner.clone(),
                instance: true,
                generic_arguments: vec![],
                parameters: implementation.parameters.clone(),
            }));
            thunk.body.push(Op::Return);
            thunk.body[branch] = Op::BranchTrue(thunk.body.len());
        }
        thunk.body.push(Op::Fault(
            "class dispatch receiver is outside the closed world".into(),
        ));
        for function in &mut module.functions {
            for op in &mut function.body {
                if let Op::CallVirtual(target) = op {
                    if target
                        .definition
                        .as_ref()
                        .is_some_and(|id| id.index as usize == contract)
                    {
                        *target = reference.clone();
                    }
                }
            }
        }
        // The original abstract declaration has no direct callers in verified IL.
        // Give the private projection a terminal body; virtual calls use the thunk.
        if module.functions[contract].is_abstract {
            module.functions[contract].is_abstract = false;
            module.functions[contract].body = vec![Op::Fault(
                "abstract class method has no direct implementation".into(),
            )];
        }
        module.functions.push(thunk);
        report["classVirtualDispatch"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|r| r["contractCompiledIndex"] == contract)
            .unwrap()["thunkCompiledIndex"] = json!(index);
    }
    Ok(())
}

fn depth(module: &neoclr::Module, index: usize) -> usize {
    let mut count = 0;
    let mut parent = module.types[index].base.as_ref();
    while let Some(ty) = parent.and_then(|t| module.type_definition(t)) {
        count += 1;
        if count >= 64 {
            break;
        }
        parent = ty.base.as_ref();
    }
    count
}

//! Verification-only restoration of implicit interface-to-Object conversions.
//! Native interface views already carry the same reference. Original load-set
//! verification and Profile::analyze establish assignability before this runs.
use super::{Error, Profile, Ty, profile::Stacks};
use neoclr::metadata::{Instruction as Op, Type};

pub(super) fn verify(p: &Profile<'_>, flows: &[Stacks]) -> Result<(), Error> {
    let mut projected = None;
    for (i, function) in p.input.functions.iter().enumerate() {
        let mut edits = Vec::new();
        for (pc, op) in function.body.iter().enumerate() {
            let Some(stack) = flows[i].get(pc).and_then(Option::as_ref) else {
                continue;
            };
            if !stack.iter().any(|ty| matches!(ty, Ty::Interface(_))) {
                continue;
            }
            let expected = match op {
                Op::Call(target) | Op::CallVirtual(target)
                    if crate::selection::callable_invoke(target).is_none() =>
                {
                    p.input.functions[p.callee(target)?].argument_types()
                }
                Op::Store(local) => vec![function.locals[*local].clone()],
                Op::StoreArg(argument) => vec![function.argument_types()[*argument].clone()],
                Op::Return if !function.no_result => vec![function.returns.clone()],
                _ => continue,
            };
            let actual = &stack[stack.len() - expected.len()..];
            let conversions = actual
                .iter()
                .zip(&expected)
                .map(|(actual, expected)| {
                    Ok(matches!(actual, Ty::Interface(_)) && p.is_object_base(&p.ty(expected)?))
                })
                .collect::<Result<Vec<_>, Error>>()?;
            if conversions.iter().any(|convert| *convert) {
                edits.push((pc, expected, actual.to_vec(), conversions));
            }
        }
        if edits.is_empty() {
            continue;
        }
        let f = &mut projected.get_or_insert_with(|| p.input.clone()).functions[i];
        let mut prefixes = vec![vec![]; f.body.len()];
        for (pc, expected, actual, conversions) in edits {
            // Spill only the consumed arguments, preserving lower stack values,
            // evaluation order, null and the original callvirt instruction.
            let first = f.locals.len();
            for (n, ty) in expected.iter().enumerate() {
                f.locals.push(if conversions[n] {
                    let Ty::Interface(index) = actual[n] else {
                        unreachable!()
                    };
                    Type::Named(p.input.types[index].name.clone())
                } else {
                    ty.clone()
                });
                if !f.local_names.is_empty() {
                    f.local_names.push(None);
                }
            }
            prefixes[pc].extend((0..expected.len()).rev().map(|n| Op::Store(first + n)));
            for (n, ty) in expected.into_iter().enumerate() {
                prefixes[pc].push(Op::Load(first + n));
                if conversions[n] {
                    prefixes[pc].push(Op::CastClass(ty));
                }
            }
        }
        let mut positions = Vec::with_capacity(f.body.len());
        let mut position = 0;
        for prefix in &prefixes {
            positions.push(position);
            position += prefix.len() + 1;
        }
        let mut body = Vec::with_capacity(position);
        for (pc, original) in f.body.iter().enumerate() {
            body.append(&mut prefixes[pc]);
            let mut op = original.clone();
            match &mut op {
                Op::Branch(i)
                | Op::BranchTrue(i)
                | Op::BranchFalse(i)
                | Op::BranchEqual(i)
                | Op::BranchNotEqual(i)
                | Op::BranchGreater(i)
                | Op::BranchGreaterUnsigned(i)
                | Op::BranchLess(i)
                | Op::BranchLessUnsigned(i)
                | Op::BranchGreaterEqual(i)
                | Op::BranchGreaterEqualUnsigned(i)
                | Op::BranchLessEqual(i)
                | Op::BranchLessEqualUnsigned(i) => *i = positions[*i],
                Op::Switch(targets) => {
                    for i in targets {
                        *i = positions[*i];
                    }
                }
                _ => {}
            }
            body.push(op);
        }
        f.body = body;
    }
    // Scalar-union box/cast elision keeps the original box instruction for
    // verification. Linking renamed the Object root to a private backend name;
    // the VM verifier still requires its canonical declaration for that opcode.
    if p.input.functions.iter().any(|f| {
        f.body
            .iter()
            .enumerate()
            .any(|(pc, _)| crate::scalar_unions::box_pair(f, pc))
    }) && !p.input.types.iter().any(|t| t.name == "System.Object")
    {
        let mut object = p
            .input
            .types
            .iter()
            .enumerate()
            .find(|(i, _)| p.is_object_base(&Ty::Reference(*i)))
            .map(|(_, t)| t.clone())
            .ok_or("scalar union verification requires the explicit Object root")?;
        object.name = "System.Object".into();
        object.definition = None;
        object.origin = None;
        projected
            .get_or_insert_with(|| p.input.clone())
            .types
            .push(object);
    }
    // This copy is only a verification artifact. Code generation, GC plans and
    // diagnostic instruction offsets continue to use the original admitted body.
    neoclr::LoadedProgram::new(projected.as_ref().unwrap_or(p.input))
        .and_then(|program| program.verify())
        .map_err(|error| error.to_string())?;
    Ok(())
}

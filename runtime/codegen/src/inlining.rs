//! Shared, bounded leaf inlining for JIT and AOT. Never inline a faulting,
//! allocating, calling or branching body: those require inline-frame/root maps.
use cranelift_codegen::ir::{InstBuilder, Value, condcodes::IntCC, types};
use cranelift_frontend::FunctionBuilder;
use neoclr::metadata::{
    AGGRESSIVE_INLINING, Function, INLINING_FLAGS, Instruction as Op, NO_INLINING, Type,
};

pub(crate) fn eligible(f: &Function) -> bool {
    let budget = if f.impl_flags & AGGRESSIVE_INLINING != 0 {
        64
    } else {
        8
    };
    if f.impl_flags & NO_INLINING != 0
        || f.impl_flags & !INLINING_FLAGS != 0
        || f.instance
        || f.is_abstract
        || f.is_virtual
        || f.pinvoke.is_some()
        || f.no_result
        || !f.locals.is_empty()
        || !f.generic_parameters.is_empty()
        || !matches!(f.returns, Type::Int32 | Type::Boolean)
        || f.parameters
            .iter()
            .any(|t| !matches!(t, Type::Int32 | Type::Boolean))
        || f.body.is_empty()
        || f.body.len() > budget
    {
        return false;
    }
    // Check the stack before emitting anything, even for unverified test inputs.
    let mut depth = 0usize;
    for (pc, op) in f.body.iter().enumerate() {
        match op {
            Op::Arg(i) if *i < f.parameters.len() => depth += 1,
            Op::Int(_) | Op::Bool(_) => depth += 1,
            Op::Add | Op::Sub | Op::Mul | Op::Equal | Op::Less | Op::Greater if depth >= 2 => {
                depth -= 1
            }
            Op::Return => return pc + 1 == f.body.len() && depth == 1,
            _ => return false,
        }
    }
    false
}

/// Arguments have already been evaluated once by the caller. Only direct calls
/// use this path; type-initializer checks remain at the original call site.
pub(crate) fn emit(b: &mut FunctionBuilder<'_>, f: &Function, arguments: &[Value]) -> Value {
    debug_assert!(eligible(f));
    let mut stack = Vec::new();
    for op in &f.body {
        match op {
            Op::Arg(i) => stack.push(arguments[*i]),
            Op::Int(v) => stack.push(b.ins().iconst(types::I32, i64::from(*v))),
            Op::Bool(v) => stack.push(b.ins().iconst(types::I32, i64::from(*v))),
            Op::Add | Op::Sub | Op::Mul | Op::Equal | Op::Less | Op::Greater => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();
                let value = match op {
                    Op::Add => b.ins().iadd(left, right),
                    Op::Sub => b.ins().isub(left, right),
                    Op::Mul => b.ins().imul(left, right),
                    _ => {
                        let cc = match op {
                            Op::Equal => IntCC::Equal,
                            Op::Less => IntCC::SignedLessThan,
                            _ => IntCC::SignedGreaterThan,
                        };
                        let value = b.ins().icmp(cc, left, right);
                        b.ins().uextend(types::I32, value)
                    }
                };
                stack.push(value);
            }
            Op::Return => return stack.pop().unwrap(),
            _ => unreachable!("eligible leaf"),
        }
    }
    unreachable!("eligible return")
}

#[cfg(test)]
mod tests {
    use super::*;
    fn leaf(flags: u16, body: Vec<Op>) -> Function {
        serde_json::from_value(serde_json::json!({"name":"Leaf", "parameters":["Int32"], "returns":"Int32", "impl_flags":flags, "body":body})).unwrap()
    }
    #[test]
    fn backend_optimization_defaults_to_speed() {
        if std::env::var_os("NEOCLR_TEST_OPT_LEVEL").is_none() {
            assert_eq!(
                cranelift_codegen::settings::Flags::new(crate::backend_settings()).opt_level(),
                cranelift_codegen::settings::OptLevel::Speed,
            );
        }
    }

    #[test]
    fn policy_and_safety_boundaries() {
        let small = vec![Op::Arg(0), Op::Int(1), Op::Add, Op::Return];
        assert!(eligible(&leaf(0, small.clone())));
        assert!(eligible(&leaf(AGGRESSIVE_INLINING, small.clone())));
        assert!(!eligible(&leaf(NO_INLINING, small.clone())));
        assert!(!eligible(&leaf(INLINING_FLAGS, small)));
        let mut large = vec![Op::Arg(0)];
        for _ in 0..8 {
            large.extend([Op::Int(1), Op::Add]);
        }
        large.push(Op::Return);
        assert!(!eligible(&leaf(0, large.clone())));
        assert!(eligible(&leaf(AGGRESSIVE_INLINING, large)));
        assert!(!eligible(&leaf(
            AGGRESSIVE_INLINING,
            vec![Op::Arg(0), Op::Int(0), Op::Divide, Op::Return]
        )));
        assert!(!eligible(&leaf(
            AGGRESSIVE_INLINING,
            vec![Op::Arg(0), Op::Return, Op::Return]
        )));
    }
    fn fixture(flags: &str, extra: &str) -> neoclr::Module {
        neoclr::assemble(&format!(".module InlineTest\n.references ()\n.function Main(Int32 value) -> Int32\nldarg 0\ncall Leaf(Int32)\nret\n.end\n.function Leaf(Int32 value) -> Int32\n{flags}ldarg 0\nldc.i4 3\nmul\n{extra}ret\n.end\n")).unwrap()
    }

    #[test]
    fn jit_and_both_aot_targets_obey_default_and_overrides() {
        use object::{Object, ObjectSection};
        for (flags, extra, expected, inline) in [
            ("", "", 21, true),
            (".methodimpl NoInlining\n", "", 21, false),
            (".methodimpl NoInlining|AggressiveInlining\n", "", 21, false),
            (
                "",
                "ldc.i4 1\nadd\nldc.i4 2\nadd\nldc.i4 3\nadd\n",
                27,
                false,
            ),
            (
                ".methodimpl AggressiveInlining\n",
                "ldc.i4 1\nadd\nldc.i4 2\nadd\nldc.i4 3\nadd\n",
                27,
                true,
            ),
        ] {
            let input = fixture(flags, extra);
            neoclr::verify(&input).unwrap();
            if cfg!(target_os = "macos") || cfg!(target_os = "windows") {
                let result = crate::jit::execute(&input, "Main", 7, 1, false, None).unwrap();
                assert_eq!(result["status"], 0);
                assert_eq!(result["value"], expected);
            }
            for target in [
                crate::compiler::Target::MacosArm64,
                crate::compiler::Target::WindowsX64,
            ] {
                // A direct call requires a relocation. This checks the optimization
                // contract without depending on a particular machine opcode encoding.
                let bytes =
                    crate::compiler::compile_for_target(&input, "Main", false, None, target)
                        .unwrap();
                let object = object::File::parse(bytes.as_slice()).unwrap();
                let count = object
                    .sections()
                    .filter(|s| s.kind() == object::SectionKind::Text)
                    .map(|s| s.relocations().count())
                    .sum::<usize>();
                assert_eq!(count, usize::from(!inline), "{flags} {extra}");
                // Also exercise the managed/value lowering path used by eager and lazy JIT.
                let bytes = crate::compiler::values::compile(&input, "Main", None, target).unwrap();
                let object = object::File::parse(bytes.as_slice()).unwrap();
                let count = object
                    .sections()
                    .filter(|s| s.kind() == object::SectionKind::Text)
                    .map(|s| s.relocations().count())
                    .sum::<usize>();
                assert_eq!(count, 1 + usize::from(!inline), "managed {flags} {extra}");
            }
        }
    }
    #[test]
    #[ignore = "explicit optimization qualification; writes to NEOCLR_INLINE_EVIDENCE"]
    fn qualification() {
        use std::{fs, path::PathBuf};
        let directory =
            PathBuf::from(std::env::var("NEOCLR_INLINE_EVIDENCE").expect("evidence directory"));
        fs::create_dir_all(&directory).unwrap();
        let mut report = Vec::new();
        for (label, flags) in [("default", ""), ("noinline", ".methodimpl NoInlining\n")] {
            let source = format!(
                ".module InlineBench\n.references ()\n.function Main(Int32 count) -> Int32\n.local Int32 i\n.local Int32 sum\nldc.i4 0\nstloc 0\nldc.i4 0\nstloc 1\nloop:\nldloc 1\nldloc 0\ncall Leaf(Int32)\nadd\nstloc 1\nldloc 0\nldc.i4 1\nadd\nstloc 0\nldloc 0\nldarg 0\nblt loop\nldloc 1\nret\n.end\n.function Leaf(Int32 value) -> Int32\n{flags}ldarg 0\nldc.i4 8\nmul\nret\n.end\n"
            );
            let input = neoclr::assemble(&source).unwrap();
            let result = crate::jit::execute(&input, "Main", 2_000_000, 11, false, None).unwrap();
            let expected =
                (0i32..2_000_000).fold(0i32, |sum, i| sum.wrapping_add(i.wrapping_mul(8)));
            assert_eq!(result["value"], expected);
            assert!(
                result["invocations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|v| v["status"] == 0 && v["value"] == expected)
            );
            let mut objects = Vec::new();
            for (name, target) in [
                ("arm64", crate::compiler::Target::MacosArm64),
                ("x64", crate::compiler::Target::WindowsX64),
            ] {
                let started = std::time::Instant::now();
                let bytes =
                    crate::compiler::compile_for_target(&input, "Main", false, None, target)
                        .unwrap();
                objects.push(serde_json::json!({"target":name, "bytes":bytes.len(), "compileSeconds":started.elapsed().as_secs_f64()}));
                fs::write(directory.join(format!("{label}-{name}.o")), bytes).unwrap();
            }
            report.push(serde_json::json!({"case":label,"jit":result,"aot":objects}));
            fs::write(directory.join(format!("{label}.neoil")), source).unwrap();
        }
        fs::write(
            directory.join("results.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
    }
}

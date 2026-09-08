use neoclr::{Limits, LoadedProgram, RuntimeService, Value, assembler::parse_function_ref};
fn program() -> LoadedProgram {
    let module = neoclr::assemble(".module App").unwrap();
    LoadedProgram::new(&neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap()).unwrap()
}
fn floating(p: &LoadedProgram, name: &str, args: &[f64]) -> f64 {
    let sig = vec!["Double"; args.len()].join(",");
    let f = p
        .resolve_function(&parse_function_ref(&format!("System.Math::{name}({sig})")).unwrap())
        .unwrap();
    let Value::Double(v) = f
        .invoke(
            args.iter().copied().map(Value::Double).collect(),
            Limits::default(),
        )
        .unwrap()
        .value
    else {
        panic!("expected Double")
    };
    v
}
#[test]
fn double_methods_cover_finite_domains_nan_infinity_and_signed_zero() {
    let p = program();
    p.verify().unwrap();
    for (name, args, expected) in [
        ("Abs", vec![-3.5], 3.5),
        ("Sqrt", vec![25.0], 5.0),
        ("Pow", vec![2.0, 3.0], 8.0),
        ("Floor", vec![-1.2], -2.0),
        ("Ceiling", vec![-1.2], -1.0),
        ("Truncate", vec![-1.2], -1.0),
        ("Round", vec![2.5], 2.0),
        ("Round", vec![3.5], 4.0),
        ("Round", vec![-2.5], -2.0),
        ("Exp", vec![0.0], 1.0),
        ("Log", vec![1.0], 0.0),
        ("Log10", vec![100.0], 2.0),
        ("Sin", vec![0.0], 0.0),
        ("Cos", vec![0.0], 1.0),
        ("Tan", vec![0.0], 0.0),
        ("Min", vec![2.0, -3.0], -3.0),
        ("Max", vec![-3.0, 2.0], 2.0),
        ("Pow", vec![f64::NAN, 0.0], 1.0),
        ("Pow", vec![1.0, f64::NAN], 1.0),
        ("Log", vec![0.0], f64::NEG_INFINITY),
        ("Exp", vec![1000.0], f64::INFINITY),
        ("Sqrt", vec![f64::INFINITY], f64::INFINITY),
    ] {
        assert_eq!(floating(&p, name, &args), expected, "{name}: {args:?}");
    }
    for (name, args) in [
        ("Sqrt", vec![-1.0]),
        ("Pow", vec![-2.0, 0.5]),
        ("Log", vec![-1.0]),
        ("Sin", vec![f64::INFINITY]),
        ("Min", vec![f64::NAN, 1.0]),
        ("Max", vec![1.0, f64::NAN]),
    ] {
        assert!(floating(&p, name, &args).is_nan());
    }
    for (name, args, negative) in [
        ("Abs", vec![-0.0], false),
        ("Round", vec![-0.5], true),
        ("Sqrt", vec![-0.0], true),
        ("Min", vec![0.0, -0.0], true),
        ("Min", vec![-0.0, 0.0], true),
        ("Max", vec![0.0, -0.0], false),
        ("Max", vec![-0.0, 0.0], false),
    ] {
        assert_eq!(
            floating(&p, name, &args).to_bits(),
            if negative { (-0.0f64).to_bits() } else { 0 },
            "{name}"
        );
    }
    assert!((floating(&p, "Sqrt", &[2.0]) - std::f64::consts::SQRT_2).abs() < 1e-14);
    assert!((floating(&p, "Log", &[std::f64::consts::E]) - 1.0).abs() < 1e-14);
}
#[test]
fn integer_helpers_do_not_overflow_and_clamp_reports_reversed_bounds() {
    let p = program();
    for (sig, args, expected) in [
        ("Min(Int32,Int32)", vec![i32::MIN, i32::MAX], i32::MIN),
        ("Max(Int32,Int32)", vec![i32::MIN, i32::MAX], i32::MAX),
        ("Sign(Int32)", vec![i32::MIN], -1),
        ("Sign(Int32)", vec![0], 0),
        ("Sign(Int32)", vec![i32::MAX], 1),
    ] {
        let f = p
            .resolve_function(&parse_function_ref(&format!("System.Math::{sig}")).unwrap())
            .unwrap();
        assert_eq!(
            f.invoke(
                args.into_iter().map(Value::Int32).collect(),
                Limits::default()
            )
            .unwrap()
            .value,
            Value::Int32(expected)
        );
    }
    let source = r#"
func Main() -> int {
    System.Math.Clamp(0, 2, 1) match {
        Ok(let value) => { return -1 },
        Error(let error) => {
            if !error.ToString().Equals("InvalidRange") { return -2 }
        }
    }
    System.Math.Clamp(-100, 7, 7) match {
        Ok(let value) => { return value },
        Error(let error) => { return -3 }
    }
}
"#;
    let module = neoclr::frontend::compile(source).unwrap();
    assert_eq!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(7)
    );
}
#[test]
fn neo_math_sample_roundtrips_and_float_comparisons_handle_nan() {
    for source in [
        include_str!("../examples/source/math.neo"),
        r#"
func Main() -> int {
    let nan = System.Math.Sqrt(-1.0)
    if nan == nan || nan < 0.0 || nan > 0.0 || nan <= 0.0 || nan >= 0.0 { return -1 }
    if !(nan != nan) { return -2 }
    if 1e2 / 2.0 != 50.0 { return -3 }
    if -1.5 + 2.5 != 1.0 { return -4 }
    var count = 0
    for i in 0..<3 { count = count + 1 }
    if count != 3 { return -5 }
    return 42
}
"#,
    ] {
        let module = neoclr::frontend::compile(source).unwrap();
        let p =
            LoadedProgram::new(&neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap())
                .unwrap();
        p.verify().unwrap();
        assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    }
    for source in [
        "func Main() -> double { return 1e999 }",
        "func Main() -> double { return 1e+ }",
        "func Main() -> double { return 1 + 2.0 }",
        "func Main() -> double { return System.Math.Sqrt(4) }",
    ] {
        assert!(neoclr::frontend::compile(source).is_err(), "{source}");
    }
}
#[test]
fn math_helpers_declare_only_the_required_services() {
    let p = program();
    for (sig, required) in [
        ("Sqrt(Double)", true),
        ("Round(Double)", true),
        ("Min(Int32,Int32)", false),
        ("Clamp(Int32,Int32,Int32)", false),
    ] {
        let graph = p
            .analyze_reachability(
                &[parse_function_ref(&format!("System.Math::{sig}")).unwrap()],
                16,
            )
            .unwrap();
        assert_eq!(
            graph
                .required_services()
                .contains(&RuntimeService::MathOperations),
            required
        );
    }
    for declaration in [
        ".function neoCLR.Runtime.MathSqrt(Int32) -> Double",
        ".function neoCLR.Runtime.MathPow(Double,Double) -> Int32",
    ] {
        assert!(
            neoclr::assemble(&format!(
                ".module System\n{declaration}\n.methodimpl InternalCall\n.end"
            ))
            .is_err()
        );
    }
}

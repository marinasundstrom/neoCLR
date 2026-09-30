//! Focused in-memory metadata admission comparison. Run with --release after producing
//! the C# --container-integration fixtures. No filesystem/process startup in timed loops.
use neoclr::{ExecutionOptions, LoadedProgram, metadata_container};
use serde_json::json;
use std::{hint::black_box, time::Instant};

fn median_us(mut action: impl FnMut(), iterations: usize) -> f64 {
    for _ in 0..3 {
        action();
    }
    let mut samples = Vec::new();
    for _ in 0..7 {
        let start = Instant::now();
        for _ in 0..iterations {
            action();
        }
        samples.push(start.elapsed().as_secs_f64() * 1e6 / iterations as f64);
    }
    samples.sort_by(f64::total_cmp);
    samples[3]
}
fn main() {
    let directory = std::env::args().nth(1).expect("fixture directory");
    let read = |name: &str| std::fs::read(std::path::Path::new(&directory).join(name)).unwrap();
    black_box(neoclr::library::system().unwrap()); // Exclude one-time bundled System parsing.
    let mut rows = Vec::new();
    for (name, json_name, json_pe_name, binary_pe_name, iterations) in [
        (
            "constant42",
            "Module.neo.json",
            "Valid.dll",
            "Binary.dll",
            2000,
        ),
        (
            "65_functions",
            "Large.neo.json",
            "LargeJson.dll",
            "LargeBinary.dll",
            100,
        ),
    ] {
        eprintln!("Measuring {name}");
        let raw = String::from_utf8(read(json_name)).unwrap();
        let json_pe = read(json_pe_name);
        let binary_pe = read(binary_pe_name);
        let module = neoclr::load(&raw).unwrap();
        for image in [&json_pe, &binary_pe] {
            assert_eq!(
                serde_json::to_value(&module).unwrap(),
                serde_json::to_value(metadata_container::load(image).unwrap()).unwrap()
            );
        }
        let program = LoadedProgram::new(&module).unwrap();
        program.verify().unwrap();
        assert!(matches!(
            program.run(ExecutionOptions::default()).unwrap().value,
            neoclr::Value::Int32(42)
        ));
        rows.push(json!({
            "case": name,
            "bytes": { "json": raw.len(), "json_pe": json_pe.len(), "binary_pe": binary_pe.len() },
            "decode_iterations_per_sample": iterations,
            "load_iterations_per_sample": 5,
            "median_microseconds": {
                "json_pe_binding_and_decode": median_us(|| { black_box(metadata_container::decode(black_box(&json_pe)).unwrap()); }, iterations),
                "binary_pe_binding_and_decode": median_us(|| { black_box(metadata_container::decode(black_box(&binary_pe)).unwrap()); }, iterations),
                "json_load_with_system_link_validation": median_us(|| { black_box(neoclr::load(black_box(&raw)).unwrap()); }, 5),
                "json_pe_load_with_system_link_validation": median_us(|| { black_box(metadata_container::load(black_box(&json_pe)).unwrap()); }, 5),
                "binary_pe_load_with_system_link_validation": median_us(|| { black_box(metadata_container::load(black_box(&binary_pe)).unwrap()); }, 5),
                "link_prepare_warm_system": median_us(|| { black_box(LoadedProgram::new(black_box(&module)).unwrap()); }, 3),
                "verify_prepared_program": median_us(|| { black_box(program.verify().unwrap()); }, 3),
                "run_prepared_program": median_us(|| { black_box(program.run(ExecutionOptions::default()).unwrap()); }, 3)
            }
        }));
    }
    println!("{}", serde_json::to_string_pretty(&json!({
        "benchmark": "bounded native container admission",
        "samples": 7, "statistic": "median per-iteration microseconds", "profile": "release",
        "platform": {"os": std::env::consts::OS, "arch": std::env::consts::ARCH},
        "excluded": ["file IO", "process startup", "one-time System parsing", "compiler emission"],
        "cases": rows
    })).unwrap());
}

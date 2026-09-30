//! Release-only comparison of the current JSON decoder and standalone schema-3 assemblies.
//! Read files and establish full metadata/behavior equality before timing. No I/O in phase loops.
use neoclr::{ExecutionOptions, LoadedProgram, Module, metadata_container};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{hint::black_box, path::Path, time::Instant};

// Mirrors src/lib.rs::decode_module exactly: Value parse, format check, then typed conversion.
// Keep distinct from the direct-typed JSON diagnostic, which is NOT the current loader.
fn current_json(text: &str) -> Module {
    let value: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(value.get("format").and_then(|v| v.as_u64()), Some(5));
    serde_json::from_value(value).unwrap()
}
fn summary(samples: &[f64], iterations: usize) -> serde_json::Value {
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    json!({"samples_us": samples, "median_us": sorted[sorted.len()/2], "min_us": sorted[0],
        "max_us": sorted[sorted.len()-1], "iterations_per_sample": iterations})
}
fn measured(iterations: usize, mut action: impl FnMut()) -> f64 {
    let start = Instant::now();
    for _ in 0..iterations {
        action();
    }
    start.elapsed().as_secs_f64() * 1e6 / iterations as f64
}
fn common_phase(mut action: impl FnMut()) -> serde_json::Value {
    action();
    let samples: Vec<_> = (0..9).map(|_| measured(1, &mut action)).collect();
    summary(&samples, 1)
}
fn main() {
    assert!(!cfg!(debug_assertions), "use --release");
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(
        args.len(),
        3,
        "JSON baseline directory and native assembly directory required"
    );
    let json_dir = Path::new(&args[1]);
    let binary_dir = Path::new(&args[2]);
    let mut cases = Vec::new();
    let mut decoded = Vec::new();
    for (name, iterations) in [("System", 2), ("FloatingMath", 10)] {
        eprintln!("Decoding {name}: rotating four decoder paths across nine samples");
        let text = std::fs::read_to_string(json_dir.join(format!("{name}.json"))).unwrap();
        let binary = std::fs::read(binary_dir.join(format!("{name}.neox"))).unwrap();
        let expected = current_json(&text);
        let compact = serde_json::to_string(&expected).unwrap();
        let paths = [
            "json_current_pretty",
            "json_current_compact",
            "json_direct_typed_diagnostic",
            "assembly_schema3",
        ];
        let decode = |index| -> Module {
            match index {
                0 => current_json(black_box(&text)),
                1 => current_json(black_box(&compact)),
                2 => {
                    let module: Module = serde_json::from_str(black_box(&text)).unwrap();
                    assert_eq!(module.format, 5);
                    module
                }
                3 => metadata_container::decode_envelope(black_box(&binary)).unwrap(),
                _ => unreachable!(),
            }
        };
        for i in 0..4 {
            assert_eq!(
                serde_json::to_value(decode(i)).unwrap(),
                serde_json::to_value(&expected).unwrap()
            );
        }
        let mut samples: [Vec<f64>; 4] = Default::default();
        for sample in 0..9 {
            for turn in 0..4 {
                let index = (sample + turn) % 4;
                samples[index].push(measured(iterations, || {
                    black_box(decode(index));
                }));
            }
        }
        let mut timings = serde_json::Map::new();
        for i in 0..4 {
            timings.insert(paths[i].into(), summary(&samples[i], iterations));
        }
        cases.push(json!({"name": name, "bytes": {"json_pretty": text.len(), "json_compact": compact.len(), "assembly": binary.len()},
            "sha256": {"json": format!("{:x}", Sha256::digest(text.as_bytes())), "assembly": format!("{:x}", Sha256::digest(&binary))},
            "decode_and_drop": timings}));
        decoded.push(expected);
    }
    eprintln!("Measuring common linking, verification and prepared-program execution phases");
    let system = &decoded[0];
    let app = &decoded[1];
    let program = LoadedProgram::with_library(app, system).unwrap();
    program.verify().unwrap();
    let expected: Vec<String> = (0..19).map(|_| "0".into()).chain(["-1".into()]).collect();
    assert_eq!(
        program.run(ExecutionOptions::default()).unwrap().output,
        expected
    );
    let phases = json!({
        "link_prepare_from_decoded": common_phase(|| { black_box(LoadedProgram::with_library(black_box(app), black_box(system)).unwrap()); }),
        "verify_prepared": common_phase(|| { black_box(program.verify().unwrap()); }),
        "execute_prepared": common_phase(|| { black_box(program.run(ExecutionOptions::default()).unwrap()); })
    });
    println!("{}", serde_json::to_string_pretty(&json!({
        "profile": "release", "samples": 9, "units": "microseconds per iteration", "decode_order": "rotate first path each sample",
        "decode_includes": ["deserialization", "allocation", "schema guard for binary", "dropping decoded module"],
        "excluded": ["file IO", "process startup", "assembly/compilation", "cold filesystem cache"],
        "direct_typed_json_is_diagnostic_only": true, "metadata_and_output_equal": true,
        "platform": {"os": std::env::consts::OS, "arch": std::env::consts::ARCH},
        "cases": cases, "common_phases": phases
    })).unwrap());
}

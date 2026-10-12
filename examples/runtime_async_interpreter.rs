//! Private runtime-async comparison host. Load once, execute with fresh heaps.
use neoclr::{Limits, LoadedProgram, Value, assembler::ModuleInput, metadata::TypeDefId};
use std::{fs, path::Path, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 4 {
        return Err(
            "expected assembly, bundle, repetitions and optional dependency assemblies".into(),
        );
    }
    let repetitions: usize = args[3].parse()?;
    if !(1..=20).contains(&repetitions) {
        return Err("repetitions must be 1..20".into());
    }
    let started = Instant::now();
    let lib = Path::new(&args[2]).join("lib");
    let system =
        neoclr::metadata_container::decode_envelope(&fs::read(lib.join("System.runtime.neox"))?)
            .map_err(|e| e.to_string())?;
    let mut images = vec![fs::read(&args[1])?];
    for name in [
        "System.Runtime",
        "System.Data",
        "System.Networking",
        "System.Web",
    ] {
        images.push(fs::read(lib.join(format!("{name}.dll")))?);
    }
    for dependency in &args[4..] {
        images.push(fs::read(dependency)?);
    }
    let root = neoclr::metadata_container::decode(&images[1]).map_err(|e| e.to_string())?;
    let index = root
        .types
        .iter()
        .position(|ty| ty.name == "System.Object")
        .ok_or("missing Object")?;
    let identity = TypeDefId {
        module: root.name,
        revision: root.revision,
        index: index.try_into()?,
    };
    let inputs: Vec<_> = images
        .iter()
        .map(|bytes| ModuleInput::MetadataPe(bytes))
        .collect();
    let modules = neoclr::assembler::read_modules_with_object_root(&inputs, &system, &identity)
        .map_err(|e| e.to_string())?;
    let program =
        LoadedProgram::with_modules_and_object_root(&modules[0], &system, &modules[1..], &identity)
            .map_err(|e| e.to_string())?;
    program.verify().map_err(|e| e.to_string())?;
    let prepare_seconds = started.elapsed().as_secs_f64();
    let mut runs = Vec::new();
    for _ in 0..repetitions {
        let started = Instant::now();
        let execution = program
            .run(Limits {
                instructions: 100_000_000,
                ..Limits::default()
            })
            .map_err(|e| e.to_string())?;
        let elapsed = started.elapsed().as_secs_f64();
        if execution.value != Value::Int32(0) || !execution.stderr.is_empty() {
            return Err("sample failed".into());
        }
        let gc = execution.heap.statistics();
        if gc.live_objects != 0 {
            return Err("completed invocation retains managed objects".into());
        }
        runs.push(serde_json::json!({"seconds": elapsed, "stdout": String::from_utf8(execution.stdout)?,
            "allocatedObjects": gc.allocated_objects, "reclaimedObjects": gc.reclaimed_objects,
            "liveObjects": gc.live_objects, "peakObjects": gc.peak_objects, "collections": gc.collections}));
    }
    println!(
        "{}",
        serde_json::json!({"prepareSeconds": prepare_seconds, "runs": runs})
    );
    Ok(())
}

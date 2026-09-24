//! Run a trusted imported async fixture under a small GC limit and report object counts.
use neoclr::{Limits, LoadedProgram, assemble};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if !(4..=5).contains(&args.len()) {
        return Err("usage: measure_async APP.neoil System.neoil HEAP_LIMIT [INSTRUCTION_LIMIT]".into());
    }
    let text = std::fs::read_to_string(&args[1])?;
    let system = assemble(&std::fs::read_to_string(&args[2])?).map_err(|e| e.to_string())?;
    let app =
        neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(&text)], &system)
            .map_err(|e| e.to_string())?
            .remove(0);
    let program = LoadedProgram::with_library(&app, &system).map_err(|e| e.to_string())?;
    program.verify().map_err(|e| e.to_string())?;
    let execution = program
        .run(Limits {
            heap_objects: args[3].parse()?,
            instructions: args
                .get(4)
                .map(|value| value.parse())
                .transpose()?
                .unwrap_or(Limits::default().instructions),
            ..Limits::default()
        })
        .map_err(|e| e.to_string())?;
    print!("{}", String::from_utf8(execution.stdout)?);
    let stats = execution.heap.statistics();
    eprintln!(
        "allocated={} live={} peak={} collections={} reclaimed={}",
        stats.allocated_objects,
        stats.live_objects,
        stats.peak_objects,
        stats.collections,
        stats.reclaimed_objects
    );
    if stats.live_objects != 0 {
        return Err("completed fixture retained managed objects".into());
    }
    Ok(())
}

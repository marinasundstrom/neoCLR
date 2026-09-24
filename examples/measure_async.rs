//! Run a trusted imported async fixture under a small GC limit and report object counts.
use neoclr::{ExecutionOptions, Limits, LoadedProgram, StdioConsole, assemble};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<_> = std::env::args().collect();
    let live = args.last().is_some_and(|arg| arg == "--live-output");
    if live {
        args.pop();
    }
    if !(4..=5).contains(&args.len()) {
        return Err("usage: measure_async APP.neoil System.neoil HEAP_LIMIT [INSTRUCTION_LIMIT] [--live-output]".into());
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
        .run(ExecutionOptions {
            console: live
                .then(|| std::sync::Arc::new(StdioConsole) as std::sync::Arc<dyn neoclr::Console>),
            limits: Limits {
                heap_objects: args[3].parse()?,
                instructions: args
                    .get(4)
                    .map(|value| value.parse())
                    .transpose()?
                    .unwrap_or(Limits::default().instructions),
                ..Limits::default()
            },
            ..ExecutionOptions::default()
        })
        .map_err(|e| e.to_string())?;
    if !live {
        print!("{}", String::from_utf8(execution.stdout)?);
    }
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

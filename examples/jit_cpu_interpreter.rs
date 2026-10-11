//! Private benchmark host: resolve once, invoke with fresh interpreter state.
use neoclr::{
    Limits, LoadedProgram, Value,
    metadata::{FunctionRef, Type},
};
use std::time::Instant;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        return Err("expected source, count and repetitions".into());
    }
    let count: i32 = args[2].parse()?;
    let repetitions: usize = args[3].parse()?;
    if !(0..=1_000_000).contains(&count) || !(1..=1000).contains(&repetitions) {
        return Err("benchmark bounds exceeded".into());
    }
    let started = Instant::now();
    let module =
        neoclr::assemble(&std::fs::read_to_string(&args[1])?).map_err(|e| e.to_string())?;
    let program = LoadedProgram::new(&module).map_err(|e| e.to_string())?;
    program.verify().map_err(|e| e.to_string())?;
    let function = program
        .resolve_function(&FunctionRef {
            definition: None,
            name: "Calculate".into(),
            owner: None,
            instance: false,
            generic_arguments: vec![],
            parameters: vec![Type::Int32],
        })
        .map_err(|e| e.to_string())?;
    let prepare_seconds = started.elapsed().as_secs_f64();
    let mut invocations = Vec::new();
    for _ in 0..repetitions {
        let started = Instant::now();
        let execution = function
            .invoke(
                vec![Value::Int32(count)],
                Limits {
                    instructions: 30_000_000,
                    ..Limits::default()
                },
            )
            .map_err(|e| e.to_string())?;
        let elapsed = started.elapsed().as_secs_f64();
        let Value::Int32(value) = execution.value else {
            return Err("expected Int32".into());
        };
        invocations.push(serde_json::json!({"status":0,"value":value,"invocationSeconds":elapsed}));
    }
    println!(
        "{}",
        serde_json::json!({"prepareSeconds":prepare_seconds,"invocations":invocations})
    );
    Ok(())
}

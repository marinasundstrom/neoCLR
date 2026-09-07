//! Inspect the closed calls and runtime imports needed by HelloWorld.
use neoclr::{LoadedProgram, assemble, assembler::parse_function_ref};

fn main() -> Result<(), neoclr::Fault> {
    let module = assemble(include_str!("hello.neoil"))?;
    let program = LoadedProgram::new(&module)?;
    let graph = program.analyze_reachability(&[parse_function_ref("Main()")?], 64)?;
    for (index, function) in graph.functions.iter().enumerate() {
        println!(
            "{index}: {} {:?} {:?}",
            function.target.name, function.target.definition, function.implementation
        );
        for service in &function.services {
            println!(
                "  service {:?} at {:?}",
                service.service, service.instruction
            );
        }
        for call in &function.calls {
            println!("  instruction {} -> {}", call.instruction, call.target);
        }
    }
    println!("Required services: {:?}", graph.required_services());
    println!(
        "Missing with no services: {:?}",
        graph.missing_services(&[])
    );
    Ok(())
}

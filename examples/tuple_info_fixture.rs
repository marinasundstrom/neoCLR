//! Explicit native type-token fixture; not Raven tuple emission support.
use neoclr::{
    metadata::{Instruction, Type},
    metadata_container,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let mut module =
        metadata_container::decode(&std::fs::read(&args[1])?).map_err(|e| e.to_string())?;
    let token = module
        .types
        .iter()
        .find(|t| t.origin.as_ref().is_some_and(|o| o.name == "TupleToken"))
        .expect("fixture declaration")
        .name
        .clone();
    let mut count = 0;
    for function in &mut module.functions {
        for instruction in &mut function.body {
            if let Instruction::LoadTypeToken(ty) = instruction {
                if ty == &Type::from_name(&token) {
                    *ty = Type::Tuple(vec![
                        Type::Int32,
                        Type::Int32,
                        Type::Tuple(vec![Type::String]),
                    ]);
                    count += 1;
                }
            }
        }
    }
    assert_eq!(count, 1, "fixture token count changed");
    std::fs::write(
        &args[2],
        metadata_container::write_module(&module).map_err(|e| e.to_string())?,
    )?;
    Ok(())
}

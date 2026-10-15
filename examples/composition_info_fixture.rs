//! Replace explicit test placeholder tokens; this is not compiler support.
use neoclr::{
    metadata::{Instruction, Type},
    metadata_container,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let mut module = metadata_container::decode(&std::fs::read(&args[1])?).map_err(|e| e.to_string())?;
    let find = |name: &str| {
        Type::from_name(&module
            .types
            .iter()
            .find(|t| t.origin.as_ref().is_some_and(|o| o.name == name))
            .unwrap()
            .name)
    };
    let left = find("Left");
    let right = find("Right");
    let third = find("Third");
    let replacements = [
        (
            find("UnionToken"),
            Type::Union(vec![left.clone(), right.clone()]),
        ),
        (
            find("IntersectionToken"),
            Type::Intersection(vec![left.clone(), right.clone()]),
        ),
        (
            find("NestedToken"),
            Type::Union(vec![left, Type::Intersection(vec![right, third])]),
        ),
    ];
    fn replace(ty: &mut Type, replacements: &[(Type, Type)]) {
        if let Some((_, target)) = replacements.iter().find(|(source, _)| source == ty) {
            *ty = target.clone();
        } else if let Type::Constructed { arguments, .. } = ty {
            for arg in arguments {
                replace(arg, replacements);
            }
        }
    }
    let mut count = 0;
    for function in &mut module.functions {
        for instruction in &mut function.body {
            if let Instruction::LoadTypeToken(ty) = instruction {
                let before = ty.clone();
                replace(ty, &replacements);
                count += usize::from(*ty != before);
            }
        }
    }
    assert_eq!(count, 4, "fixture token count changed");
    std::fs::write(&args[2], metadata_container::write_module(&module).map_err(|e| e.to_string())?)?;
    Ok(())
}

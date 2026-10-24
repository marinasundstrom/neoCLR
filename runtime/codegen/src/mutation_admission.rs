//! Call-closure gate for the experimental live-GC pipeline. Store lowering and
//! host capability validation remain separate obligations; this is not admission
//! of arbitrary providers merely because their name and ABI match.
use cranelift_codegen::ir::{self, ExternalName, InstructionData, Opcode, Signature};
use cranelift_module::{FuncId, Linkage, Module};
use std::collections::HashMap;
type Error = Box<dyn std::error::Error>;

pub(crate) fn check_call_closure(
    module: &impl Module,
    bodies: &[(FuncId, &ir::Function)],
    providers: &[(&str, Signature)],
) -> Result<(), Error> {
    let declarations: HashMap<_, _> = module.declarations().get_functions().collect();
    let mut generated = HashMap::new();
    for &(id, body) in bodies {
        let declaration = declarations
            .get(&id)
            .ok_or("missing generated declaration")?;
        if declaration.linkage == Linkage::Import || declaration.signature != body.signature {
            return Err("generated body does not match its definition contract".into());
        }
        if generated.insert(id, body).is_some() {
            return Err("duplicate generated body".into());
        }
    }
    let mut admitted = HashMap::new();
    for (name, signature) in providers {
        if admitted.insert(*name, signature).is_some() {
            return Err("duplicate provider contract".into());
        }
    }
    for &(_, body) in bodies {
        for block in body.layout.blocks() {
            for inst in body.layout.block_insts(block) {
                let instruction = &body.dfg.insts[inst];
                if !instruction.opcode().is_call() {
                    continue;
                }
                let InstructionData::Call {
                    opcode: Opcode::Call,
                    func_ref,
                    ..
                } = instruction
                else {
                    return Err(
                        "live-GC closure rejects indirect, tail and exceptional calls".into(),
                    );
                };
                let external = &body.dfg.ext_funcs[*func_ref];
                let ExternalName::User(reference) = external.name else {
                    return Err("live-GC closure requires an explicit module target".into());
                };
                let name = &body.params.user_named_funcs()[reference];
                if name.namespace != 0 {
                    return Err("invalid module function namespace".into());
                }
                let target = FuncId::from_u32(name.index);
                let declaration = declarations.get(&target).ok_or("undeclared call target")?;
                if body.dfg.signatures[external.signature] != declaration.signature {
                    return Err("call-site signature does not match target".into());
                }
                if generated.contains_key(&target) {
                    continue;
                }
                if declaration.linkage != Linkage::Import {
                    return Err("generated call target has no supplied body".into());
                }
                let signature = declaration
                    .name
                    .as_deref()
                    .and_then(|name| admitted.get(name))
                    .ok_or("native call target has no admitted provider contract")?;
                if **signature != declaration.signature {
                    return Err("native provider signature mismatch".into());
                }
            }
        }
    }
    Ok(())
}

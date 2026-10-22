//! Versioned C exports for scalar native libraries; private status ABI stays private.
use crate::{compiler, load_context, scalar_abi};
use cranelift_codegen::ir::{AbiParam, InstBuilder, StackSlotData, StackSlotKind, TrapCode, types};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{FuncId, Linkage, Module};
use neoclr::metadata::{Function, Type};
use std::{collections::HashSet, ffi::OsString, fs, io::Write, path::Path};
type Error = Box<dyn std::error::Error>;

fn hex(value: &str) -> String {
    value
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn tag(t: &Type) -> Result<String, Error> {
    Ok(match t {
        Type::Void => "v".into(),
        Type::Byte => "u8".into(),
        Type::Int32 => "i32".into(),
        Type::UInt32 => "u32".into(),
        Type::Int64 => "i64".into(),
        Type::UInt64 => "u64".into(),
        Type::IntPtr => "is".into(),
        Type::UIntPtr => "us".into(),
        Type::Single => "f32".into(),
        Type::Double => "f64".into(),
        Type::Ptr(inner) if scalar_abi::pointer_target(inner) => format!("p{}", tag(inner)?),
        _ => return Err("unsupported native export type".into()),
    })
}
fn symbol(module: &str, f: &Function) -> Result<String, Error> {
    let args = f
        .parameters
        .iter()
        .map(tag)
        .collect::<Result<Vec<_>, _>>()?
        .join("_");
    Ok(format!(
        "neoclr_c_v1_m{}_n{}_f{}_a{}_r{}",
        hex(module),
        hex(&f.namespace),
        hex(f
            .origin
            .as_ref()
            .map(|o| o.full_name.as_deref().unwrap_or(&o.name))
            .unwrap_or(&f.name)),
        args,
        if f.no_result {
            "v".into()
        } else {
            tag(&f.returns)?
        }
    ))
}

pub(super) fn run(mut args: Vec<OsString>, imports: bool) -> Result<i32, Error> {
    let dependency_args = load_context::split(&mut args);
    if args.len() < 5 || args.len() > 131 || (args.len() - 3) % 2 != 0 {
        return Err("usage: --native-library <input> <output.o> --export <function> [--export <function> ...] [--native-imports] [load-context pairs]".into());
    }
    let paths = load_context::parse(&dependency_args)?;
    let input = crate::read_module(Path::new(&args[1]))?;
    let mut selected = HashSet::new();
    let mut exports = Vec::new();
    for pair in args[3..].chunks_exact(2) {
        if pair[0] != "--export" {
            return Err("native library expects explicit --export function pairs".into());
        }
        let name = pair[1].to_str().ok_or("export name must be UTF-8")?;
        let matches: Vec<_> = input
            .functions
            .iter()
            .enumerate()
            .filter(|(_, f)| {
                f.name == name
                    || f.origin
                        .as_ref()
                        .is_some_and(|o| o.name == name || o.full_name.as_deref() == Some(name))
            })
            .collect();
        let [(index, f)] = matches.as_slice() else {
            return Err(format!("export must resolve exactly one free function: {name}").into());
        };
        if !selected.insert(*index)
            || f.owner.is_some()
            || f.pinvoke.is_some()
            || f.parameters.len() > 16
            || f.parameters
                .iter()
                .any(|t| scalar_abi::scalar_type(t).is_none())
            || (!f.no_result && scalar_abi::scalar_type(&f.returns).is_none())
        {
            return Err(format!("unsupported or duplicate export: {name}").into());
        }
        let assembly = f
            .origin
            .as_ref()
            .map(|o| {
                input
                    .assemblies
                    .iter()
                    .find(|a| a.full_name == o.assembly)
                    .map(|a| a.name.as_str())
                    .unwrap_or(&o.assembly)
            })
            .unwrap_or(&input.name);
        exports.push((*index, symbol(assembly, f)?));
    }
    let dependencies = paths
        .modules
        .iter()
        .map(|p| crate::read_module(Path::new(p)))
        .collect::<Result<Vec<_>, _>>()?;
    let object_root = load_context::object_root(&paths.modules, paths.object, &dependencies)?;
    let system = paths
        .system
        .map(|p| crate::read_module(Path::new(p)))
        .transpose()?;
    let verification = if dependency_args.is_empty() {
        None
    } else {
        Some(load_context::verification(
            &input,
            system
                .as_ref()
                .ok_or("native library load context requires --system")?,
            &dependencies,
            object_root.as_ref(),
        )?)
    };
    let object =
        compiler::compile_native_library(&input, &exports, imports, verification.as_ref())?;
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])?;
    output.write_all(&object)?;
    // A build report, not a runtime metadata sidecar or loader contract.
    let report: Vec<_> = exports
        .iter()
        .map(|(i, name)| {
            serde_json::json!({
                "function": input.functions[*i].origin.as_ref().map(|o| o.name.as_str()).unwrap_or(&input.functions[*i].name), "symbol": name
            })
        })
        .collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "abi": "neoclr-c-v1", "target": "aarch64-apple-darwin", "exports": report
        }))?
    );
    Ok(0)
}

pub(super) fn emit<M: Module>(
    module: &mut M,
    f: &Function,
    callee: FuncId,
    name: &str,
) -> Result<(), Error> {
    let mut context = module.make_context();
    context.func.signature = module.make_signature();
    context.func.signature.params = f
        .parameters
        .iter()
        .map(|t| AbiParam::new(scalar_abi::scalar_type(t).unwrap()))
        .collect();
    if !f.no_result {
        context
            .func
            .signature
            .returns
            .push(AbiParam::new(scalar_abi::scalar_type(&f.returns).unwrap()));
    }
    let export = module.declare_function(name, Linkage::Export, &context.func.signature)?;
    let target = module.declare_func_in_func(callee, &mut context.func);
    let mut frontend = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut context.func, &mut frontend);
        let entry = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.switch_to_block(entry);
        let mut args = b.block_params(entry).to_vec();
        let slot = b.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 8, 3));
        args.push(b.ins().stack_addr(types::I64, slot, 0));
        let call = b.ins().call(target, &args);
        let status = b.inst_results(call)[0];
        // C has no managed fault channel. Never return a fabricated successful value.
        b.ins().trapnz(status, TrapCode::unwrap_user(1));
        if f.no_result {
            b.ins().return_(&[]);
        } else {
            let result = b
                .ins()
                .stack_load(scalar_abi::scalar_type(&f.returns).unwrap(), slot, 0);
            b.ins().return_(&[result]);
        }
        b.seal_all_blocks();
        b.finalize();
    }
    module.define_function(export, &mut context)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn names_encode_identity_without_platform_decoration_or_row_indices() {
        let module = neoclr::assemble(
            ".module Library\n.function Echo(Int32 value) -> Int32\nldarg value\nret\n.end",
        )
        .unwrap();
        let f = &module.functions[0];
        assert_eq!(
            symbol("Library", f).unwrap(),
            "neoclr_c_v1_m4c696272617279_n_f4563686f_ai32_ri32"
        );
        let mut other = f.clone();
        other.namespace = "A.B".into();
        assert_ne!(
            symbol("Library", f).unwrap(),
            symbol("Library", &other).unwrap()
        );
        other = f.clone();
        other.parameters[0] = Type::UInt32;
        assert_ne!(
            symbol("Library", f).unwrap(),
            symbol("Library", &other).unwrap()
        );
        assert_ne!(symbol("é", f).unwrap(), symbol("e\u{301}", f).unwrap());
        assert!(symbol("日本語", f).unwrap().is_ascii());
        let mut projected = f.clone();
        projected.origin = Some(
            serde_json::from_value(serde_json::json!({
                "assembly": "Library, Version=1.0.0.0", "module": "Library.dll",
                "name": "Echo", "token": 100663297
            }))
            .unwrap(),
        );
        projected.name = "NeoMetadata_old_hash.F_4563686f".into();
        let first = symbol("Library", &projected).unwrap();
        projected.name = "NeoMetadata_new_hash.F_4563686f".into();
        projected.origin.as_mut().unwrap().token += 10;
        assert_eq!(first, symbol("Library", &projected).unwrap());
        assert_eq!(first, symbol("Library", f).unwrap());
        assert_ne!(symbol("Library", f).unwrap(), symbol("Other", f).unwrap());
    }
}

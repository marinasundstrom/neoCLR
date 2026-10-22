//! Versioned C exports for scalar native libraries; private status ABI stays private.
use crate::{compiler, load_context, scalar_abi};
use cranelift_codegen::ir::{AbiParam, InstBuilder, StackSlotData, StackSlotKind, TrapCode, types};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{FuncId, Linkage, Module};
use neoclr::metadata::{AttributeArgument, Function, Type};
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
        Type::Named(name) => format!("s{}", hex(name)),
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

// Resolve source identity from loaded metadata, never from encoded executable names.
fn export_attribute_types<'a>(
    modules: impl Iterator<Item = &'a neoclr::Module>,
) -> HashSet<String> {
    const NAME: &str = "System.Runtime.InteropServices.NativeExportAttribute";
    let mut names = HashSet::from([NAME.to_owned()]);
    for module in modules {
        for ty in &module.types {
            if let Some(origin) = &ty.origin {
                if origin.name == NAME
                    && module
                        .assemblies
                        .iter()
                        .any(|a| a.full_name == origin.assembly && a.name == "System.Runtime")
                {
                    names.insert(ty.name.clone());
                }
            }
        }
    }
    names
}

fn annotation(f: &Function, attribute_types: &HashSet<String>) -> Result<Option<String>, Error> {
    let attributes: Vec<_> = f
        .custom_attributes
        .iter()
        .filter(|a| {
            a.constructor
                .owner
                .as_ref()
                .and_then(Type::definition_name)
                .is_some_and(|name| attribute_types.contains(name))
        })
        .collect();
    if attributes.is_empty() {
        return Ok(None);
    }
    let [attribute] = attributes.as_slice() else {
        return Err("duplicate NativeExportAttribute".into());
    };
    if attribute.target_token.is_some() || !attribute.named_arguments.is_empty() {
        return Err("NativeExportAttribute requires constructor arguments on the function".into());
    }
    let (name, convention) = match attribute.arguments.as_slice() {
        [] => ("", 0),
        [AttributeArgument::String(Some(name))] => (name.as_str(), 0),
        [AttributeArgument::String(Some(name)), AttributeArgument::Int32(convention)] => (name.as_str(), *convention),
        _ => return Err("NativeExportAttribute requires an optional entry-point string and NativeCallingConvention".into()),
    };
    if !matches!(convention, 0 | 1) || name.contains('\0') {
        return Err("unsupported native export name or calling convention".into());
    }
    // Cdecl and System are the same ABI on the admitted macOS ARM64 target.
    Ok(Some(name.to_owned()))
}

fn exported_name(
    input: &neoclr::Module,
    f: &Function,
    attribute_types: &HashSet<String>,
) -> Result<String, Error> {
    if let Some(name) = annotation(f, attribute_types)? {
        if !name.is_empty() {
            return Ok(name);
        }
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
    let layouts = crate::native_layout::Layouts::build(input)?;
    let mut signature = f.clone();
    for ty in signature
        .parameters
        .iter_mut()
        .chain([&mut signature.returns])
    {
        if let Type::Ptr(inner) = ty {
            if let Type::Named(name) = &**inner {
                *inner = Box::new(Type::Named(
                    layouts
                        .0
                        .get(name)
                        .ok_or("unresolved native struct export")?
                        .name
                        .clone(),
                ));
            }
        }
    }
    symbol(assembly, &signature)
}

fn validate_export(f: &Function) -> Result<(), Error> {
    if f.owner.is_some()
        || f.pinvoke.is_some()
        || f.parameters.len() > 16
        || f.parameters
            .iter()
            .any(|t| scalar_abi::scalar_type(t).is_none())
        || (!f.no_result && scalar_abi::scalar_type(&f.returns).is_none())
    {
        return Err(format!("unsupported native export: {}", f.name).into());
    }
    Ok(())
}

fn c_type(t: &Type, layouts: &crate::native_layout::Layouts) -> Result<String, Error> {
    Ok(match t {
        Type::Void => "void".into(),
        Type::Named(name) => layouts
            .0
            .get(name)
            .ok_or("unresolved native header struct")?
            .name
            .clone(),
        Type::Byte => "uint8_t".into(),
        Type::Int32 => "int32_t".into(),
        Type::UInt32 => "uint32_t".into(),
        Type::Int64 => "int64_t".into(),
        Type::UInt64 => "uint64_t".into(),
        Type::IntPtr => "intptr_t".into(),
        Type::UIntPtr => "uintptr_t".into(),
        Type::Single => "float".into(),
        Type::Double => "double".into(),
        Type::Ptr(inner) if scalar_abi::pointer_target(inner) => {
            format!("{} *", c_type(inner, layouts)?)
        }
        _ => return Err("unsupported C header type".into()),
    })
}

fn header(input: &neoclr::Module, exports: &[(usize, String)]) -> Result<String, Error> {
    let mut text = String::from(
        "/* Generated neoCLR C ABI v1 declarations for aarch64-apple-darwin.\n * Pointer validity, ownership and lifetime follow each function's contract.\n * Guest faults escaping an export terminate the process; no C++ unwinding.\n */\n#pragma once\n#include <stdint.h>\n#if !defined(__APPLE__) || !defined(__aarch64__)\n#error This neoCLR library targets macOS ARM64\n#endif\n#ifdef __cplusplus\nextern \"C\" {\n#endif\n",
    );
    let layouts = crate::native_layout::Layouts::build(input)?;
    text.push_str(&layouts.header()?);
    for (index, name) in exports {
        let keywords = "alignas alignof and and_eq asm auto bitand bitor bool break case catch char char8_t char16_t char32_t class compl concept const consteval constexpr constinit const_cast continue co_await co_return co_yield decltype default delete do double dynamic_cast else enum explicit export extern false float for friend goto if inline int long mutable namespace new noexcept not not_eq nullptr operator or or_eq private protected public register reinterpret_cast requires return short signed sizeof static static_assert static_cast struct switch template this thread_local throw true try typedef typeid typename union unsigned using virtual void volatile wchar_t while xor xor_eq _Alignas _Alignof _Atomic _Bool _Complex _Generic _Imaginary _Noreturn _Static_assert _Thread_local restrict";
        if name.is_empty()
            || !name
                .bytes()
                .enumerate()
                .all(|(i, b)| b.is_ascii_alphabetic() || b == b'_' || i > 0 && b.is_ascii_digit())
            || keywords.split_whitespace().any(|word| word == name)
        {
            return Err(format!(
                "export name cannot be declared in a portable C/C++ header: {name}"
            )
            .into());
        }
        let f = &input.functions[*index];
        let args = if f.parameters.is_empty() {
            "void".into()
        } else {
            f.parameters
                .iter()
                .enumerate()
                .map(|(i, t)| Ok(format!("{} arg{i}", c_type(t, &layouts)?)))
                .collect::<Result<Vec<_>, Error>>()?
                .join(", ")
        };
        let result = if f.no_result {
            "void".into()
        } else {
            c_type(&f.returns, &layouts)?
        };
        text.push_str(&format!("{result} {name}({args});\n"));
    }
    text.push_str("#ifdef __cplusplus\n}\n#endif\n");
    Ok(text)
}

fn write_outputs(
    object_path: &Path,
    object: &[u8],
    header: Option<(&Path, &str)>,
) -> Result<(), Error> {
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(object_path)?;
    let mut header_file = match header {
        Some((path, _)) => match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
        {
            Ok(file) => Some(file),
            Err(error) => {
                drop(output);
                let _ = fs::remove_file(object_path);
                return Err(error.into());
            }
        },
        None => None,
    };
    let result = (|| -> std::io::Result<()> {
        output.write_all(object)?;
        if let (Some(file), Some((_, text))) = (&mut header_file, header) {
            file.write_all(text.as_bytes())?;
        }
        Ok(())
    })();
    drop(output);
    drop(header_file);
    if let Err(error) = result {
        let _ = fs::remove_file(object_path);
        if let Some((path, _)) = header {
            let _ = fs::remove_file(path);
        }
        return Err(error.into());
    }
    Ok(())
}

pub(super) fn run(mut args: Vec<OsString>, imports: bool) -> Result<i32, Error> {
    let dependency_args = load_context::split(&mut args);
    if args.len() < 3 || args.len() > 133 || (args.len() - 3) % 2 != 0 {
        return Err("usage: --native-library <input> <output.o> --export <function> [--export <function> ...] [--header <output.h>] [--native-imports] [load-context pairs]".into());
    }
    let paths = load_context::parse(&dependency_args)?;
    let input = crate::read_module(Path::new(&args[1]))?;
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
        Some(neoclr::LoadedProgram::new(&input).map_err(|error| error.to_string())?)
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
    let attribute_types = export_attribute_types(
        std::iter::once(&input)
            .chain(system.iter())
            .chain(dependencies.iter()),
    );
    // Keep the loaded source program for verification; normalize layout decoder
    // tags only in the private emission inventory, using supplied source identity.
    let input = crate::native_layout::normalize_annotations(
        &input,
        std::iter::once(&input)
            .chain(system.iter())
            .chain(dependencies.iter()),
    )?;
    let mut selected = HashSet::new();
    let mut exports = Vec::new();
    for (index, f) in input.functions.iter().enumerate() {
        if annotation(f, &attribute_types)?.is_some() {
            validate_export(f)?;
            selected.insert(index);
            exports.push((index, exported_name(&input, f, &attribute_types)?));
        }
    }
    let mut header_path = None;
    for pair in args[3..].chunks_exact(2) {
        if pair[0] == "--header" {
            if header_path.replace(Path::new(&pair[1])).is_some() {
                return Err("duplicate --header option".into());
            }
            continue;
        }
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
        if !selected.insert(*index) {
            return Err(format!("duplicate native export selection: {name}").into());
        }
        validate_export(f)?;
        exports.push((*index, exported_name(&input, f, &attribute_types)?));
    }
    if exports.is_empty() || exports.len() > 64 {
        return Err(
            "native library requires 1..64 NativeExportAttribute or explicit --export selections"
                .into(),
        );
    }
    let mut symbols = HashSet::new();
    for (_, name) in &exports {
        if !symbols.insert(name) {
            return Err(format!("duplicate native export symbol: {name}").into());
        }
    }
    let generated_header = header_path.map(|_| header(&input, &exports)).transpose()?;
    let object =
        compiler::compile_native_library(&input, &exports, imports, verification.as_ref())?;
    write_outputs(
        Path::new(&args[2]),
        &object,
        header_path.zip(generated_header.as_deref()),
    )?;
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
    fn annotation_arguments_and_header_contract() {
        let mut input = neoclr::assemble(
            ".module Library\n.function Echo(Int32 value) -> Int32\nldarg value\nret\n.end",
        )
        .unwrap();
        let names = export_attribute_types(std::iter::once(&input));
        let attribute = |arguments| neoclr::metadata::CustomAttribute {
            constructor: neoclr::metadata::FunctionRef {
                definition: None,
                name: "System.Runtime.InteropServices.NativeExportAttribute..ctor".into(),
                owner: Some(Type::Named(
                    "System.Runtime.InteropServices.NativeExportAttribute".into(),
                )),
                instance: true,
                generic_arguments: vec![],
                parameters: vec![],
            },
            arguments,
            named_arguments: vec![],
            target_token: None,
        };
        input.functions[0].custom_attributes.push(attribute(vec![]));
        assert_eq!(
            annotation(&input.functions[0], &names).unwrap(),
            Some(String::new())
        );
        input.functions[0].custom_attributes[0] = attribute(vec![
            AttributeArgument::String(Some("echo".into())),
            AttributeArgument::Int32(1),
        ]);
        assert_eq!(
            exported_name(&input, &input.functions[0], &names).unwrap(),
            "echo"
        );
        let text = header(&input, &[(0, "echo".into())]).unwrap();
        assert!(text.contains("int32_t echo(int32_t arg0);"));
        for name in ["", "int", "class", "bad-name", "1echo", "écho"] {
            assert!(header(&input, &[(0, name.into())]).is_err(), "{name}");
        }
        input.functions[0].custom_attributes[0] = attribute(vec![
            AttributeArgument::String(Some("echo".into())),
            AttributeArgument::Int32(2),
        ]);
        assert!(annotation(&input.functions[0], &names).is_err());
        input.functions[0].custom_attributes[0] = attribute(vec![AttributeArgument::String(None)]);
        assert!(annotation(&input.functions[0], &names).is_err());
        input.functions[0].custom_attributes = vec![attribute(vec![]), attribute(vec![])];
        assert!(annotation(&input.functions[0], &names).is_err());
    }

    #[test]
    fn output_failures_preserve_existing_files() {
        let dir = std::env::temp_dir().join(format!(
            "neoclr-export-output-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&dir).unwrap();
        let object = dir.join("exports.o");
        let header = dir.join("exports.h");
        fs::write(&header, "existing header").unwrap();
        assert!(write_outputs(&object, b"object", Some((&header, "new header"))).is_err());
        assert!(!object.exists());
        assert_eq!(fs::read_to_string(&header).unwrap(), "existing header");
        assert!(write_outputs(&object, b"object", Some((&object, "new header"))).is_err());
        assert!(!object.exists());
        fs::write(&object, "existing object").unwrap();
        assert!(write_outputs(&object, b"new object", None).is_err());
        assert_eq!(fs::read_to_string(&object).unwrap(), "existing object");
        fs::remove_dir_all(dir).unwrap();
    }

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

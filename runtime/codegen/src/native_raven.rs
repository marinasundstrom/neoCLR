//! Raven declarations for the same checked native surface as the C header.
use crate::native_layout::Layouts;
use neoclr::{Module, metadata::Type};
type Error = Box<dyn std::error::Error>;

fn field_type(c: &str) -> Result<String, Error> {
    if let Some(inner) = c.strip_suffix(" *") {
        return Ok(format!("*{}", field_type(inner)?));
    }
    Ok(match c {
        "void" => "unit",
        "uint8_t" => "byte",
        "int8_t" => "sbyte",
        "int16_t" => "short",
        "uint16_t" => "ushort",
        "int32_t" => "int",
        "uint32_t" => "uint",
        "int64_t" => "long",
        "uint64_t" => "ulong",
        "intptr_t" => "nint",
        "uintptr_t" => "nuint",
        "float" => "float",
        "double" => "double",
        name if name.starts_with("neoclr_struct_") => name,
        _ => return Err("unsupported Raven import field representation".into()),
    }
    .into())
}
fn signature_type(ty: &Type, layouts: &Layouts) -> Result<String, Error> {
    Ok(match ty {
        Type::Ptr(inner) => format!("*{}", signature_type(inner, layouts)?),
        Type::Named(name) => layouts
            .0
            .get(name)
            .ok_or("unresolved Raven import struct")?
            .name
            .clone(),
        Type::Void => "unit".into(),
        Type::Byte => "byte".into(),
        Type::Int32 => "int".into(),
        Type::UInt32 => "uint".into(),
        Type::Int64 => "long".into(),
        Type::UInt64 => "ulong".into(),
        Type::IntPtr => "nint".into(),
        Type::UIntPtr => "nuint".into(),
        Type::Single => "float".into(),
        Type::Double => "double".into(),
        _ => return Err("unsupported Raven import signature".into()),
    })
}
pub(crate) fn generate(
    input: &Module,
    exports: &[(usize, String)],
    library: &str,
) -> Result<String, Error> {
    if library.is_empty() || library.contains('\0') {
        return Err("import library must be nonempty and contain no NUL".into());
    }
    let layouts = Layouts::build(input)?;
    let mut text = String::from(
        "// Generated C ABI imports for macOS ARM64. Caller owns native pointer lifetimes.\n// These structs describe native storage, not managed marshalling.\nimport System.Runtime.InteropServices.*\n\n",
    );
    for layout in layouts.0.values() {
        text.push_str(&format!(
            "[StructLayout(LayoutKind.Explicit, Pack: {}, Size: {})]\npublic struct {} {{\n",
            layout.pack, layout.size, layout.name
        ));
        for field in &layout.fields {
            text.push_str(&format!(
                "    [FieldOffset({})]\n    field {}: {}\n",
                field.offset,
                field.name,
                field_type(&field.c_type)?
            ));
        }
        text.push_str("}\n\n");
    }
    for (ordinal, (index, symbol)) in exports.iter().enumerate() {
        let f = &input.functions[*index];
        let parameters = f
            .parameters
            .iter()
            .enumerate()
            .map(|(i, ty)| Ok(format!("arg{i}: {}", signature_type(ty, &layouts)?)))
            .collect::<Result<Vec<_>, Error>>()?
            .join(", ");
        text.push_str(&format!("[NativeImport({}, EntryPoint: {})]\nunsafe extern func NativeExport{ordinal}({parameters}) -> {};\n\n", serde_json::to_string(library)?, serde_json::to_string(symbol)?, if f.no_result { "unit".into() } else { signature_type(&f.returns, &layouts)? }));
    }
    Ok(text)
}

//! Native data layout, separate from managed storage and aggregate calling ABI.
//! Currently qualified for the LP64 macOS ARM64 C target.
use neoclr::{
    Module,
    metadata::{AttributeArgument, CustomAttribute, Type, TypeDef},
};
use std::collections::{BTreeMap, HashSet};
type Error = Box<dyn std::error::Error>;

#[derive(Clone, Debug)]
pub(crate) struct Field {
    pub name: String,
    pub c_type: String,
    pub offset: usize,
    pub size: usize,
}
#[derive(Clone, Debug)]
pub(crate) struct Layout {
    pub name: String,
    pub size: usize,
    pub align: usize,
    pub pack: usize,
    pub explicit: bool,
    pub fields: Vec<Field>,
}
pub(crate) struct Layouts(pub BTreeMap<String, Layout>);

/// Normalize only the three layout annotations in a private emission copy.
/// Source verification must continue to use the original loaded program: these
/// canonical names are decoder tags, not replacement executable type identities.
pub(crate) fn normalize_annotations<'a>(
    input: &Module,
    modules: impl Iterator<Item = &'a Module>,
) -> Result<Module, Error> {
    const NAMES: [&str; 3] = [
        "System.Runtime.InteropServices.StructLayoutAttribute",
        "System.Runtime.InteropServices.FieldOffsetAttribute",
        "System.Runtime.InteropServices.MarshalAsAttribute",
    ];
    let mut identities = BTreeMap::new();
    for module in modules {
        for ty in &module.types {
            let Some(origin) = &ty.origin else { continue };
            if !NAMES.contains(&origin.name.as_str())
                || !module.assemblies.iter().any(|assembly| {
                    assembly.full_name == origin.assembly && assembly.name == "System.Runtime"
                })
            {
                continue;
            }
            if identities
                .insert(ty.name.as_str(), origin.name.as_str())
                .is_some_and(|previous| previous != origin.name)
            {
                return Err("ambiguous native layout attribute identity".into());
            }
        }
    }
    let mut projected = input.clone();
    for ty in &mut projected.types {
        for attribute in &mut ty.custom_attributes {
            if let Some(canonical) = attribute_name(attribute).and_then(|name| identities.get(name))
            {
                attribute.constructor.owner = Some(Type::Named((*canonical).to_owned()));
            }
        }
    }
    Ok(projected)
}

fn identifier(name: &str) -> String {
    // Encode rather than guess whether a source identifier is a C keyword.
    name.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
fn native_name(module: &Module, ty: &TypeDef) -> String {
    let (assembly, name) = if let Some(origin) = &ty.origin {
        (
            module
                .assemblies
                .iter()
                .find(|a| a.full_name == origin.assembly)
                .map(|a| a.name.as_str())
                .unwrap_or(&origin.assembly),
            origin.name.as_str(),
        )
    } else {
        (module.name.as_str(), ty.name.as_str())
    };
    format!(
        "neoclr_struct_{}",
        identifier(&format!("{assembly}:{name}"))
    )
}
fn align(value: usize, alignment: usize) -> Result<usize, Error> {
    value
        .checked_add(alignment - 1)
        .map(|v| v & !(alignment - 1))
        .filter(|v| *v <= 1024 * 1024)
        .ok_or_else(|| "native struct size exceeds 1 MiB".into())
}
fn attribute_name(attribute: &CustomAttribute) -> Option<&str> {
    attribute
        .constructor
        .owner
        .as_ref()
        .and_then(Type::definition_name)
}
fn integer(attribute: &CustomAttribute) -> Result<usize, Error> {
    match attribute.arguments.as_slice() {
        [AttributeArgument::Int32(n)] if *n >= 0 => Ok(*n as usize),
        _ => Err("native layout attribute requires one nonnegative Int32 argument".into()),
    }
}
fn scalar(ty: &Type, mapping: Option<usize>) -> Result<(&'static str, usize), Error> {
    if let Some(mapping) = mapping {
        return match (ty, mapping) {
            (Type::Boolean, 2) => Ok(("uint8_t", 1)),
            (Type::Boolean, 3) => Ok(("int32_t", 4)),
            _ => Err("unsupported or incompatible native struct field MarshalAs".into()),
        };
    }
    Ok(match ty {
        Type::Boolean => ("int32_t", 4), // .NET default unmanaged BOOL representation.
        Type::Byte => ("uint8_t", 1),
        Type::SByte => ("int8_t", 1),
        Type::Int16 => ("int16_t", 2),
        Type::UInt16 => ("uint16_t", 2),
        Type::Int32 => ("int32_t", 4),
        Type::UInt32 => ("uint32_t", 4),
        Type::Int64 => ("int64_t", 8),
        Type::UInt64 => ("uint64_t", 8),
        Type::IntPtr => ("intptr_t", 8),
        Type::UIntPtr => ("uintptr_t", 8),
        Type::Single => ("float", 4),
        Type::Double => ("double", 8),
        _ => {
            return Err(
                "native struct field requires a supported numeric or pointer representation".into(),
            );
        }
    })
}

impl Layouts {
    pub fn build(module: &Module) -> Result<Self, Error> {
        if module.types.len() > 128 {
            return Err("native struct inventory exceeds 128 types".into());
        }
        let mut result = Self(BTreeMap::new());
        let mut visiting = HashSet::new();
        for ty in &module.types {
            result.add(module, ty, &mut visiting)?;
        }
        let mut identities = HashSet::new();
        if result
            .0
            .values()
            .any(|layout| !identities.insert(&layout.name))
        {
            return Err("duplicate native struct source identity".into());
        }
        Ok(result)
    }
    fn add(
        &mut self,
        module: &Module,
        ty: &TypeDef,
        visiting: &mut HashSet<String>,
    ) -> Result<(), Error> {
        if self.0.contains_key(&ty.name) {
            return Ok(());
        }
        if !visiting.insert(ty.name.clone()) {
            return Err("recursive by-value native struct layout".into());
        }
        if ty.is_reference_type
            || ty.memory_view.is_some()
            || ty.enum_info.is_some()
            || !ty.generic_parameters.is_empty()
            || ty.fields.len() > 256
            || ty.representation != neoclr::metadata::Representation::Record
        {
            return Err(
                "native layout requires a closed value struct without managed storage".into(),
            );
        }
        let mut layout = Layout {
            name: native_name(module, ty),
            size: 0,
            align: 1,
            pack: ty.packing.unwrap_or(0) as usize,
            explicit: false,
            fields: vec![],
        };
        let mut minimum = ty.minimum_size.unwrap_or(0) as usize;
        let mut seen_layout = false;
        for a in ty
            .custom_attributes
            .iter()
            .filter(|a| a.target_token.is_none())
        {
            if attribute_name(a) != Some("System.Runtime.InteropServices.StructLayoutAttribute") {
                return Err("unresolved native struct layout annotation".into());
            }
            if seen_layout {
                return Err("duplicate StructLayout".into());
            }
            seen_layout = true;
            layout.explicit = match integer(a)? {
                0 => false,
                2 => true,
                3 => return Err("Auto layout cannot cross a native boundary".into()),
                _ => return Err("invalid StructLayout kind".into()),
            };
            let mut names = HashSet::new();
            for named in &a.named_arguments {
                if !names.insert(&named.name) {
                    return Err("duplicate StructLayout option".into());
                }
                let AttributeArgument::Int32(value) = named.value else {
                    return Err("invalid StructLayout option".into());
                };
                if value < 0 {
                    return Err("negative StructLayout option".into());
                }
                match named.name.as_str() {
                    "Pack" if ty.packing.is_none() => layout.pack = value as usize,
                    "Size" if ty.minimum_size.is_none() => minimum = value as usize,
                    _ => return Err("unsupported or conflicting StructLayout option".into()),
                }
            }
        }
        if layout.pack == 0 {
            layout.pack = 8;
        }
        if !layout.pack.is_power_of_two() || layout.pack > 128 {
            return Err("invalid native struct packing".into());
        }
        let mut field_tokens = HashSet::new();
        let mut field_names = HashSet::new();
        for (i, field) in ty.fields.iter().enumerate() {
            if !field_names.insert(&field.name) {
                return Err("duplicate native field name".into());
            }
            let token = ty
                .origin
                .as_ref()
                .and_then(|o| o.field_tokens.get(i))
                .copied();
            let mut offset = None;
            let mut mapping = None;
            for a in ty
                .custom_attributes
                .iter()
                .filter(|a| a.target_token.is_some() && a.target_token == token)
            {
                field_tokens.insert(a.target_token.unwrap());
                if !a.named_arguments.is_empty() {
                    return Err("unsupported field marshalling options".into());
                }
                let destination = match attribute_name(a) {
                    Some("System.Runtime.InteropServices.FieldOffsetAttribute") => &mut offset,
                    Some("System.Runtime.InteropServices.MarshalAsAttribute") => &mut mapping,
                    _ => return Err("unresolved native field layout annotation".into()),
                };
                if destination.replace(integer(a)?).is_some() {
                    return Err("duplicate field layout annotation".into());
                }
            }
            let (c_type, size, alignment) = match &field.ty {
                Type::Named(name) if mapping.is_none() => {
                    let nested = module
                        .types
                        .iter()
                        .find(|t| t.name == *name)
                        .ok_or("unresolved nested native struct")?;
                    self.add(module, nested, visiting)?;
                    let nested = &self.0[name];
                    (nested.name.clone(), nested.size, nested.align)
                }
                Type::Ptr(inner) if mapping.is_none() => {
                    let c = if **inner == Type::Void {
                        "void".into()
                    } else if let Type::Named(name) = &**inner {
                        let nested = module
                            .types
                            .iter()
                            .find(|t| t.name == *name)
                            .ok_or("unresolved native struct pointer")?;
                        if nested.is_reference_type {
                            return Err(
                                "native struct pointer cannot expose a managed class".into()
                            );
                        }
                        native_name(module, nested)
                    } else {
                        if **inner == Type::Boolean {
                            return Err(
                                "Boolean pointer fields require an explicit pointee contract"
                                    .into(),
                            );
                        }
                        scalar(inner, None)?.0.into()
                    };
                    (format!("{c} *"), 8, 8)
                }
                _ => {
                    let (c, size) = scalar(&field.ty, mapping)?;
                    (c.into(), size, size)
                }
            };
            let alignment = alignment.min(layout.pack);
            layout.align = layout.align.max(alignment);
            let offset = if layout.explicit {
                offset.ok_or("Explicit layout requires every FieldOffset")?
            } else {
                if offset.is_some() {
                    return Err("FieldOffset requires Explicit layout".into());
                }
                align(layout.size, alignment)?
            };
            layout.size = layout.size.max(
                offset
                    .checked_add(size)
                    .ok_or("native field offset overflow")?,
            );
            layout.fields.push(Field {
                name: format!("field_{}", identifier(&field.name)),
                c_type,
                offset,
                size,
            });
        }
        if ty
            .custom_attributes
            .iter()
            .filter_map(|a| a.target_token)
            .any(|t| !field_tokens.contains(&t))
        {
            return Err("native layout annotation has no matching instance field".into());
        }
        layout.size = align(layout.size.max(minimum).max(1), layout.align)?;
        visiting.remove(&ty.name);
        self.0.insert(ty.name.clone(), layout);
        Ok(())
    }
}

impl Layouts {
    pub fn check_pointer(&self, ty: &Type) -> Result<(), Error> {
        if let Type::Ptr(inner) = ty {
            if let Type::Named(name) = &**inner {
                if !self.0.contains_key(name) {
                    return Err("unresolved native struct pointer layout".into());
                }
            }
        }
        Ok(())
    }
    pub fn header(&self) -> Result<String, Error> {
        let mut text = String::from(
            "#include <stddef.h>\n#ifdef __cplusplus\n#define NEOCLR_LAYOUT_ALIGN(n) alignas(n)\n#define NEOCLR_LAYOUT_ASSERT(c) static_assert(c, #c)\n#define NEOCLR_LAYOUT_ALIGNOF(t) alignof(t)\n#else\n#define NEOCLR_LAYOUT_ALIGN(n) _Alignas(n)\n#define NEOCLR_LAYOUT_ASSERT(c) _Static_assert(c, #c)\n#define NEOCLR_LAYOUT_ALIGNOF(t) _Alignof(t)\n#endif\n",
        );
        for layout in self.0.values() {
            let kind = if layout.explicit { "union" } else { "struct" };
            text.push_str(&format!("typedef {kind} {0} {0};\n", layout.name));
        }
        let mut emitted = HashSet::new();
        fn emit(
            layout: &Layout,
            layouts: &Layouts,
            emitted: &mut HashSet<String>,
            text: &mut String,
        ) {
            if !emitted.insert(layout.name.clone()) {
                return;
            }
            for field in &layout.fields {
                if let Some(nested) = layouts.0.values().find(|l| l.name == field.c_type) {
                    emit(nested, layouts, emitted, text);
                }
            }
            if layout.explicit {
                // Packed named views preserve explicit offsets, including overlap,
                // without depending on anonymous-struct compiler extensions.
                for field in &layout.fields {
                    text.push_str(&format!(
                        "#pragma pack(push, 1)\nstruct {}_{} {{\n",
                        layout.name, field.name
                    ));
                    if field.offset != 0 {
                        text.push_str(&format!("uint8_t padding[{}];\n", field.offset));
                    }
                    text.push_str(&format!(
                        "{} value;\n}};\n#pragma pack(pop)\n",
                        field.c_type
                    ));
                }
                text.push_str(&format!(
                    "union {} {{\nNEOCLR_LAYOUT_ALIGN({}) uint8_t storage[{}];\n",
                    layout.name, layout.align, layout.size
                ));
                for field in &layout.fields {
                    text.push_str(&format!(
                        "struct {}_{} {};\n",
                        layout.name, field.name, field.name
                    ));
                }
                text.push_str("};\n");
            } else {
                // The admitted LP64 field set has natural alignment at most eight.
                // Larger requested packs are equivalent and need no nonportable pragma.
                text.push_str(&format!(
                    "#pragma pack(push, {})\nstruct {} {{\n",
                    layout.pack.min(8),
                    layout.name
                ));
                for field in &layout.fields {
                    text.push_str(&format!("{} {};\n", field.c_type, field.name));
                }
                let end = layout.fields.last().map(|f| f.offset + f.size).unwrap_or(0);
                if end < layout.size {
                    text.push_str(&format!(
                        "uint8_t trailing_padding[{}];\n",
                        layout.size - end
                    ));
                }
                text.push_str("};\n#pragma pack(pop)\n");
            }
            text.push_str(&format!("NEOCLR_LAYOUT_ASSERT(sizeof({0}) == {1});\nNEOCLR_LAYOUT_ASSERT(NEOCLR_LAYOUT_ALIGNOF({0}) == {2});\n", layout.name, layout.size, layout.align));
            for field in &layout.fields {
                text.push_str(&format!(
                    "NEOCLR_LAYOUT_ASSERT(offsetof({}, {}{}) == {});\n",
                    layout.name,
                    field.name,
                    if layout.explicit { ".value" } else { "" },
                    field.offset
                ));
            }
        }
        for layout in self.0.values() {
            emit(layout, self, &mut emitted, &mut text);
        }
        text.push_str("#undef NEOCLR_LAYOUT_ALIGN\n#undef NEOCLR_LAYOUT_ASSERT\n#undef NEOCLR_LAYOUT_ALIGNOF\n");
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use neoclr::metadata::{CustomAttributeNamedArgument, FunctionRef};
    fn module() -> Module {
        neoclr::assemble(
            ".module Native\n.type Pair\n.field Flag Boolean\n.field Value Int32\n.end\n",
        )
        .unwrap()
    }
    fn annotation(name: &str, value: i32, token: Option<u32>) -> CustomAttribute {
        let name = format!("System.Runtime.InteropServices.{name}Attribute");
        CustomAttribute {
            constructor: FunctionRef {
                definition: None,
                name: format!("{name}..ctor"),
                owner: Some(Type::Named(name)),
                instance: true,
                generic_arguments: vec![],
                parameters: vec![],
            },
            arguments: vec![AttributeArgument::Int32(value)],
            named_arguments: vec![],
            target_token: token,
        }
    }
    fn origin(module: &mut Module) {
        module.types[0].origin = Some(
            serde_json::from_value(serde_json::json!({
                "assembly": "Native", "module": "Native.dll", "name": "Pair", "token": 33554433,
                "field_tokens": [67108865, 67108866]
            }))
            .unwrap(),
        );
    }
    #[test]
    fn supported_packing_values_generate_portable_target_headers() {
        for pack in [0, 1, 2, 4, 8, 16, 32, 64, 128] {
            let mut input = module();
            input.types[0].packing = Some(pack);
            let layout = Layouts::build(&input).unwrap();
            compile_header(&layout);
        }
    }

    #[test]
    fn loaded_attribute_identity_controls_field_marshalling() {
        let mut input = module();
        origin(&mut input);
        input.types[0].packing = Some(1);
        let mut attribute = annotation("MarshalAs", 2, Some(67108865));
        attribute.constructor.owner = Some(Type::Named("Physical.Attribute".into()));
        input.types[0].custom_attributes.push(attribute);
        assert!(Layouts::build(&input).is_err());
        let mut dependency = module();
        dependency.types[0].name = "Physical.Attribute".into();
        origin(&mut dependency);
        let source = dependency.types[0].origin.as_mut().unwrap();
        source.name = "System.Runtime.InteropServices.MarshalAsAttribute".into();
        source.assembly = "System.Runtime, Version=1.0.0.0".into();
        dependency.assemblies = vec![
            serde_json::from_value(serde_json::json!({
                "name": "System.Runtime", "full_name": "System.Runtime, Version=1.0.0.0",
                "modules": [], "references": []
            }))
            .unwrap(),
        ];
        let projected = normalize_annotations(&input, std::iter::once(&dependency)).unwrap();
        let layouts = Layouts::build(&projected).unwrap();
        assert_eq!(layouts.0["Pair"].size, 5);
        assert_eq!(layouts.0["Pair"].fields[1].offset, 1);
        assert_eq!(
            attribute_name(&input.types[0].custom_attributes[0]),
            Some("Physical.Attribute")
        );
        dependency.assemblies[0].name = "Unrelated".into();
        let unrelated = normalize_annotations(&input, std::iter::once(&dependency)).unwrap();
        assert!(Layouts::build(&unrelated).is_err());
        // A canonical-looking source name without a matching assembly scope is
        // not enough to authorize a physical attribute owner.
        dependency.assemblies[0].name = "System.Runtime".into();
        dependency.assemblies[0].full_name = "OtherRevision".into();
        assert!(
            Layouts::build(&normalize_annotations(&input, std::iter::once(&dependency)).unwrap())
                .is_err()
        );
    }

    #[test]
    fn defaults_overrides_packing_and_explicit_overlap() {
        let mut m = module();
        let default = Layouts::build(&m).unwrap();
        let l = &default.0["Pair"];
        assert_eq!((l.size, l.align, l.fields[1].offset), (8, 4, 4));
        origin(&mut m);
        m.types[0].packing = Some(1);
        m.types[0]
            .custom_attributes
            .push(annotation("MarshalAs", 2, Some(67108865)));
        let packed = Layouts::build(&m).unwrap();
        let l = &packed.0["Pair"];
        assert_eq!((l.size, l.align, l.fields[1].offset), (5, 1, 1));
        m.types[0].packing = None;
        let mut explicit = annotation("StructLayout", 2, None);
        explicit.named_arguments.push(CustomAttributeNamedArgument {
            name: "Size".into(),
            is_field: false,
            value: AttributeArgument::Int32(16),
        });
        m.types[0].custom_attributes.extend([
            explicit,
            annotation("FieldOffset", 1, Some(67108865)),
            annotation("FieldOffset", 1, Some(67108866)),
        ]);
        let overlapping = Layouts::build(&m).unwrap();
        let l = &overlapping.0["Pair"];
        assert_eq!(
            (l.size, l.align, l.fields[0].offset, l.fields[1].offset),
            (16, 4, 1, 1)
        );
        for layout in [default, packed, overlapping] {
            compile_header(&layout);
        }
    }
    fn compile_header(layouts: &Layouts) {
        if !cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            return;
        }
        use std::{fs, process::Command};
        let directory = std::env::temp_dir().join(format!(
            "neoclr-layout-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&directory).unwrap();
        let source = directory.join("layout.c");
        fs::write(
            &source,
            format!(
                "#include <stdint.h>\n{}\nint main(void) {{return 0;}}",
                layouts.header().unwrap()
            ),
        )
        .unwrap();
        for language in ["c", "c++"] {
            let output = Command::new("xcrun")
                .args([
                    "--sdk",
                    "macosx",
                    "clang",
                    "-x",
                    language,
                    if language == "c" {
                        "-std=c11"
                    } else {
                        "-std=c++17"
                    },
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                    "-fsyntax-only",
                ])
                .arg(&source)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn nested_structs_and_self_pointers_have_checked_headers() {
        let m = neoclr::assemble(".module Native\n.type Inner\n.field Number Double\n.end\n.type Outer\n.field Prefix Byte\n.field Child Inner\n.field Next Ptr<Outer>\n.end\n").unwrap();
        let layouts = Layouts::build(&m).unwrap();
        let outer = &layouts.0["Outer"];
        assert_eq!((outer.size, outer.align), (24, 8));
        assert_eq!((outer.fields[1].offset, outer.fields[2].offset), (8, 16));
        compile_header(&layouts);
    }
    #[test]
    fn invalid_layouts_and_managed_fields_reject() {
        for pack in [3, 256] {
            let mut m = module();
            m.types[0].packing = Some(pack);
            assert!(Layouts::build(&m).is_err());
        }
        for kind in [2, 3, 99] {
            let mut m = module();
            m.types[0]
                .custom_attributes
                .push(annotation("StructLayout", kind, None));
            assert!(Layouts::build(&m).is_err());
        }
        let mut m = module();
        m.types[0].fields[0].ty = Type::String;
        assert!(Layouts::build(&m).is_err());
        let mut m = module();
        m.types[0].is_reference_type = true;
        assert!(Layouts::build(&m).is_err());
        let mut m = module();
        origin(&mut m);
        m.types[0]
            .custom_attributes
            .push(annotation("MarshalAs", 2, Some(67108866)));
        assert!(Layouts::build(&m).is_err());
        let m = neoclr::assemble(".module Native\n.type Loop\n.field Child Loop\n.end\n").unwrap();
        assert!(Layouts::build(&m).is_err());
    }
}

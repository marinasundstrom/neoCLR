//! Enum metadata retains nominal identity over an ordinary one-integer value layout.
use crate::{
    Fault,
    metadata::{EnumInfo, Representation, Type, TypeDef, Visibility},
};

pub(crate) fn validate(def: &TypeDef) -> Result<(), Fault> {
    let Some(info) = &def.enum_info else {
        return Ok(());
    };
    if info.underlying != Type::Int32
        || def.representation != Representation::Record
        || def.packing.is_some()
        || def.minimum_size.is_some()
        || def.is_abstract
        || def.base.is_some()
        || !def.generic_parameters.is_empty()
        || !def.implements.is_empty()
        || def.fields.len() != 1
        || def.fields[0].ty != Type::Int32
        || def.fields[0].visibility != Visibility::Private
    {
        return Err(Fault::new(
            "enum requires a concrete nongeneric one-Int32 private-field layout without a base or interfaces",
        ));
    }
    let mut names = std::collections::HashSet::new();
    for member in &info.members {
        if member.name.is_empty()
            || !member
                .name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
            || member.name.chars().next().unwrap().is_ascii_digit()
            || !names.insert(&member.name)
        {
            return Err(Fault::new("invalid or duplicate enum member name"));
        }
    }
    Ok(())
}

pub(crate) fn emit(name: &str, info: &EnumInfo) -> String {
    let mut il = format!(
        ".type {name}\n.enum Int32{}\n.field private Bits Int32\n",
        if info.flags { " flags" } else { "" }
    );
    for member in &info.members {
        il.push_str(&format!(".literal {} {}\n", member.name, member.value));
    }
    il.push_str(&format!(".method static FromValue(Int32 value) -> {name}\nldarg value\nnewobj {name}\nret\n.end\n.method instance get_Value() -> Int32\nldarg this\nldfld 0\nret\n.end\n"));
    for (method, opcode) in [("Or", "or"), ("And", "and"), ("Xor", "xor")] {
        il.push_str(&format!(".method instance {method}({name} other) -> {name}\nldarg this\nldfld 0\nldarg other\nldfld 0\n{opcode}\nnewobj {name}\nret\n.end\n"));
    }
    il.push_str(&format!(".method instance Not() -> {name}\nldarg this\nldfld 0\nnot\nnewobj {name}\nret\n.end\n.method instance HasFlag({name} other) -> Boolean\nldarg this\nldfld 0\nldarg other\nldfld 0\nand\nldarg other\nldfld 0\nceq\nret\n.end\n.method instance Equals({name} other) -> Boolean\nldarg this\nldfld 0\nldarg other\nldfld 0\nceq\nret\n.end\n"));
    for member in &info.members {
        il.push_str(&format!(
            ".method static {}() -> {name}\nldc.i4 {}\nnewobj {name}\nret\n.end\n",
            member.name, member.value
        ));
    }
    il.push_str(".end\n");
    il
}

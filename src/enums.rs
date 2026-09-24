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

/// Shared unsigned-value ordering for names and values. Stable sorting preserves
/// metadata order for aliases, without promising .NET's unspecified alias choice.
pub(crate) fn members(info: &EnumInfo) -> Vec<&crate::metadata::EnumMember> {
    let mut result = info.members.iter().collect::<Vec<_>>();
    result.sort_by_key(|member| member.value as u32);
    result
}

pub(crate) fn format(info: &EnumInfo, value: i32) -> String {
    if let Some(member) = info.members.iter().find(|member| member.value == value) {
        return member.name.clone();
    }
    if info.flags && value != 0 {
        let mut remaining = value as u32;
        let mut names = Vec::new();
        // Prefer the first declared alias for a given mask.
        let mut unique = members(info);
        unique.dedup_by_key(|member| member.value);
        for member in unique.into_iter().rev() {
            let bits = member.value as u32;
            if bits != 0 && remaining & bits == bits {
                remaining &= !bits;
                names.push(member.name.as_str());
            }
        }
        if remaining == 0 {
            names.reverse();
            return names.join(", ");
        }
    }
    value.to_string()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::EnumMember;

    #[test]
    fn names_values_and_formatting_share_unsigned_metadata_order() {
        let info = EnumInfo {
            underlying: Type::Int32,
            flags: true,
            members: [("Negative", -1), ("Two", 2), ("One", 1), ("Alias", 1)]
                .into_iter()
                .map(|(name, value)| EnumMember {
                    name: name.into(),
                    value,
                })
                .collect(),
        };
        assert_eq!(
            members(&info)
                .iter()
                .map(|m| m.name.as_str())
                .collect::<Vec<_>>(),
            ["One", "Alias", "Two", "Negative"]
        );
        assert_eq!(format(&info, 1), "One");
        assert_eq!(format(&info, 3), "One, Two");
        assert_eq!(format(&info, -1), "Negative");
        assert_eq!(format(&info, 0), "0");
        assert_eq!(format(&info, 5), "5");
        assert_eq!(
            format(
                &EnumInfo {
                    flags: false,
                    ..info
                },
                3
            ),
            "3"
        );
    }
}

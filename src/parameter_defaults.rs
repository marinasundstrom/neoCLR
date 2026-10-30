//! Optional-argument metadata is a caller contract, never a runtime arity change.
use crate::{
    Fault, Module,
    metadata::{Function, Type},
};

pub(crate) fn validate(module: &Module, function: &Function) -> Result<(), Fault> {
    let defaults = &function.parameter_defaults;
    if defaults.is_empty() {
        return Ok(());
    }
    let parameters = &function.parameters;
    if defaults.len() != parameters.len() {
        return Err(Fault::new(
            "parameter defaults must align with declared parameters",
        ));
    }
    for (default, ty) in defaults.iter().zip(parameters) {
        let Some(default) = default else { continue };
        let text = default.value.as_str();
        macro_rules! number {
            ($t:ty) => {
                text.parse::<$t>()
                    .is_ok_and(|value| value.to_string() == text)
            };
        }
        let valid = match (default.kind.as_str(), ty) {
            ("Null", Type::String | Type::Array(_) | Type::Function(_)) => text.is_empty(),
            ("Null", _) => text.is_empty() && module.is_reference_type(ty),
            ("Boolean", Type::Boolean) => matches!(text, "true" | "false"),
            ("String", Type::String) => true,
            ("SByte", Type::SByte) => number!(i8),
            ("Byte", Type::Byte) => number!(u8),
            ("Int16", Type::Int16) => number!(i16),
            ("UInt16", Type::UInt16) => number!(u16),
            ("Int32", Type::Int32) => number!(i32),
            ("UInt32", Type::UInt32) => number!(u32),
            ("Int64", Type::Int64) => number!(i64),
            ("UInt64", Type::UInt64) => number!(u64),
            ("Single", Type::Single) => {
                text.len() == 8
                    && u32::from_str_radix(text, 16)
                        .is_ok_and(|value| format!("{value:08x}") == text)
            }
            ("Double", Type::Double) => {
                text.len() == 16
                    && u64::from_str_radix(text, 16)
                        .is_ok_and(|value| format!("{value:016x}") == text)
            }
            _ => false,
        };
        if !valid {
            return Err(Fault::new("invalid or mismatched parameter default"));
        }
    }
    Ok(())
}

//! Typed private service boundary. Guest references never cross the provider ABI.
use crate::{Fault, Value, metadata::Type};
use neoclr_security_provider as provider;

pub(crate) enum Operation {
    Sha256,
    HmacSha256,
    VerifyHmacSha256,
    RandomFill,
}

pub(crate) fn view_type(readonly: bool) -> Type {
    Type::Constructed {
        definition: if readonly {
            "System.ReadOnlySlice"
        } else {
            "System.Slice"
        }
        .into(),
        arguments: vec![Type::Byte],
    }
}
fn view(value: &Value, readonly: bool) -> Result<&crate::MemoryView, Fault> {
    let Value::MemoryView(view) = value else {
        return Err(Fault::new("cryptography requires a byte slice"));
    };
    if view.ty() != view_type(readonly) {
        return Err(Fault::new(
            "cryptography requires the matching byte slice capability",
        ));
    }
    view.assigned()?;
    Ok(view)
}
fn bytes(value: &Value) -> Result<Vec<u8>, Fault> {
    let view = view(value, true)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(view.length())
        .map_err(|_| Fault::new("cryptography buffer allocation failed"))?;
    for index in 0..view.length() {
        let Value::Byte(byte) = view.get(index)? else {
            return Err(Fault::new("cryptography requires initialized input bytes"));
        };
        bytes.push(byte);
    }
    Ok(bytes)
}
fn destination(value: &Value) -> Result<&crate::MemoryView, Fault> {
    let view = view(value, false)?;
    if let Some(reference) = view.reference() {
        reference.require_writable()?;
    }
    Ok(view)
}
fn write(view: &crate::MemoryView, data: &[u8]) -> Result<(), Fault> {
    for (index, byte) in data.iter().enumerate() {
        view.set(index, Value::Byte(*byte))?;
    }
    Ok(())
}
impl Operation {
    pub(crate) fn invoke(&self, args: &[Value]) -> Result<Value, Fault> {
        match (self, args) {
            (Self::Sha256, [data, output]) => {
                let output = destination(output)?;
                if output.length() < 32 {
                    return Ok(Value::Int32(-1));
                }
                let hash = provider::sha256(&bytes(data)?);
                write(output, &hash)?;
                Ok(Value::Int32(0))
            }
            (Self::HmacSha256, [key, data, output]) => {
                let output = destination(output)?;
                if output.length() < 32 {
                    return Ok(Value::Int32(-1));
                }
                let hash = provider::hmac_sha256(&bytes(key)?, &bytes(data)?);
                write(output, &hash)?;
                Ok(Value::Int32(0))
            }
            (Self::VerifyHmacSha256, [key, data, tag]) => Ok(Value::Boolean(
                provider::verify_hmac_sha256(&bytes(key)?, &bytes(data)?, &bytes(tag)?),
            )),
            (Self::RandomFill, [output]) => {
                let output = destination(output)?;
                // Existing guest view/array quotas bound this temporary buffer. Allocate
                // fallibly, and publish only after the provider has returned.
                let count = output.length();
                let mut data = Vec::new();
                data.try_reserve_exact(count)
                    .map_err(|_| Fault::new("cryptography buffer allocation failed"))?;
                data.resize(count, 0);
                let status = if provider::random_fill(&mut data).is_ok() {
                    0
                } else {
                    -2
                };
                write(output, &data)?;
                Ok(Value::Int32(status))
            }
            _ => Err(Fault::new("invalid cryptography arguments")),
        }
    }
}

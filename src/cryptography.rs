//! Typed private service boundary. Guest references never cross the provider ABI.
use crate::{Fault, Value, metadata::Type};
use neoclr_security_provider as provider;

pub(crate) enum Operation {
    Sha256,
    HmacSha256,
    VerifyHmacSha256,
    RandomFill,
}

fn bytes(value: &Value) -> Result<Vec<u8>, Fault> {
    let Value::ObjectReference(array) = value else {
        return Err(Fault::new("cryptography requires a byte array reference"));
    };
    let Value::Array {
        element: Type::Byte,
        elements,
    } = array.reference.read()?
    else {
        return Err(Fault::new("cryptography requires a byte array"));
    };
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(elements.len())
        .map_err(|_| Fault::new("cryptography buffer allocation failed"))?;
    for value in elements {
        let Value::Byte(byte) = value else {
            return Err(Fault::new("cryptography requires initialized input bytes"));
        };
        bytes.push(byte);
    }
    Ok(bytes)
}
fn destination(value: &Value) -> Result<&crate::value::ObjectReference, Fault> {
    let Value::ObjectReference(array) = value else {
        return Err(Fault::new("cryptography requires a byte array reference"));
    };
    if array.reference.target() != &Type::Array(Box::new(Type::Byte)) {
        return Err(Fault::new("cryptography requires a byte array"));
    }
    array.reference.require_writable()?;
    Ok(array)
}
fn write(array: &crate::value::ObjectReference, data: &[u8]) -> Result<(), Fault> {
    for (index, byte) in data.iter().enumerate() {
        array
            .reference
            .element(index, &Type::Byte)?
            .write(Value::Byte(*byte))?;
    }
    Ok(())
}
impl Operation {
    pub(crate) fn invoke(&self, args: &[Value]) -> Result<Value, Fault> {
        match (self, args) {
            (Self::Sha256, [data, output]) => {
                let output = destination(output)?;
                if output.reference.array_length()? < 32 {
                    return Ok(Value::Int32(-1));
                }
                let hash = provider::sha256(&bytes(data)?);
                write(output, &hash)?;
                Ok(Value::Int32(0))
            }
            (Self::HmacSha256, [key, data, output]) => {
                let output = destination(output)?;
                if output.reference.array_length()? < 32 {
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
                // Existing guest array quotas bound this temporary buffer. Allocate
                // fallibly, and publish only after the provider has returned.
                let count = output.reference.array_length()?;
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

//! Bounded decoding of CLI instruction bytes for the static-call experiment.
//!
//! This is not a PE loader or verifier. The caller must admit the method header,
//! resolve tokens against explicit inputs, and verify signatures and stack effects.

/// Maximum code size admitted by the first experimental decoder (64 KiB).
pub const MAX_CODE_BYTES: usize = 65_536;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instruction {
    /// Offset within the CIL code stream, excluding the method header.
    pub offset: usize,
    pub operation: Operation,
}

/// Token operands remain module-scoped CLI tokens, never runtime definition indices.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operation {
    Nop,
    LoadArgument(u16),
    LoadLocal(u16),
    StoreLocal(u16),
    Int32(i32),
    Dup,
    Pop,
    Call(u32),
    Construct(u32),
    LoadField(u32),
    StoreField(u32),
    Return,
    LoadString(u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodeError {
    pub offset: usize,
    pub message: String,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IL_{:04x}: {}", self.offset, self.message)
    }
}
impl std::error::Error for DecodeError {}

/// Decode a complete code stream, rejecting unsupported bytes even after `ret`.
///
/// A successful decode establishes instruction framing only. In particular, token
/// existence, MemberRef method/field signatures, static-call eligibility, local and
/// argument bounds, maxstack, and valid returns still require metadata and verification.
pub fn decode(code: &[u8]) -> Result<Vec<Instruction>, DecodeError> {
    if code.len() > MAX_CODE_BYTES {
        return Err(DecodeError {
            offset: 0,
            message: "method exceeds 64 KiB code limit".into(),
        });
    }
    if code.is_empty() {
        return Err(DecodeError {
            offset: 0,
            message: "empty CIL code stream".into(),
        });
    }
    let mut result = Vec::new();
    let mut cursor = 0;
    while cursor < code.len() {
        let offset = cursor;
        let error = |message: &str| DecodeError {
            offset,
            message: message.into(),
        };
        let opcode = code[cursor];
        cursor += 1;
        let mut read = |size: usize| -> Result<&[u8], DecodeError> {
            let end = cursor
                .checked_add(size)
                .ok_or_else(|| error("operand size overflow"))?;
            let bytes = code
                .get(cursor..end)
                .ok_or_else(|| error("truncated operand"))?;
            cursor = end;
            Ok(bytes)
        };
        let operation = match opcode {
            0x00 => Operation::Nop,
            0x02..=0x05 => Operation::LoadArgument(u16::from(opcode - 0x02)),
            0x06..=0x09 => Operation::LoadLocal(u16::from(opcode - 0x06)),
            0x0a..=0x0d => Operation::StoreLocal(u16::from(opcode - 0x0a)),
            0x0e => Operation::LoadArgument(u16::from(read(1)?[0])),
            0x11 => Operation::LoadLocal(u16::from(read(1)?[0])),
            0x13 => Operation::StoreLocal(u16::from(read(1)?[0])),
            0x15..=0x1e => Operation::Int32(i32::from(opcode) - 0x16),
            0x1f => Operation::Int32(i32::from(read(1)?[0] as i8)),
            0x20 => Operation::Int32(i32::from_le_bytes(read(4)?.try_into().unwrap())),
            0x25 => Operation::Dup,
            0x26 => Operation::Pop,
            0x28 | 0x73 => {
                let token = u32::from_le_bytes(read(4)?.try_into().unwrap());
                // MethodSpec is intentionally excluded with generic signatures.
                if !matches!(token >> 24, 0x06 | 0x0a) || token & 0x00ff_ffff == 0 {
                    return Err(error(
                        "call/newobj requires a non-nil MethodDef or MemberRef token",
                    ));
                }
                if opcode == 0x28 {
                    Operation::Call(token)
                } else {
                    Operation::Construct(token)
                }
            }
            0x7b | 0x7d => {
                let token = u32::from_le_bytes(read(4)?.try_into().unwrap());
                if !matches!(token >> 24, 0x04 | 0x0a) || token & 0x00ff_ffff == 0 {
                    return Err(error(
                        "field access requires a non-nil Field or MemberRef token",
                    ));
                }
                if opcode == 0x7b {
                    Operation::LoadField(token)
                } else {
                    Operation::StoreField(token)
                }
            }
            0x2a => Operation::Return,
            0x72 => {
                let token = u32::from_le_bytes(read(4)?.try_into().unwrap());
                if token >> 24 != 0x70 || token & 0x00ff_ffff == 0 {
                    return Err(error("ldstr requires a nonzero user-string heap token"));
                }
                Operation::LoadString(token)
            }
            0xfe => match read(1)?[0] {
                op @ (0x09 | 0x0c | 0x0e) => {
                    let index = u16::from_le_bytes(read(2)?.try_into().unwrap());
                    match op {
                        0x09 => Operation::LoadArgument(index),
                        0x0c => Operation::LoadLocal(index),
                        _ => Operation::StoreLocal(index),
                    }
                }
                op => return Err(error(&format!("unsupported opcode 0xfe{op:02x}"))),
            },
            op => return Err(error(&format!("unsupported opcode 0x{op:02x}"))),
        };
        result.push(Instruction { offset, operation });
    }
    Ok(result)
}

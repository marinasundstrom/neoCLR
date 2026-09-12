//! Read-only, bounded CLI PE container inspection. This is not assembly admission.
use crate::cil;

pub const MAX_IMAGE_BYTES: usize = 16 * 1024 * 1024;

/// Explicit opt-in; this profile does not claim arbitrary .NET binary compatibility.
#[derive(Clone, Copy, Debug)]
pub enum Profile {
    StaticCallsV1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(pub String);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
fn fail(message: &str) -> Error {
    Error(message.into())
}
fn bytes(data: &[u8], offset: usize, size: usize) -> Result<&[u8], Error> {
    let end = offset
        .checked_add(size)
        .ok_or_else(|| fail("range overflow"))?;
    data.get(offset..end)
        .ok_or_else(|| fail("truncated image or method body"))
}
fn u16_at(data: &[u8], offset: usize) -> Result<u16, Error> {
    Ok(u16::from_le_bytes(
        bytes(data, offset, 2)?.try_into().unwrap(),
    ))
}
fn u32_at(data: &[u8], offset: usize) -> Result<u32, Error> {
    Ok(u32::from_le_bytes(
        bytes(data, offset, 4)?.try_into().unwrap(),
    ))
}
#[derive(Debug)]
struct Section {
    rva: u32,
    virtual_size: u32,
    raw: usize,
    size: usize,
}

#[derive(Debug)]
pub struct Image<'a> {
    data: &'a [u8],
    sections: Vec<Section>,
    metadata: &'a [u8],
    pub entry_point_token: Option<u32>,
}

#[derive(Debug)]
pub struct MethodBody<'a> {
    pub rva: u32,
    pub header_size: usize,
    pub max_stack: u16,
    pub init_locals: bool,
    /// A StandAloneSig token, still requiring metadata resolution.
    pub local_signature: Option<u32>,
    pub code: &'a [u8],
}
impl MethodBody<'_> {
    pub fn decode(&self) -> Result<Vec<cil::Instruction>, cil::DecodeError> {
        cil::decode(self.code)
    }
}

impl<'a> Image<'a> {
    /// Inspect a PE32 IL-only container without running or resolving anything.
    /// Metadata contents are not validated by this operation.
    pub fn parse(data: &'a [u8], _profile: Profile) -> Result<Self, Error> {
        if data.len() > MAX_IMAGE_BYTES {
            return Err(fail("image exceeds 16 MiB limit"));
        }
        if bytes(data, 0, 2)? != b"MZ" {
            return Err(fail("missing DOS signature"));
        }
        let pe = u32_at(data, 0x3c)? as usize;
        if pe < 0x40 || bytes(data, pe, 4)? != b"PE\0\0" {
            return Err(fail("invalid PE signature offset"));
        }
        let coff = bytes(data, pe + 4, 20)?;
        if u16_at(coff, 0)? != 0x14c {
            return Err(fail("profile requires I386 PE32 container"));
        }
        let count = u16_at(coff, 2)? as usize;
        if count == 0 || count > 96 {
            return Err(fail("invalid section count (1..96 required)"));
        }
        let optional_size = u16_at(coff, 16)? as usize;
        let optional = bytes(data, pe + 24, optional_size)?;
        if u16_at(optional, 0)? != 0x10b {
            return Err(fail("profile requires PE32 optional header"));
        }
        let directories = u32_at(optional, 92)? as usize;
        if !(15..=16).contains(&directories) {
            return Err(fail("unsupported data directory count"));
        }
        bytes(optional, 96, directories * 8)?;
        let header_size = u32_at(optional, 60)? as usize;
        let table = pe + 24 + optional_size;
        bytes(data, table, count * 40)?;
        if header_size < table + count * 40 || header_size > data.len() {
            return Err(fail("invalid SizeOfHeaders"));
        }
        let mut sections: Vec<Section> = Vec::new();
        for index in 0..count {
            let row = bytes(data, table + index * 40, 40)?;
            let section = Section {
                virtual_size: u32_at(row, 8)?,
                rva: u32_at(row, 12)?,
                size: u32_at(row, 16)? as usize,
                raw: u32_at(row, 20)? as usize,
            };
            let extent = section.virtual_size.max(section.size as u32);
            let end = section
                .rva
                .checked_add(extent)
                .ok_or_else(|| fail("section RVA overflow"))?;
            if section.rva < header_size as u32 {
                return Err(fail("section overlaps headers"));
            }
            if section.size != 0 {
                if section.raw < header_size {
                    return Err(fail("raw section overlaps headers"));
                }
                bytes(data, section.raw, section.size)?;
            }
            for prior in &sections {
                let prior_end = prior.rva + prior.virtual_size.max(prior.size as u32);
                if section.rva < prior_end && prior.rva < end {
                    return Err(fail("overlapping virtual sections"));
                }
                if section.size != 0
                    && prior.size != 0
                    && section.raw < prior.raw + prior.size
                    && prior.raw < section.raw + section.size
                {
                    return Err(fail("overlapping raw sections"));
                }
            }
            sections.push(section);
        }
        let mut image = Self {
            data,
            sections,
            metadata: &[],
            entry_point_token: None,
        };
        let cli_rva = u32_at(optional, 96 + 14 * 8)?;
        let cli_size = u32_at(optional, 100 + 14 * 8)? as usize;
        if cli_size != 72 {
            return Err(fail("profile requires 72-byte CLI header"));
        }
        let cli = image.range(cli_rva, cli_size)?;
        if u32_at(cli, 0)? != 72 || u16_at(cli, 4)? != 2 || u16_at(cli, 6)? != 5 {
            return Err(fail("unsupported CLI header version or size"));
        }
        // ILONLY is required. Strong names, native entry points and other modes await admission policy.
        if u32_at(cli, 16)? != 1 {
            return Err(fail("profile requires plain IL-only flags"));
        }
        if cli[24..72].iter().any(|b| *b != 0) {
            return Err(fail(
                "unsupported CLI resources, signatures or native directories",
            ));
        }
        let entry = u32_at(cli, 20)?;
        if entry != 0 {
            if entry >> 24 != 6 || entry & 0xffffff == 0 {
                return Err(fail("entry point must be a MethodDef token"));
            }
            image.entry_point_token = Some(entry);
        }
        let metadata_size = u32_at(cli, 12)? as usize;
        if metadata_size < 4 {
            return Err(fail("missing metadata root"));
        }
        image.metadata = image.range(u32_at(cli, 8)?, metadata_size)?;
        if &image.metadata[..4] != b"BSJB" {
            return Err(fail("invalid metadata signature"));
        }
        Ok(image)
    }

    /// Raw metadata root; tables, heaps and dependency closure still require validation.
    pub fn metadata(&self) -> &'a [u8] {
        self.metadata
    }

    fn tail(&self, rva: u32) -> Result<&'a [u8], Error> {
        for section in &self.sections {
            if let Some(delta) = rva.checked_sub(section.rva) {
                let delta = delta as usize;
                // Do not read raw alignment padding or synthesize zero-filled virtual memory.
                if delta < section.size && delta < section.virtual_size as usize {
                    let available = section.size.min(section.virtual_size as usize) - delta;
                    return bytes(self.data, section.raw + delta, available);
                }
            }
        }
        Err(fail("RVA has no file-backed section data"))
    }
    fn range(&self, rva: u32, size: usize) -> Result<&'a [u8], Error> {
        bytes(self.tail(rva)?, 0, size)
    }

    /// Extract a method body at a metadata-supplied RVA. This does not prove that the
    /// RVA belongs to a MethodDef or validate its signature, locals or stack behavior.
    pub fn method_body(&self, rva: u32) -> Result<MethodBody<'a>, Error> {
        let body = self.tail(rva)?;
        let first = *body.first().ok_or_else(|| fail("missing method header"))?;
        let (header_size, code_size, max_stack, init_locals, local_signature) = match first & 3 {
            2 => (1, (first >> 2) as usize, 8, false, None),
            3 => {
                if rva % 4 != 0 {
                    return Err(fail("fat method header must be aligned to four bytes"));
                }
                let flags = u16_at(body, 0)?;
                if flags >> 12 != 3 || flags & 0x0fff & !0x13 != 0 {
                    return Err(fail(
                        "unsupported fat method flags/header (including extra sections)",
                    ));
                }
                let local = u32_at(body, 8)?;
                if local != 0 && (local >> 24 != 0x11 || local & 0xffffff == 0) {
                    return Err(fail("invalid local signature token"));
                }
                (
                    12,
                    u32_at(body, 4)? as usize,
                    u16_at(body, 2)?,
                    flags & 0x10 != 0,
                    (local != 0).then_some(local),
                )
            }
            _ => return Err(fail("unsupported method header format")),
        };
        if code_size == 0 || code_size > cil::MAX_CODE_BYTES {
            return Err(fail("invalid method code size (1..65536 required)"));
        }
        Ok(MethodBody {
            rva,
            header_size,
            max_stack,
            init_locals,
            local_signature,
            code: bytes(body, header_size, code_size)?,
        })
    }
}

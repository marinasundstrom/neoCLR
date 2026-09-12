use neoclr::pe::{Image, MAX_IMAGE_BYTES, Profile};
fn hex(text: &str) -> Vec<u8> {
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
fn fixture() -> (Vec<u8>, serde_json::Value) {
    let report: serde_json::Value =
        serde_json::from_str(include_str!("../docs/experiments/pe-reader/result.json")).unwrap();
    (hex(report["ImageHex"].as_str().unwrap()), report)
}
fn read32(data: &[u8], at: usize) -> usize {
    u32::from_le_bytes(data[at..at + 4].try_into().unwrap()) as usize
}
fn put32(data: &mut [u8], at: usize, value: usize) {
    data[at..at + 4].copy_from_slice(&(value as u32).to_le_bytes());
}
fn table(data: &[u8]) -> usize {
    let pe = read32(data, 0x3c);
    pe + 24 + u16::from_le_bytes(data[pe + 20..pe + 22].try_into().unwrap()) as usize
}
fn file_offset(data: &[u8], rva: usize) -> usize {
    // Fixture methods and CLI header reside in the first section.
    let table = table(data);
    read32(data, table + 20) + rva - read32(data, table + 12)
}
#[test]
fn reads_dotnet_pe_and_tiny_and_fat_bodies() {
    let (data, report) = fixture();
    let image = Image::parse(&data, Profile::StaticCallsV1).unwrap();
    assert_eq!(image.entry_point_token, None);
    assert_eq!(
        image.metadata().len(),
        report["MetadataSize"].as_u64().unwrap() as usize
    );
    for method in report["Methods"].as_array().unwrap() {
        let body = image
            .method_body(method["Rva"].as_u64().unwrap() as u32)
            .unwrap();
        assert_eq!(body.max_stack as u64, method["MaxStack"].as_u64().unwrap());
        assert_eq!(body.init_locals, method["InitLocals"].as_bool().unwrap());
        assert_eq!(
            body.local_signature.unwrap_or(0) as u64,
            method["LocalSignature"].as_u64().unwrap()
        );
        assert_eq!(body.code, hex(method["CodeHex"].as_str().unwrap()));
        assert!(!body.decode().unwrap().is_empty());
        assert_eq!(body.header_size, if body.init_locals { 12 } else { 1 });
    }
}
#[test]
fn every_truncated_fixture_prefix_fails_without_panicking() {
    let (data, _) = fixture();
    // Every section's declared raw extent must be available, even unused padding.
    for length in 0..data.len() {
        assert!(
            Image::parse(&data[..length], Profile::StaticCallsV1).is_err(),
            "length {length}"
        );
    }
    assert!(
        Image::parse(&vec![0; MAX_IMAGE_BYTES + 1], Profile::StaticCallsV1)
            .unwrap_err()
            .0
            .contains("limit")
    );
}
#[test]
fn malformed_headers_and_overlapping_sections_are_rejected() {
    let (original, _) = fixture();
    let pe = read32(&original, 0x3c);
    let table = table(&original);
    let cli = file_offset(&original, read32(&original, pe + 24 + 96 + 14 * 8));
    for (at, value, message) in [
        (0x3c, usize::MAX, "truncated"),
        (pe + 24 + 92, 0, "directory"),
        (table + 20, 0, "headers"),
        (table + 12, 0xfffffff0, "overflow"),
        (
            table + 40 + 12,
            read32(&original, table + 12),
            "overlapping",
        ),
        (
            table + 40 + 20,
            read32(&original, table + 20),
            "overlapping",
        ),
        (cli + 16, 0x11, "IL-only"),
        (cli + 24, 1, "unsupported"),
        (cli + 20, 0x06000000, "MethodDef"),
        (cli + 8, 0xffffffff, "RVA"),
    ] {
        let mut data = original.clone();
        put32(&mut data, at, value);
        assert!(
            Image::parse(&data, Profile::StaticCallsV1)
                .unwrap_err()
                .0
                .contains(message),
            "at {at}"
        );
    }
}
#[test]
fn method_headers_reject_extra_sections_bad_tokens_and_sizes() {
    let (original, report) = fixture();
    let rva = report["Methods"][1]["Rva"].as_u64().unwrap() as u32;
    let offset = file_offset(&original, rva as usize);
    for (relative, value, message) in [
        (0, 0x301b, "unsupported"),
        (8, 0x06000001, "signature"),
        (8, 0x11000000, "signature"),
        (4, 65537, "code size"),
        (4, 0, "code size"),
        (4, 60000, "truncated"),
    ] {
        let mut data = original.clone();
        put32(&mut data, offset + relative, value);
        let image = Image::parse(&data, Profile::StaticCallsV1).unwrap();
        assert!(image.method_body(rva).unwrap_err().0.contains(message));
    }
    let image = Image::parse(&original, Profile::StaticCallsV1).unwrap();
    assert!(image.method_body(0).is_err());
    assert!(image.method_body(u32::MAX).is_err());
    // The first section's raw alignment padding cannot serve as a method body.
    let table = table(&original);
    let padding = read32(&original, table + 12) + read32(&original, table + 8);
    assert!(image.method_body(padding as u32).is_err());
}

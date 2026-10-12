//! Private host primitives for the future managed cryptography projection.
//! Pointers are trusted host buffers, valid for the supplied lengths and call.
use neoclr_security_provider as provider;

const SUCCESS: i32 = 0;
const INVALID: i32 = -1;
const PROVIDER_FAILURE: i32 = -2;
const TAG_LENGTH: usize = 32;

unsafe fn input<'a>(data: *const u8, count: u32) -> Option<&'a [u8]> {
    if count == 0 {
        return Some(&[]);
    }
    if data.is_null() || count as usize > isize::MAX as usize {
        return None;
    }
    Some(unsafe { std::slice::from_raw_parts(data, count as usize) })
}
fn output_valid(output: *mut u8, capacity: u32) -> bool {
    !output.is_null() && capacity >= TAG_LENGTH as u32
}

/// Hash first, then copy, allowing the destination to overlap the input.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_crypto_sha256(
    data: *const u8,
    count: u32,
    output: *mut u8,
    capacity: u32,
) -> i32 {
    if !output_valid(output, capacity) {
        return INVALID;
    }
    let Some(data) = (unsafe { input(data, count) }) else {
        return INVALID;
    };
    let hash = provider::sha256(data);
    unsafe {
        std::ptr::copy_nonoverlapping(hash.as_ptr(), output, TAG_LENGTH);
    }
    SUCCESS
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_crypto_hmac_sha256(
    key: *const u8,
    key_count: u32,
    data: *const u8,
    count: u32,
    output: *mut u8,
    capacity: u32,
) -> i32 {
    if !output_valid(output, capacity) {
        return INVALID;
    }
    let Some(key) = (unsafe { input(key, key_count) }) else {
        return INVALID;
    };
    let Some(data) = (unsafe { input(data, count) }) else {
        return INVALID;
    };
    let tag = provider::hmac_sha256(key, data);
    unsafe {
        std::ptr::copy_nonoverlapping(tag.as_ptr(), output, TAG_LENGTH);
    }
    SUCCESS
}

/// 0 valid, 1 mismatch (including a truncated tag), -1 invalid buffer arguments.
/// Delegate verification to ring rather than an early-exit byte comparison.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_crypto_hmac_sha256_verify(
    key: *const u8,
    key_count: u32,
    data: *const u8,
    count: u32,
    tag: *const u8,
    tag_count: u32,
) -> i32 {
    let Some(key) = (unsafe { input(key, key_count) }) else {
        return INVALID;
    };
    let Some(data) = (unsafe { input(data, count) }) else {
        return INVALID;
    };
    let Some(tag) = (unsafe { input(tag, tag_count) }) else {
        return INVALID;
    };
    if provider::verify_hmac_sha256(key, data, tag) {
        SUCCESS
    } else {
        1
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_crypto_random_fill(output: *mut u8, count: u32) -> i32 {
    if count == 0 {
        return SUCCESS;
    }
    if output.is_null() || count as usize > isize::MAX as usize {
        return INVALID;
    }
    match provider::random_fill(unsafe { std::slice::from_raw_parts_mut(output, count as usize) }) {
        Ok(()) => SUCCESS,
        Err(_) => PROVIDER_FAILURE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn hex(data: &[u8]) -> String {
        data.iter().map(|b| format!("{b:02x}")).collect()
    }
    #[test]
    fn sha256_known_answers_and_overlap() {
        for (data, expected) in [
            (
                &b""[..],
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ),
            (
                &b"abc"[..],
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            ),
            (
                &b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"[..],
                "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
            ),
        ] {
            let mut output = [0xa5; 80];
            output[..data.len()].copy_from_slice(data);
            assert_eq!(
                unsafe {
                    neoclr_crypto_sha256(
                        output.as_ptr(),
                        data.len() as u32,
                        output.as_mut_ptr(),
                        80,
                    )
                },
                SUCCESS
            );
            assert_eq!(hex(&output[..32]), expected);
            assert_eq!(output[79], 0xa5);
        }
        let mut output = [0; 32];
        assert_eq!(
            unsafe { neoclr_crypto_sha256(std::ptr::null(), 0, output.as_mut_ptr(), 32) },
            SUCCESS
        );
    }
    #[test]
    fn hmac_rfc4231_sign_and_verify() {
        for (key, data, expected) in [
            (
                vec![0x0b; 20],
                &b"Hi There"[..],
                "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7",
            ),
            (
                b"Jefe".to_vec(),
                &b"what do ya want for nothing?"[..],
                "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843",
            ),
            (
                vec![0xaa; 131],
                &b"Test Using Larger Than Block-Size Key - Hash Key First"[..],
                "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54",
            ),
        ] {
            let mut tag = [0; 32];
            unsafe {
                assert_eq!(
                    neoclr_crypto_hmac_sha256(
                        key.as_ptr(),
                        key.len() as u32,
                        data.as_ptr(),
                        data.len() as u32,
                        tag.as_mut_ptr(),
                        32
                    ),
                    SUCCESS
                );
                assert_eq!(hex(&tag), expected);
                assert_eq!(
                    neoclr_crypto_hmac_sha256_verify(
                        key.as_ptr(),
                        key.len() as u32,
                        data.as_ptr(),
                        data.len() as u32,
                        tag.as_ptr(),
                        32
                    ),
                    SUCCESS
                );
                for index in [0, 15, 31] {
                    tag[index] ^= 1;
                    assert_eq!(
                        neoclr_crypto_hmac_sha256_verify(
                            key.as_ptr(),
                            key.len() as u32,
                            data.as_ptr(),
                            data.len() as u32,
                            tag.as_ptr(),
                            32
                        ),
                        1
                    );
                    tag[index] ^= 1;
                }
                assert_eq!(
                    neoclr_crypto_hmac_sha256_verify(
                        key.as_ptr(),
                        key.len() as u32,
                        data.as_ptr(),
                        data.len() as u32,
                        tag.as_ptr(),
                        31
                    ),
                    1
                );
            }
        }
    }
    #[test]
    fn invalid_arguments_do_not_modify_output() {
        let mut output = [0xa5; 32];
        unsafe {
            assert_eq!(
                neoclr_crypto_sha256(std::ptr::null(), 1, output.as_mut_ptr(), 32),
                INVALID
            );
            assert_eq!(
                neoclr_crypto_sha256(std::ptr::null(), 0, output.as_mut_ptr(), 31),
                INVALID
            );
            assert_eq!(
                neoclr_crypto_hmac_sha256(
                    std::ptr::null(),
                    1,
                    std::ptr::null(),
                    0,
                    output.as_mut_ptr(),
                    32
                ),
                INVALID
            );
            assert_eq!(
                neoclr_crypto_hmac_sha256_verify(
                    std::ptr::null(),
                    0,
                    std::ptr::null(),
                    1,
                    std::ptr::null(),
                    0
                ),
                INVALID
            );
            assert_eq!(neoclr_crypto_random_fill(std::ptr::null_mut(), 1), INVALID);
        }
        assert_eq!(output, [0xa5; 32]);
    }
    #[test]
    fn random_fill_respects_bounds_and_empty_input() {
        let mut output = [0xa5; 66];
        unsafe {
            assert_eq!(neoclr_crypto_random_fill(std::ptr::null_mut(), 0), SUCCESS);
            assert_eq!(
                neoclr_crypto_random_fill(output.as_mut_ptr().add(1), 64),
                SUCCESS
            );
        }
        assert_eq!(output[0], 0xa5);
        assert_eq!(output[65], 0xa5);
        // Deliberately no statistical assertions: this checks the provider contract.
    }
}

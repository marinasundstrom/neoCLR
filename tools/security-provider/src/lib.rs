//! Private shared cryptographic implementation, independent of TLS and guest storage.
use ring::{
    digest, hmac,
    rand::{SecureRandom, SystemRandom},
};

/// Failure to obtain operating-system randomness. The destination has been cleared.
#[derive(Debug, PartialEq, Eq)]
pub struct RandomFailure;

pub fn sha256(data: &[u8]) -> [u8; 32] {
    digest::digest(&digest::SHA256, data)
        .as_ref()
        .try_into()
        .expect("SHA-256 length")
}
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    hmac::sign(&hmac::Key::new(hmac::HMAC_SHA256, key), data)
        .as_ref()
        .try_into()
        .expect("HMAC-SHA-256 length")
}
pub fn verify_hmac_sha256(key: &[u8], data: &[u8], tag: &[u8]) -> bool {
    hmac::verify(&hmac::Key::new(hmac::HMAC_SHA256, key), data, tag).is_ok()
}
pub fn random_fill(output: &mut [u8]) -> Result<(), RandomFailure> {
    if output.is_empty() {
        return Ok(());
    }
    fill(output, |bytes| {
        SystemRandom::new().fill(bytes).map_err(|_| RandomFailure)
    })
}
fn fill(
    output: &mut [u8],
    generate: impl FnOnce(&mut [u8]) -> Result<(), RandomFailure>,
) -> Result<(), RandomFailure> {
    if generate(output).is_err() {
        output.fill(0);
        Err(RandomFailure)
    } else {
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failure_discards_partial_random_output() {
        let mut bytes = [0xa5; 32];
        assert_eq!(
            fill(&mut bytes, |output| {
                output[0] = 73;
                Err(RandomFailure)
            }),
            Err(RandomFailure)
        );
        assert_eq!(bytes, [0; 32]);
    }
}

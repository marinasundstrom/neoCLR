#ifndef NEOCLR_PRIVATE_CRYPTO_H
#define NEOCLR_PRIVATE_CRYPTO_H
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
/* Private host ABI. Buffers are borrowed for the call, never retained.
 * A pointer may be null only when its input length is zero.
 * Hash/MAC outputs require at least 32 writable bytes; only 32 are written.
 * Outputs may overlap inputs. Invalid arguments leave output untouched.
 * 0 success, -1 invalid arguments, -2 random provider failure (output cleared).
 * No algorithm, key-generation policy or trust-store selection is implicit.
 */
int32_t neoclr_crypto_sha256(const uint8_t *data, uint32_t count,
                           uint8_t *output, uint32_t capacity);
int32_t neoclr_crypto_hmac_sha256(const uint8_t *key, uint32_t key_count,
                                const uint8_t *data, uint32_t count,
                                uint8_t *output, uint32_t capacity);
/* Verify returns 0 valid, 1 mismatch (including non-32-byte tags), -1 invalid. */
int32_t neoclr_crypto_hmac_sha256_verify(const uint8_t *key, uint32_t key_count,
                                       const uint8_t *data, uint32_t count,
                                       const uint8_t *tag, uint32_t tag_count);
int32_t neoclr_crypto_random_fill(uint8_t *output, uint32_t count);
#ifdef __cplusplus
}
#endif
#endif

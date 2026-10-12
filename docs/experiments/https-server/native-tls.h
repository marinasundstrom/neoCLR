#ifndef NEOCLR_PRIVATE_TLS_H
#define NEOCLR_PRIVATE_TLS_H
#include <stdint.h>
/* Internal borrowed I/O: -1 would-block, -2 failure, otherwise transferred bytes. */
typedef int64_t (*neoclr_tls_io)(void *, unsigned char *, uint32_t, int32_t);
/* Configuration status: output is cleared on failure. Buffers are borrowed only
 * during construction; successful sessions retain config after its handle is freed.
 * The output slot must be writable and must not alias input storage. */
enum neoclr_tls_configuration_status {
    NEOCLR_TLS_CONFIG_OK = 0,
    NEOCLR_TLS_CONFIG_INVALID_INPUT = -10,
    NEOCLR_TLS_CONFIG_INVALID_CERTIFICATE = -11,
    NEOCLR_TLS_CONFIG_INVALID_PRIVATE_KEY = -12,
    NEOCLR_TLS_CONFIG_KEY_MISMATCH = -13,
    NEOCLR_TLS_CONFIG_INVALID_TRUST_ANCHOR = -14,
    NEOCLR_TLS_CONFIG_INVALID_PROTOCOL = -15,
    NEOCLR_TLS_CONFIG_PROVIDER_FAILURE = -16
};
int32_t neoclr_tls_server_config_create_pem(const unsigned char *, uint32_t,
    const unsigned char *, uint32_t, const unsigned char *, uint32_t, void **);
int32_t neoclr_tls_client_config_create_pem(const unsigned char *, uint32_t,
    const unsigned char *, uint32_t, void **);
void *neoclr_tls_server_config_from_pem(const unsigned char *, uint32_t,
    const unsigned char *, uint32_t, const unsigned char *, uint32_t);
void *neoclr_tls_client_config_from_pem(const unsigned char *, uint32_t,
    const unsigned char *, uint32_t);
void neoclr_tls_client_config_free(void *);
void *neoclr_tls_client_session_new(void *, const unsigned char *, uint32_t);
int32_t neoclr_tls_handshake(void *, void *, neoclr_tls_io);
/* Local close-notify flush: 0 done, -1 pending, -2 failed, -3 handshake/write pending.
 * Does not close transport or await peer notification. Reads remain available;
 * neoclr_tls_transfer rejects subsequent writes with -3. Serialize session calls. */
int32_t neoclr_tls_shutdown(void *, void *, neoclr_tls_io);
void *neoclr_tls_config_new(const char *, const char *);
void neoclr_tls_config_free(void *);
void *neoclr_tls_session_new(void *);
int64_t neoclr_tls_transfer(void *, void *, neoclr_tls_io, unsigned char *, uint32_t, int32_t);
void neoclr_tls_abort(void *);
void neoclr_tls_session_free(void *, void *, neoclr_tls_io);
/* Checked resource contexts for managed adapters. The host owns this pointer and
 * serializes calls, including callbacks (no reentrancy). Guests receive only IDs.
 * IDs are unique/non-reused within this provider instance, not authorization tokens. Free/release performs
 * abortive cleanup without I/O; explicit shutdown must be polled before release.
 * Additional statuses: -17 resource limit, -18 wrong/stale/foreign handle, -19 name.
 * All create outputs are zeroed on failure; slots must not alias input storage. */
void *neoclr_tls_context_new(uint32_t);
void neoclr_tls_context_free(void *);
int32_t neoclr_tls_context_release(void *, uint64_t);
int32_t neoclr_tls_context_server_config(void *, const unsigned char *, uint32_t,
    const unsigned char *, uint32_t, const unsigned char *, uint32_t, uint64_t *);
int32_t neoclr_tls_context_client_config(void *, const unsigned char *, uint32_t,
    const unsigned char *, uint32_t, uint64_t *);
int32_t neoclr_tls_context_session(void *, uint64_t, const unsigned char *, uint32_t, uint64_t *);
int32_t neoclr_tls_context_handshake(void *, uint64_t, void *, neoclr_tls_io);
int32_t neoclr_tls_context_shutdown(void *, uint64_t, void *, neoclr_tls_io);
int32_t neoclr_tls_context_abort(void *, uint64_t);
int64_t neoclr_tls_context_transfer(void *, uint64_t, void *, neoclr_tls_io,
    unsigned char *, uint32_t, int32_t);
/* Borrowed-memory TLS pump: 1 handshake, 2 read, 3 write, 4 local shutdown.
 * Each byte buffer is <=64 KiB. Nonempty ranges must be valid, pairwise disjoint
 * and disjoint from the writable aligned progress record and opaque context.
 * Empty buffers may be NULL. eof is 0 (would-block) or 1 (no bytes follow input).
 * Returns the underlying operation status; -10 invalid input, -18 invalid ID.
 * Progress counts apply even on pending/failure. Preserve unconsumed ciphertext;
 * send all produced ciphertext before reusing output, even when status is pending.
 * A pending write is retried with the same plaintext until completion or abort.
 * Handshake/shutdown require empty plaintext. No callback, socket, I/O or retained
 * buffer. Rejected null/overflow/length/overlap/op/eof arguments leave progress
 * untouched. The host remains responsible for accessible, aligned memory. */
typedef struct { uint32_t consumed, produced; } neoclr_tls_pump_progress;
int64_t neoclr_tls_context_pump_v1(void *, uint64_t, int32_t,
    const unsigned char *, uint32_t, int32_t, unsigned char *, uint32_t,
    unsigned char *, uint32_t, neoclr_tls_pump_progress *);
#endif

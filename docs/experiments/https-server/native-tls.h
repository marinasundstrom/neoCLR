#ifndef NEOCLR_PRIVATE_TLS_H
#define NEOCLR_PRIVATE_TLS_H
#include <stdint.h>
/* Internal borrowed I/O: -1 would-block, -2 failure, otherwise transferred bytes. */
typedef int64_t (*neoclr_tls_io)(void *, unsigned char *, uint32_t, int32_t);
void *neoclr_tls_config_new(const char *, const char *);
void neoclr_tls_config_free(void *);
void *neoclr_tls_session_new(void *);
int64_t neoclr_tls_transfer(void *, void *, neoclr_tls_io, unsigned char *, uint32_t, int32_t);
void neoclr_tls_abort(void *);
void neoclr_tls_session_free(void *, void *, neoclr_tls_io);
#endif

#define _CRT_SECURE_NO_WARNINGS
#include "tls-managed.h"
#include "../https-server/native-tls.h"
#include <stdio.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "TLS adapter check failed: %d\n", __LINE__); return 1; } } while (0)
static void *array(neoclr_aot_context *c, const void *data, uint32_t count) {
    void *p = NULL;
    if (neoclr_gc_allocate_v1(&c->text, 16 + count, NEOCLR_GC_BYTES, &p)) return NULL;
    uint64_t header[2] = {UINT64_C(0x80000001), count};
    memcpy(p, header, 16);
    if (count) memcpy((char *)p + 16, data, count);
    return p;
}
static void *file(neoclr_aot_context *c, const char *path) {
    unsigned char data[65536];
    FILE *f = fopen(path, "rb");
    if (!f) return NULL;
    size_t n = fread(data, 1, sizeof(data), f);
    int failed = ferror(f) || !feof(f);
    fclose(f);
    return failed ? NULL : array(c, data, (uint32_t)n);
}
int main(int argc, char **argv) {
    CHECK(argc == 3);
    uint64_t storage[32768] = {0}, other_storage[32] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)storage, sizeof(storage), 0}};
    neoclr_aot_context other = {.text = {(unsigned char *)other_storage, sizeof(other_storage), 0}};
    void *cert = file(&c, argv[1]), *key = file(&c, argv[2]);
    void *empty = array(&c, NULL, 0), *config = array(&c, NULL, 0);
    void *session = array(&c, NULL, 0), *alien = array(&other, NULL, 0);
    CHECK(cert && key && empty && config && session && alien);
    /* Exercise the C buffer-pump ABI with a real client flight and no socket. */
    void *provider = neoclr_tls_context_new(2);
    uint64_t config_id = 0, session_id = 0, cert_size = 0;
    memcpy(&cert_size, (char *)cert + 8, 8);
    CHECK(provider);
    CHECK(!neoclr_tls_context_client_config(provider, (unsigned char *)cert + 16,
        (uint32_t)cert_size, NULL, 0, &config_id));
    CHECK(!neoclr_tls_context_session(provider, config_id, (const unsigned char *)"localhost", 9, &session_id));
    unsigned char encrypted[17] = {0};
    neoclr_tls_pump_progress progress = {123, 456};
    CHECK(neoclr_tls_context_pump_v1(provider, session_id, 1, NULL, 0, 0,
        encrypted, sizeof(encrypted), NULL, 0, &progress) == -1);
    CHECK(!progress.consumed && progress.produced == sizeof(encrypted));
    CHECK(neoclr_tls_context_pump_v1(provider, session_id, 1, NULL, 0, 0,
        NULL, 0, NULL, 0, &progress) == -1 && !progress.produced);
    neoclr_tls_context_free(provider);
    int32_t result = 777;
    CHECK(neoclr_tls_managed_v1(NEOCLR_TLS_RELEASE, &c, alien, NULL, NULL, NULL, &result) == 3 && result == 777);
    CHECK(neoclr_tls_managed_v1(NEOCLR_TLS_RELEASE, &c, (char *)config + 8, NULL, NULL, NULL, &result) == 3);
    CHECK(neoclr_tls_managed_v1(NEOCLR_TLS_RELEASE, &c, cert, NULL, NULL, NULL, &result) == 3);
    CHECK(!neoclr_tls_managed_v1(NEOCLR_TLS_CLIENT, &c, config, empty, empty, NULL, &result) && result == -14);
    CHECK(!neoclr_gc_entry_check_v1(&c)); /* failed creation leaves no registration */
    CHECK(!neoclr_tls_managed_v1(NEOCLR_TLS_SERVER, &c, config, cert, key, empty, &result) && !result);
    CHECK(neoclr_gc_entry_check_v1(&c) == 3);
    result = 777;
    CHECK(neoclr_tls_managed_v1(NEOCLR_TLS_SERVER, &c, config, cert, key, empty, &result) == 3 && result == 777);
    CHECK(!neoclr_tls_managed_v1(NEOCLR_TLS_SESSION, &c, session, config, empty, NULL, &result) && !result);
    void *extra = array(&c, NULL, 0);
    CHECK(extra);
    CHECK(!neoclr_tls_managed_v1(NEOCLR_TLS_SESSION, &c, extra, session, empty, NULL, &result) && result == -18);
    uint64_t strong = 0;
    CHECK(!neoclr_gc_host_root_create_v1(&c, session, &strong));
    /* Keep only session: PEM, configuration and its managed owner can die. */
    CHECK(!neoclr_gc_collect_v1(&c, NULL));
    CHECK(!neoclr_tls_managed_v1(NEOCLR_TLS_ABORT, &c, session, NULL, NULL, NULL, &result) && !result);
    CHECK(neoclr_tls_managed_v1(NEOCLR_TLS_RELEASE, &c, config, NULL, NULL, NULL, &result) == 3);
    CHECK(!neoclr_tls_managed_v1(NEOCLR_TLS_RELEASE, &c, session, NULL, NULL, NULL, &result) && !result);
    CHECK(!neoclr_tls_managed_v1(NEOCLR_TLS_RELEASE, &c, session, NULL, NULL, NULL, &result) && !result);
    CHECK(!neoclr_gc_host_root_release_v1(&c, strong));
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used && !neoclr_gc_entry_check_v1(&c));
    /* Client lifetime and explicit host teardown, including terminal Fault. */
    cert = file(&c, argv[1]); empty = array(&c, NULL, 0); config = array(&c, NULL, 0);
    session = array(&c, NULL, 0);
    void *name = array(&c, "localhost", 9);
    CHECK(cert && empty && config && session && name);
    CHECK(!neoclr_tls_managed_v1(NEOCLR_TLS_CLIENT, &c, config, cert, empty, NULL, &result) && !result);
    CHECK(!neoclr_tls_managed_v1(NEOCLR_TLS_SESSION, &c, session, config, name, NULL, &result) && !result);
    CHECK(!neoclr_tls_managed_v1(NEOCLR_TLS_RELEASE, &c, config, NULL, NULL, NULL, &result) && !result);
    c.fault.code = 1;
    neoclr_tls_managed_dispose_v1(&c);
    CHECK(c.fault.code == 1 && !neoclr_gc_entry_check_v1(&c));
    neoclr_tls_managed_dispose_v1(&c);
    CHECK(!neoclr_tls_managed_v1(NEOCLR_TLS_ABORT, &c, session, NULL, NULL, NULL, &result) && result == -18);
    /* Reserved uninitialized and malformed payloads fault before provider access. */
    void *bad = array(&c, "xx", 2);
    CHECK(bad);
    uint64_t tag = UINT64_C(0x80000002), count = 1;
    memcpy(bad, &tag, 8); memcpy((char *)bad + 8, &count, 8);
    ((unsigned char *)bad)[17] = 0;
    result = 777;
    CHECK(neoclr_tls_managed_v1(NEOCLR_TLS_CLIENT, &c, config, bad, empty, NULL, &result) == 3 && result == 777);
    count = 65536; memcpy((char *)bad + 8, &count, 8);
    CHECK(neoclr_tls_managed_v1(NEOCLR_TLS_CLIENT, &c, config, bad, empty, NULL, &result) == 3 && result == 777);
    CHECK(!neoclr_gc_entry_check_v1(&c));
    return 0;
}

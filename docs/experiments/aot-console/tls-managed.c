#include "tls-managed.h"
#include "../https-server/native-tls.h"
#include <stdlib.h>
#include <string.h>

typedef struct tls_context tls_context;
typedef struct tls_owner {
    neoclr_gc_resource registration;
    tls_context *context;
    void *owner;
    uint64_t handle;
    struct tls_owner *next;
} tls_owner;
struct tls_context {
    neoclr_aot_context *heap;
    void *provider;
    tls_owner *owners;
    tls_context *next;
};
static _Thread_local tls_context *contexts;
typedef struct { const unsigned char *data; uint32_t count; } bytes;
static int32_t array(neoclr_aot_context *context, const void *value, bytes *out) {
    uint64_t size, tag, count;
    uint32_t kind;
    if (neoclr_gc_allocation_info_v1(context, value, &size, &kind) ||
        kind != NEOCLR_GC_BYTES || size < 16) return 3;
    memcpy(&tag, value, 8);
    memcpy(&count, (const unsigned char *)value + 8, 8);
    if ((tag != UINT64_C(0x80000001) && tag != UINT64_C(0x80000002)) ||
        count > NEOCLR_NATIVE_BYTE_ARRAY_LIMIT ||
        count > (size - 16) / (tag == UINT64_C(0x80000002) ? 2 : 1)) return 3;
    const unsigned char *data = (const unsigned char *)value + 16;
    if (tag == UINT64_C(0x80000002))
        for (uint64_t i = 0; i < count; ++i) if (!data[count + i]) return 3;
    *out = (bytes){data, (uint32_t)count};
    return 0;
}
static int32_t owner_token(neoclr_aot_context *context, void *owner) {
    bytes view;
    return array(context, owner, &view) || view.count ? 3 : 0;
}
static tls_context *find_context(neoclr_aot_context *heap) {
    for (tls_context *c = contexts; c; c = c->next) if (c->heap == heap) return c;
    return NULL;
}
static tls_owner *find_owner(tls_context *context, void *owner) {
    if (context) for (tls_owner *o = context->owners; o; o = o->next)
        if (o->owner == owner) return o;
    return NULL;
}
static void drop_empty_context(tls_context *context) {
    if (context->owners) return;
    for (tls_context **link = &contexts; *link; link = &(*link)->next)
        if (*link == context) {
            *link = context->next;
            neoclr_tls_context_free(context->provider);
            free(context);
            return;
        }
}
static void cleanup(void *state) {
    tls_owner *owner = state;
    tls_context *context = owner->context;
    for (tls_owner **link = &context->owners; *link; link = &(*link)->next)
        if (*link == owner) { *link = owner->next; break; }
    neoclr_tls_context_release(context->provider, owner->handle);
    free(owner);
    drop_empty_context(context);
}
void neoclr_tls_managed_dispose_v1(neoclr_aot_context *heap) {
    tls_context *context;
    while ((context = find_context(heap)) != NULL)
        if (neoclr_gc_resource_release_v1(heap, &context->owners->registration)) return;
}
static uint32_t read32(const unsigned char *p) {
    return (uint32_t)p[0] | (uint32_t)p[1] << 8 | (uint32_t)p[2] << 16 | (uint32_t)p[3] << 24;
}
static void write32(unsigned char *p, uint32_t value) {
    for (int i = 0; i < 4; ++i) p[i] = (unsigned char)(value >> (8 * i));
}
int32_t neoclr_tls_managed_v1(int32_t operation, neoclr_aot_context *heap,
    void *owner, void *first, void *second, void *third, int32_t *result) {
    if (operation > NEOCLR_TLS_ABORT) return 3;
    return neoclr_tls_managed_v2(operation, heap, owner, first, second, third, NULL, result);
}
int32_t neoclr_tls_managed_v2(int32_t operation, neoclr_aot_context *heap,
    void *owner, void *first, void *second, void *third, void *fourth, int32_t *result) {
    if (!result || operation < NEOCLR_TLS_CLIENT || operation > NEOCLR_TLS_PUMP ||
        owner_token(heap, owner)) return 3;
    tls_context *context = find_context(heap);
    tls_owner *existing = find_owner(context, owner);
    if (operation == NEOCLR_TLS_RELEASE) {
        if (existing && neoclr_gc_resource_release_v1(heap, &existing->registration)) return 3;
        *result = 0;
        return 0;
    }
    if (operation == NEOCLR_TLS_ABORT) {
        *result = existing ? neoclr_tls_context_abort(context->provider, existing->handle) : -18;
        return 0;
    }
    if (operation == NEOCLR_TLS_PUMP) {
        bytes input, output, plaintext, control;
        if (array(heap, first, &input) || array(heap, second, &output) ||
            array(heap, third, &plaintext) || array(heap, fourth, &control) ||
            first == second || first == third || first == fourth || second == third ||
            second == fourth || third == fourth) return 3;
        if (control.count != 16 || control.data[0] < 1 || control.data[0] > 4 ||
            control.data[1] > 1 || control.data[2] || control.data[3] ||
            read32(control.data + 4) > input.count) { *result = -10; return 0; }
        neoclr_tls_pump_progress progress = {0};
        *result = existing ? (int32_t)neoclr_tls_context_pump_v1(context->provider,
            existing->handle, control.data[0], input.data, read32(control.data + 4),
            control.data[1], (unsigned char *)output.data, output.count,
            (unsigned char *)plaintext.data, plaintext.count, &progress) : -18;
        write32((unsigned char *)control.data + 8, progress.consumed);
        write32((unsigned char *)control.data + 12, progress.produced);
        return 0;
    }
    if (existing) return 3;
    bytes a = {0}, b = {0}, c = {0};
    tls_owner *configuration = NULL;
    if (operation == NEOCLR_TLS_SESSION) {
        if (owner_token(heap, first) || array(heap, second, &b)) return 3;
        configuration = find_owner(context, first);
        if (!configuration) { *result = -18; return 0; }
    } else {
        if (array(heap, first, &a) || array(heap, second, &b) ||
            (operation == NEOCLR_TLS_SERVER && array(heap, third, &c))) return 3;
    }
    if (!context) {
        context = calloc(1, sizeof(*context));
        if (!context) return NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT;
        context->provider = neoclr_tls_context_new(1024);
        if (!context->provider) { free(context); return NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT; }
        context->heap = heap;
        context->next = contexts;
        contexts = context;
    }
    tls_owner *entry = calloc(1, sizeof(*entry));
    if (!entry) { drop_empty_context(context); return NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT; }
    int32_t status;
    if (operation == NEOCLR_TLS_CLIENT)
        status = neoclr_tls_context_client_config(context->provider, a.data, a.count, b.data, b.count, &entry->handle);
    else if (operation == NEOCLR_TLS_SERVER)
        status = neoclr_tls_context_server_config(context->provider, a.data, a.count, b.data, b.count, c.data, c.count, &entry->handle);
    else
        status = neoclr_tls_context_session(context->provider, configuration->handle, b.data, b.count, &entry->handle);
    if (status) {
        free(entry);
        drop_empty_context(context);
        *result = status;
        return 0;
    }
    status = neoclr_gc_resource_attach_v1(heap, owner, &entry->registration, entry, cleanup);
    if (status) {
        neoclr_tls_context_release(context->provider, entry->handle);
        free(entry);
        drop_empty_context(context);
        return status;
    }
    entry->context = context;
    entry->owner = owner;
    entry->next = context->owners;
    context->owners = entry;
    *result = 0;
    return 0;
}

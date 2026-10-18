/* Native fixture only: known C callback, not Raven delegate dispatch. */
static uint64_t *churn_create(neoclr_aot_context *c, unsigned length) {
    uint64_t *callback = allocate(c, 24, NEOCLR_GC_OBJECT);
    callback[0] = UINT32_MAX;
    uint64_t *tail = NULL;
    for (unsigned i = 0; i < length; i++) {
        uint64_t *node = allocate(c, 24, NEOCLR_GC_OBJECT);
        node[2] = i;
        if (tail) tail[1] = (uintptr_t)node;
        else callback[1] = (uintptr_t)node;
        tail = node;
    }
    callback[2] = (uintptr_t)tail;
    return callback;
}
static void churn_invoke(neoclr_aot_context *c, uint64_t *callback, unsigned count,
    uint64_t *retired, uint64_t *next, uint64_t *checksum) {
    for (unsigned i = 0; i < count; i++) {
        uint64_t *head = (void *)(uintptr_t)callback[1];
        uint64_t *tail = (void *)(uintptr_t)callback[2];
        assert(head && head[2] == *retired);
        *checksum = *checksum * UINT64_C(6364136223846793005) + head[2];
        (*retired)++;
        uint64_t *fresh = allocate(c, 24, NEOCLR_GC_OBJECT);
        fresh[2] = (*next)++;
        uint64_t successor = head[1];
        if (successor) {
            tail[1] = (uintptr_t)fresh;
            callback[1] = successor;
        } else callback[1] = (uintptr_t)fresh;
        callback[2] = (uintptr_t)fresh;
        head[1] = 0; /* Retired objects must not keep the live window linked. */
    }
}
static void churn_check(const uint64_t *callback, unsigned length, uint64_t retired) {
    const uint64_t *node = (void *)(uintptr_t)callback[1], *tail = NULL;
    for (unsigned i = 0; i < length; i++) {
        assert(node && node[2] == retired + i);
        tail = node;
        node = (void *)(uintptr_t)node[1];
    }
    assert(!node && (uintptr_t)tail == callback[2]);
}

/* Owned parked frames form a conditional graph, not an unconditional root set. */
static void parked_case(gc_worker *worker, unsigned mode) {
    size_t size = (size_t)sysconf(_SC_PAGESIZE);
    void *heap = mmap(NULL, size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANON, -1, 0);
    typedef struct {
        neoclr_probe_frame frames[2];
        neoclr_probe_storage slots[2];
        uint64_t words[2];
    } frame_storage;
    frame_storage *storage = mmap(NULL, size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANON, -1, 0);
    assert(heap != MAP_FAILED && (void *)storage != MAP_FAILED && sizeof(*storage) <= size);
    neoclr_aot_context c = {.text = {heap, size, 0}};
    void *a = allocate(&c, 8, NEOCLR_GC_BYTES);
    void *b = allocate(&c, 8, NEOCLR_GC_BYTES);
    void *value = allocate(&c, 8, NEOCLR_GC_BYTES);
    void *dead = allocate(&c, 8, NEOCLR_GC_BYTES);
    neoclr_parked_roots records[2] = {0};
    storage->words[0] = (uintptr_t)b;
    storage->words[1] = (uintptr_t)value + 1; /* Interior references remain roots. */
    for (unsigned i = 0; i < 2; i++) {
        storage->slots[i] = (neoclr_probe_storage){&storage->words[i], 8, 0};
        neoclr_probe_enter_v3(&storage->frames[i], &c, 0, &storage->slots[i], 1, "{}", 2);
        if (mode == 2 && i == 0) neoclr_roots_park_v1(&records[i]);
        else assert(!neoclr_gc_roots_park_owned_v1(&c, i ? b : a, &records[i]));
    }
    uint64_t handle = 0;
    if (mode == 1) assert(!neoclr_gc_host_root_create_v1(&c, a, &handle));
    snapshot *s = malloc(sizeof(*s));
    assert(s && capture(s, &c) && s->parked_count == 2);
    assert(gc_worker_submit(worker, trace_snapshot, s));
    wait_entered(s);
    assert(!mprotect(heap, size, PROT_NONE));
    assert(!mprotect(storage, size, PROT_NONE));
    assert(finish(worker, s) == 0);
    assert(!mprotect(heap, size, PROT_READ | PROT_WRITE));
    assert(!mprotect(storage, size, PROT_READ | PROT_WRITE));
    assert(marked(s, a) == (mode == 1));
    assert(marked(s, b) == (mode != 0) && marked(s, value) == (mode != 0));
    assert(!marked(s, dead));
    assert(!neoclr_gc_collect_v1(&c, NULL));
    assert((bool)validated_bases(&c.text, a, NULL) == (mode == 1));
    assert((bool)validated_bases(&c.text, b, NULL) == (mode != 0));
    assert((bool)validated_bases(&c.text, value, NULL) == (mode != 0));
    assert(!validated_bases(&c.text, dead, NULL));
    assert((bool)records[1].expired == (mode == 0));
    free(s->copy.text.data);
    if (mode == 0) {
        assert(records[0].expired && capture(s, &c) && !s->parked_count);
        assert(gc_worker_submit(worker, trace_snapshot, s));
        assert(finish(worker, s) == 0);
        free(s->copy.text.data);
    }
    for (unsigned i = 0; i < 2; i++) neoclr_roots_discard_v1(&records[i]);
    if (handle) assert(!neoclr_gc_host_root_release_v1(&c, handle));
    assert(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    free(s);
    assert(!munmap(heap, size) && !munmap(storage, size));
}
static void parked_limits(void) {
    uint64_t heap[16] = {0};
    neoclr_aot_context c = {.text = {(void *)heap, sizeof(heap), 0}};
    neoclr_parked_roots records[SNAPSHOT_PARKED + 1] = {0};
    for (unsigned i = 0; i <= SNAPSHOT_PARKED; i++) neoclr_roots_park_v1(&records[i]);
    snapshot *s = malloc(sizeof(*s));
    assert(s && !capture(s, &c) && !s->copy.text.data);
    for (unsigned i = 0; i <= SNAPSHOT_PARKED; i++) neoclr_roots_discard_v1(&records[i]);
    free(s);
}

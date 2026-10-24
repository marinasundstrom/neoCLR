/* Reads only through the mutation gate; allocation headers are detached. */
static int live_read(shadow_snapshot *s, const block *b, uint64_t offset, void *out, size_t bytes) {
    size_t i = (size_t)(b - s->headers);
    return !gate_read(s->gate, s->epoch,
        (void *)(s->base + s->offsets[i] + sizeof(block)), b->bytes, offset, out, bytes);
}
static int live_words(shadow_snapshot *s, const block *b, uint64_t start, uint64_t end) {
    uint64_t scratch[GATE_SCRATCH / 8];
    while (start < end) {
        size_t bytes = end - start > sizeof(scratch) ? sizeof(scratch) : (size_t)(end-start);
        if (!live_read(s,b,start,scratch,bytes)) return 0;
        s->payload_read += bytes;
        for (size_t i = 0; i < bytes / 8; i++) shadow_mark(s,scratch[i]);
        start += bytes;
    }
    return 1;
}
static int shadow_scan(shadow_snapshot *s, const block *b) {
    if (b->kind == NEOCLR_GC_INTERN)
        return b->bytes == 8 && live_words(s,b,0,8);
    if (b->kind == NEOCLR_GC_OBJECT)
        return live_words(s,b,8,b->bytes & ~UINT64_C(7));
    if (b->kind == NEOCLR_GC_RECORDS) {
        uint64_t header[3];
        if (b->bytes < 24 || !live_read(s,b,0,header,24)) return 0;
        uint64_t kind=header[0], length=header[1], lanes=header[2];
        if ((kind & UINT64_C(0xffffffff)) != UINT64_C(0x80000005) ||
            (kind >> 32) > NEOCLR_NATIVE_TYPE_LIMIT || ((kind >> 32) && lanes != 1) ||
            !lanes || lanes > NEOCLR_NATIVE_LAYOUT_LANES || length > 65536 ||
            length > (b->bytes-24)/(lanes*8+1)) return 0;
        uint64_t markers=24+length*lanes*8;
        for (uint64_t i=0;i<length;i++) {
            unsigned char initialized;
            if (!live_read(s,b,markers+i,&initialized,1)) return 0;
            if (initialized && !live_words(s,b,24+i*lanes*8,24+(i+1)*lanes*8)) return 0;
        }
    } else if (b->kind == NEOCLR_GC_STRINGS) {
        uint64_t header[2];
        if (b->bytes < 16 || !live_read(s,b,0,header,16)) return 0;
        uint64_t kind=header[0]&UINT64_C(0xffffffff), length=header[1];
        int reserved=kind==UINT64_C(0x80000004);
        if ((!reserved && kind!=UINT64_C(0x80000003)) ||
            length>(b->bytes-16)/(reserved?9:8)) return 0;
        if (!reserved) return live_words(s,b,16,16+length*8);
        for (uint64_t i=0;i<length;i++) {
            unsigned char initialized;
            if (!live_read(s,b,16+length*8+i,&initialized,1)) return 0;
            if (initialized && !live_words(s,b,16+i*8,24+i*8)) return 0;
        }
    }
    return 1;
}

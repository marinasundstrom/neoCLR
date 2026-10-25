"""Explicit admitted-writer live-heap candidate, never automatic production GC."""
import argparse
from pathlib import Path
import subprocess
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();base=Path(__file__).resolve().parent
subprocess.run(['python3',base.parent/'concurrent-remark/generate.py','--output',a.output],check=True)
s=a.output.read_text()
def replace(old,new,count=1):
    global s
    assert s.count(old)==count, (old[:80],s.count(old),count)
    s=s.replace(old,new)
s='#include "gate.h"\n#if defined(NEOCLR_WORKER_GC) && !defined(NEOCLR_WORKER_READ_EPOCHS)\n#error Live shared heaps require the explicit read-epoch pool protocol\n#endif\n'+s
# Optional persistent index belongs to the owned heap, not candidate/marker state.
replace('    const void *bases[OWNED_BASE_CACHE_SIZE];', """    const void *bases[OWNED_BASE_CACHE_SIZE];
#ifdef NEOCLR_LIVE_ALLOCATION_INDEX
    uint64_t *starts, *start_summary, start_words, start_summary_words;
#endif""")
replace('/* Positive entries only:', (base/'allocation-index.h').read_text()+'\n/* Positive entries only:')
replace('static int known_base(const void *value) {', """#ifdef NEOCLR_LIVE_INLINE_BASE
static inline __attribute__((always_inline)) int known_base(const void *value)
#else
static int known_base(const void *value)
#endif
{""")
replace('    return !value || owned_heap.bases[base_slot(value)] == value;', """#ifdef NEOCLR_LIVE_EXACT_INDEX
    if (!value) return 1;
#ifndef NEOCLR_LIVE_DIRECT_EXACT_INDEX
    if (owned_heap.bases[base_slot(value)] == value) return 1;
#endif
    uintptr_t address=(uintptr_t)value, base=(uintptr_t)owned_heap.data;
    if (address<base || address-base>=owned_heap.used || ((address-base)&7)) return 0;
    uint64_t bit=(address-base)/8;
    return !!(owned_heap.starts[bit/64] & (UINT64_C(1)<<(bit%64)));
#else
    return !value || owned_heap.bases[base_slot(value)] == value;
#endif""")
replace('    owned_heap.arena = arena;', """#ifdef NEOCLR_LIVE_ALLOCATION_INDEX
    if (!owned_index_create(arena->capacity)) return 3;
#endif
    owned_heap.arena = arena;""")
replace('    memset(&owned_heap, 0, sizeof(owned_heap));', """#ifdef NEOCLR_LIVE_ALLOCATION_INDEX
    owned_index_destroy();
#endif
    memset(&owned_heap, 0, sizeof(owned_heap));""")
replace('    if (owned) remember_base(chosen + 1);', """    if (owned) remember_base(chosen + 1);
#ifdef NEOCLR_LIVE_ALLOCATION_INDEX
    if (owned) owned_index_set((unsigned char *)(chosen+1)-a->data);
#endif""")
replace('                unsigned slot = base_slot(b + 1);', """#ifdef NEOCLR_LIVE_ALLOCATION_INDEX
                owned_index_clear((unsigned char *)(b+1)-a->data);
#endif
                unsigned slot = base_slot(b + 1);""")
replace('    uint64_t used, *offsets, *pending;', '''    uint64_t used, *offsets, *pending;
    block *headers;
    mutation_gate *gate;
    uint64_t epoch, payload_read, barrier_words;
#ifdef NEOCLR_LIVE_BATCH
    unsigned read_held, read_bytes, read_calls;
    uint64_t read_acquisitions, max_read_bytes, max_read_calls;
    uint64_t hold_start, hold_max, hold_total;
#endif''')
replace('static _Thread_local uint64_t shadow_stats[5];', 'static _Thread_local uint64_t live_counts[5], live_batch_counts[5], live_time_counts[5];\nstatic _Thread_local uint64_t shadow_stats[5];')
# Keep the predicate identical; expose its loads to the caller's optimizer.
# The qualified candidate uses Clang/GCC; ordinary native GC is unchanged.
replace('static int block_valid(const neoclr_aot_text_arena *a, uint64_t at) {', '\n'.join([
    '#ifdef NEOCLR_LIVE_INLINE_VALIDATION',
    'static inline __attribute__((always_inline)) int block_valid(const neoclr_aot_text_arena *a, uint64_t at)',
    '#else',
    'static int block_valid(const neoclr_aot_text_arena *a, uint64_t at)',
    '#endif', '{']))
# All descriptor accesses use compact detached headers.
replace('(const void *)(s->data + s->offsets[i])','&s->headers[i]',5)
# Optional monotonic old-header validation; the default indexed proof is retained.
start=s.index('    for (size_t i = 0; i < s->count; i++) {',s.index('static int candidate_seed('))
end=s.index('    size_t previous = 0;',start)
original=s[start:end]
s=s[:start]+'#ifdef NEOCLR_LIVE_LINEAR_SEED\n    if (candidate_validate_old_linear(a,s)) return 3;\n#else\n'+original+'#endif\n'+s[end:]
replace('static int candidate_seed(', (base/'seed-linear.h').read_text()+'\nstatic int candidate_seed(')

replace('''    s->data = shadow_alloc(s->used, 1);
    if (s->used) memcpy(s->data, context->text.data, s->used);''','''    s->gate = shadow_alloc(1, sizeof(*s->gate));
    if (gate_init(s->gate) || gate_begin(s->gate, &s->epoch)) abort();''')
replace('const block *b = (const void *)(s->data + at);','const block *b = (const void *)(context->text.data + at);',2)
replace('    s->offsets = shadow_alloc(s->count, sizeof(uint64_t));','    s->headers = shadow_alloc(s->count, sizeof(block));\n    s->offsets = shadow_alloc(s->count, sizeof(uint64_t));')
replace('        s->offsets[n++] = at;','        s->headers[n] = *b;\n        s->offsets[n++] = at;')
replace('    shadow_stats[1] += s->used;','    shadow_stats[1] += s->count * sizeof(block);')
replace('    free(s->edges); free(s->pending); free(s->offsets);','''    gate_stop(s->gate);
    if (gate_destroy(s->gate)) abort();
#ifdef NEOCLR_LIVE_BATCH
    live_batch_counts[0]+=s->read_acquisitions;
    if (s->max_read_bytes>live_batch_counts[1]) live_batch_counts[1]=s->max_read_bytes;
    if (s->max_read_calls>live_batch_counts[2]) live_batch_counts[2]=s->max_read_calls;
    if (s->hold_max>live_batch_counts[3]) live_batch_counts[3]=s->hold_max;
    live_batch_counts[4]+=s->hold_total;
#endif
    live_time_counts[0] += s->cpu_ns;
    live_counts[0] += s->count * sizeof(block);
    live_counts[1] += s->payload_read;
    live_counts[2] += s->barrier_words;
    live_counts[3] += !!s->valid;
    live_counts[4] += !s->valid;
    free(s->gate); free(s->headers);
    free(s->edges); free(s->pending); free(s->offsets);''',2)
# Every new allocation is retained at remark, including storage named by an
# unresolved conservative word. No separate future-address vector is needed.
replace('        s->valid = 1;', '        /* Validity persists across worker passes. */')
replace('    s->base = (uintptr_t)context->text.data;', '    s->valid = 1;\n    s->base = (uintptr_t)context->text.data;')
replace('s->cpu_ns = shadow_clock(CLOCK_THREAD_CPUTIME_ID) - start;', 's->cpu_ns += shadow_clock(CLOCK_THREAD_CPUTIME_ID) - start;')
# Each worker pass drains any available barrier words before deciding it is done.
replace('''            if (!s->pending_count) break;
        }
        s->cpu_ns''','''            uint64_t words[GATE_WORDS]; size_t count;
            if (gate_drain(s->gate,s->epoch,words,GATE_WORDS,&count)) { s->valid=0; break; }
            s->barrier_words += count;
            for (size_t i=0;i<count;i++) shadow_mark(s,words[i]);
            if (!s->pending_count) break;
        }
        s->cpu_ns''')
replace('/* Generated scanner below shares the production payload semantics. */',
        (base/'batch.h').read_text()+'\n/* Generated scanner below shares the production payload semantics. */')
replace('''            while (s->pending_count) {
                size_t i = s->pending[--s->pending_count];
                if (!shadow_scan(s, &s->headers[i])) {
                    s->valid = 0;
                    break;
                }
            }''', '''            while (s->pending_count) {
#ifdef NEOCLR_LIVE_BATCH
                unsigned objects=0;
                while (s->pending_count && objects++<32) {
#endif
                    size_t i = s->pending[--s->pending_count];
                    if (!shadow_scan(s, &s->headers[i])) {
                        s->valid = 0;
                        break;
                    }
#ifdef NEOCLR_LIVE_BATCH
                }
                live_batch_leave(s);
                if (!s->valid || !live_batch_drain(s)) { s->valid=0; break; }
#endif
            }''')
start=s.index('static int shadow_scan(shadow_snapshot *s, const block *b) {')
end=s.index('/* Explicit experimental lifecycle;',start)
s=s[:start]+(base/'scan.h').read_text()+'\n'+s[end:]
replace('? (const void *)(s->data + at) : NULL;', '? &s->headers[previous] : NULL;')
start=s.index('            int changed = !existed || memcmp(')
end=s.index('\n        }\n        at += b->span;',start)
s=s[:start]+'''            if (!existed) {
                /* New allocations are not in the detached index. Trace their
                 * completed current payloads on the owner at remark. */
                mark(a,index,(uintptr_t)(b+1),pending);
            } else if (s->marked[previous] && !(b->state & MARKED)) {
                b->state=ALLOCATED|MARKED|SCANNED;
            }'''+s[end:]
# The seed walk already holds a validated allocated header. Reuse the canonical
# mark transition, including zero-byte exclusion and atomic fixed-point counts.
mark_start=s.index('static void mark(')
mark_end=s.index('\nstatic void mark_slots(',mark_start)
mark_body=s[mark_start:mark_end]
transition=mark_body[mark_body.index('        LOOKUP_COUNT(newly_marked);'):]
assert transition.endswith('    }\n}')
transition=transition[:-len('    }\n}')]
helper='#ifdef NEOCLR_LIVE_DIRECT_SEED\nstatic void candidate_mark_validated(block *b, block_index *index, block **pending) {\n    LOOKUP_COUNT(mark_candidates);\n    if (b->bytes) LOOKUP_COUNT(payload_matches);\n    if (b->bytes && !(b->state & MARKED)) {\n'+transition+'    }\n}\n#endif\n'
replace('static int candidate_seed(', helper+'static int candidate_seed(')
replace('                mark(a,index,(uintptr_t)(b+1),pending);',
    '#ifdef NEOCLR_LIVE_DIRECT_SEED\n                candidate_mark_validated(b,index,pending);\n#else\n                mark(a,index,(uintptr_t)(b+1),pending);\n#endif')
# Finish drains after mutations stop; only newly discovered old objects need
# another worker pass. Never lose barriers published after the first pass idles.
replace('''    candidate_wait();
    candidate_phase = 2;''','''    candidate_wait();
    shadow_worker *w=shadow_owner;
    shadow_snapshot *s=w->job;
    uint64_t words[GATE_WORDS]; size_t barrier_count=0;
    if (gate_drain(s->gate,s->epoch,words,GATE_WORDS,&barrier_count)) s->valid=0;
    else {
        s->barrier_words += barrier_count;
        for (size_t i=0;i<barrier_count;i++) shadow_mark(s,words[i]);
    }
    if (s->valid && s->pending_count) {
        if (pthread_mutex_lock(&w->mutex)) abort();
        w->done=0; w->ready=1;
        if (pthread_cond_broadcast(&w->condition) || pthread_mutex_unlock(&w->mutex)) abort();
        candidate_wait();
    }
    candidate_phase = 2;''')
s+='''
void experiment_gc_time_counts(uint64_t *out) { memcpy(out,live_time_counts,sizeof(live_time_counts)); }
void experiment_gc_batch_counts(uint64_t *out) { memcpy(out,live_batch_counts,sizeof(live_batch_counts)); }
void experiment_gc_live_counts(uint64_t *out) { memcpy(out,live_counts,sizeof(live_counts)); }
void experiment_gc_live_wait(void) { if (!candidate_active) abort(); candidate_wait(); }
/* Raw native caller must admit every mutation through this adapter. This is not
 * sufficient admission for generated code, providers, or shared heaps. */
static int live_write_known(void *payload, uint64_t bytes, uint32_t kind,
    const mutation_patch *patches, size_t count) {
    shadow_snapshot *s=shadow_owner->job;
    /* Published array shape is immutable. Markers and payload use grouped patches. */
    size_t header=kind==NEOCLR_GC_RECORDS?24:kind==NEOCLR_GC_STRINGS?16:0;
    if (count && !patches) return GATE_INVALID;
    if (count > GATE_PATCHES) return GATE_INVALID;
    for (size_t i=0;i<count;i++)
        if (patches[i].length && patches[i].offset < header) return GATE_INVALID;
#ifdef NEOCLR_LIVE_GATE_PROFILE
    uint64_t start=shadow_clock(CLOCK_MONOTONIC), cpu=shadow_clock(CLOCK_THREAD_CPUTIME_ID);
#endif
    int result=gate_write(s->gate,payload,bytes,kind==NEOCLR_GC_OBJECT?8:0,patches,count);
#ifdef NEOCLR_LIVE_FINISH_WATERMARK
    /* Do not collect inside a store: roots/results may not be published yet.
     * Request the existing slow path at the next qualified publication. */
    if (result==GATE_OK && s->gate->finish_requested)
        gc_state.pacing.boundaries=NEOCLR_GC_PUBLICATION_BUDGET;
#endif
#ifdef NEOCLR_LIVE_GATE_PROFILE
    live_time_counts[4]+=shadow_clock(CLOCK_THREAD_CPUTIME_ID)-cpu;
    uint64_t elapsed=shadow_clock(CLOCK_MONOTONIC)-start;
    live_time_counts[1]+=elapsed;
    if (elapsed>live_time_counts[2]) live_time_counts[2]=elapsed;
    live_time_counts[3]++;
#endif
    return result;
}
int experiment_gc_write(neoclr_aot_context *context, void *payload,
    const mutation_patch *patches, size_t count) {
    if (!candidate_active || context != candidate_context) return GATE_INVALID;
    uint64_t bytes; uint32_t kind;
    if (neoclr_gc_allocation_info_v1(context,payload,&bytes,&kind)) return GATE_INVALID;
    return live_write_known(payload,bytes,kind,patches,count);
}
'''
replace('No collection or arena\n * release is permitted between begin and finish.', 'Published collection completes the epoch; arena\n * release remains forbidden until completion.')
# A completed capture establishes the pacing baseline for this owned epoch.
replace('    else candidate_active = 1;', '    else {\n        candidate_active = 1;\n        gc_state.pacing.context=context;\n        gc_state.pacing.allocations=gc_state.statistics.allocations;\n        gc_state.pacing.collections=gc_state.statistics.collections;\n        gc_state.pacing.boundaries=0;\n    }')
# A published managed boundary may complete the active epoch before continuing.
# The phase guard prevents recursion during finish's own collection request.
replace('static int32_t collect_conditional(neoclr_aot_context *context,',
    'int32_t experiment_gc_finish(neoclr_aot_context *, const neoclr_probe_frame *, neoclr_gc_conditional_root *, uint32_t);\nstatic int32_t collect_conditional(neoclr_aot_context *context,')
boundary='int32_t neoclr_gc_collect_conditional_v1(neoclr_aot_context *context,\n    const neoclr_probe_frame *head, neoclr_gc_conditional_root *conditional, uint32_t count) {'
replace(boundary,boundary+'\n    if (candidate_active && !candidate_phase && context == candidate_context && !gc_state.cleanup_active)\n        return experiment_gc_finish(context,head,conditional,count);')
s+='\nint experiment_gc_live_active(neoclr_aot_context *context) { return candidate_active && candidate_context==context; }\n'
replace('static _Thread_local neoclr_aot_context *candidate_context;', '')
state=(base/'state.h').read_text()
replace('static _Thread_local int candidate_active;', 'typedef struct shadow_worker shadow_worker;\n'+state)
replace('typedef struct {\n    pthread_t thread;', 'struct shadow_worker {\n    pthread_t thread;')
replace('} shadow_worker;', '};')
replace('static _Thread_local shadow_worker *shadow_owner;', '#ifndef NEOCLR_LIVE_GROUP_WORKER_TLS\nstatic _Thread_local shadow_worker *shadow_owner;\n#endif')
s+='\n'+(base/'store.h').read_text()
s+='\n'+(base/'contract.h').read_text()
s+='\n'+(base/'workers.h').read_text()
# Reuse only detached per-job arrays, after the worker completion handshake.
# Keep ordinary allocation as the control; gates and snapshots retain their lifetimes.
position=s.index('static void *shadow_alloc(')
position=s.index('\n}',position)+2
s=s[:position]+'\n'+(base/'scratch.h').read_text()+s[position:]
for slot, (field, count, width) in enumerate([
    ('headers','s->count','sizeof(block)'), ('offsets','s->count','sizeof(uint64_t)'),
    ('pending','s->count','sizeof(uint64_t)'), ('marked','s->count','1'),
    ('expected','s->count','1'), ('starts','s->index_capacity','sizeof(size_t)')]):
    replace(f's->{field} = shadow_alloc({count}, {width});',
            f's->{field} = live_scratch_alloc({slot}, {count}, {width});')
    replace(f'free(s->{field});', f'live_scratch_free({slot}, s->{field});', 2)
replace('    shadow_owner = NULL;', '    shadow_owner = NULL;\n    live_scratch_stop();')
# Allocation-index helpers occur before the later snapshot headers.
a.output.write_text('#include <stdlib.h>\n'+s)

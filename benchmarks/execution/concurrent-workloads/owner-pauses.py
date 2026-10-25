"""Instrument owned GC operations in a copied collector; diagnostic-only wall times."""
import argparse
from pathlib import Path
import re

HEADER = r'''
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#ifndef OWNER_PAUSE_NOW
static uint64_t owner_pause_now(void) {
    struct timespec t;
    if (clock_gettime(CLOCK_MONOTONIC, &t)) abort();
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + (uint64_t)t.tv_nsec;
}
#define OWNER_PAUSE_NOW() owner_pause_now()
#endif
/* Owner TLS only. Phase histograms overlap; row 5 counts all outermost operations; row 6 excludes guarded writes. */
typedef struct { uint64_t count, total, maximum, failures, bins[64]; } owner_pause_row;
static _Thread_local owner_pause_row owner_pauses[7];
static _Thread_local uint32_t owner_pause_depth;
static _Thread_local uint64_t owner_pause_outer;
static _Thread_local unsigned owner_pause_outer_kind;
static void owner_pause_record(unsigned kind, uint64_t ns, int failed) {
    owner_pause_row *r = &owner_pauses[kind];
    if (r->count == UINT64_MAX || ns > UINT64_MAX-r->total) abort();
    r->count++; r->total += ns; r->failures += failed != 0;
    if (ns > r->maximum) r->maximum = ns;
    unsigned bucket = 0;
    for (uint64_t n=ns; n>1; n>>=1) bucket++;
    r->bins[bucket]++;
}
static uint64_t owner_pause_enter(unsigned kind) {
    uint64_t start = OWNER_PAUSE_NOW();
    if (owner_pause_depth == UINT32_MAX) abort();
    if (!owner_pause_depth++) { owner_pause_outer = start; owner_pause_outer_kind = kind; }
    return start;
}
static void owner_pause_leave(unsigned kind, uint64_t start, int failed) {
    uint64_t end = OWNER_PAUSE_NOW();
    if (!owner_pause_depth || end < start) abort();
    owner_pause_record(kind, end-start, failed);
    if (!--owner_pause_depth) {
        if (end < owner_pause_outer) abort();
        owner_pause_record(5, end-owner_pause_outer, failed);
        if (owner_pause_outer_kind != 4) owner_pause_record(6, end-owner_pause_outer, failed);
    }
}
void experiment_owner_pauses_report(void) {
    if (owner_pause_depth) abort();
    for (unsigned i=0;i<7;i++) {
        owner_pause_row *r=&owner_pauses[i];
        fprintf(stderr,"OWNER_PAUSE %u %llu %llu %llu %llu",i,
            (unsigned long long)r->count,(unsigned long long)r->total,
            (unsigned long long)r->maximum,(unsigned long long)r->failures);
        for (unsigned j=0;j<64;j++) fprintf(stderr," %llu",(unsigned long long)r->bins[j]);
        fprintf(stderr,"\n");
    }
}
'''

def instrument(source, live):
    assert 'owner_pause_impl_' not in source
    specs = [('neoclr_gc_collect_conditional_v1', 'context, head, conditional, count', 0, False)]
    if live:
        specs += [('experiment_gc_begin', 'context, head, conditional, count', 1, False),
                  ('experiment_gc_finish', 'context, head, conditional, count', 2, False),
                  ('candidate_cancel', '', 3, True),
                  ('live_write_known', 'payload, bytes, kind, patches, count', 4, False)]
    wrappers = []
    for name, args, kind, void in specs:
        pattern = re.compile(r'(?m)^((?:static )?(?:int32_t|int|void) )' + name + r'(\([^;{}]*\)) \{')
        matches = list(pattern.finditer(source))
        assert len(matches) == 1, (name, len(matches))
        match = matches[0]
        signature = match[1] + name + match[2]
        replacement = signature + ';\n' + match[1] + 'owner_pause_impl_' + name + match[2] + ' {'
        source = source[:match.start()] + replacement + source[match.end():]
        call = 'owner_pause_impl_' + name + '(' + args + ')'
        body = (call + ';\n    owner_pause_leave(%d, start, 0);' % kind if void else
                'int32_t result = ' + call + ';\n    owner_pause_leave(%d, start, result != 0);\n    return result;' % kind)
        wrappers.append(signature + ' {\n    uint64_t start = owner_pause_enter(%d);\n    ' % kind + body + '\n}\n')
    return HEADER + '\n' + source + '\n' + '\n'.join(wrappers)

if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--source', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--live', action='store_true')
    a = p.parse_args()
    a.output.write_text(instrument(a.source.read_text(), a.live))

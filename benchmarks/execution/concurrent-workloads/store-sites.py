"""Add bounded owner-thread call-site counts to a generated live collector.

AOT macOS diagnostic only: no timing claim, no managed allocation, no GC mutation.
Reports offsets from the executable Mach-O header to remove ASLR.
"""
import argparse
from pathlib import Path
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--source', type=Path, required=True)
a = p.parse_args()
code = a.source.read_text()
anchor = 'int experiment_gc_store(neoclr_aot_context *context, void *destination,'
assert code.count(anchor) in [1, 2]
helpers = r'''
#include <stdio.h>
extern const char _mh_execute_header;
#define STORE_SITE_CAPACITY 16384
static _Thread_local struct { uintptr_t pc; uint64_t count; } store_sites[STORE_SITE_CAPACITY];
static _Thread_local uint64_t store_site_overflow;
static void store_site_note(uintptr_t pc) {
    size_t slot=(pc>>2)%STORE_SITE_CAPACITY;
    for(unsigned i=0;i<STORE_SITE_CAPACITY;i++) {
        if (!store_sites[slot].pc || store_sites[slot].pc==pc) {
            store_sites[slot].pc=pc;store_sites[slot].count++;return;
        }
        slot=(slot+1)%STORE_SITE_CAPACITY;
    }
    store_site_overflow++;
}
void experiment_store_sites_report(void) {
    uint64_t total=0;
    for(unsigned i=0;i<STORE_SITE_CAPACITY;i++) if(store_sites[i].pc) {
        total+=store_sites[i].count;
        fprintf(stderr,"STORE_SITE %llx %llu\n",
            (unsigned long long)(store_sites[i].pc-(uintptr_t)&_mh_execute_header),
            (unsigned long long)store_sites[i].count);
    }
    fprintf(stderr,"STORE_SITE_TOTAL %llu %llu\n",(unsigned long long)total,
        (unsigned long long)store_site_overflow);
}
'''
code = '#ifdef NEOCLR_LIVE_STORE_HANDLE\n#error Store-site profiling requires ordinary checked stores\n#endif\n' + code.replace(anchor, helpers + '\n' + anchor, 1)
anchor = 'live_store_counts[0]++;'
assert code.count(anchor) == 1
code = code.replace(anchor, anchor + '\n        store_site_note((uintptr_t)__builtin_return_address(0));')
a.source.write_text(code)

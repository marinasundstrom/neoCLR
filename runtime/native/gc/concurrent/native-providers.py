"""Generate explicitly admitted live-GC native providers; production stays unchanged."""
import argparse
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args()
native=Path(__file__).resolve().parent.parent.parent
s=(native/'memory/text-arena.c').read_text()
old='        table[type + 1] = (uintptr_t)storage;'
assert s.count(old)==1
s=s.replace(old,'''        /* No collection between allocation and guarded publication: storage is
         * not yet rooted except by this native temporary. */
        status = experiment_gc_store(context, table + type + 1, (uintptr_t)storage, 8);
        if (status) return status;''')
old='int32_t neoclr_static_storage_rooted_v1('
assert s.count(old)==1
s=s.replace(old,'int32_t experiment_gc_static_storage_rooted_v1(')
s=s.replace('#include "text-arena.h"','#include "text-arena.h"\nextern int experiment_gc_store(neoclr_aot_context *, void *, uint64_t, uint32_t);',1)
a.output.write_text(s)

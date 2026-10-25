"""Check shared owned-heap allocation/publication budgets and forced slow paths."""
import json,subprocess,tempfile
from pathlib import Path
root=Path(__file__).resolve().parents[3]
recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
source=r'''
#include "execution-state.h"
#include <assert.h>
static unsigned calls;
int32_t neoclr_gc_paced_collect_v1(neoclr_aot_context *c,const neoclr_probe_frame *f) {
    assert(c && !f);calls++;return 17;
}
int main(void) {
    neoclr_execution_state state={0};neoclr_aot_context c={0};
    state.gc.pacing.context=&c;state.gc.pacing.collections=7;state.gc.statistics.collections=7;
    for(unsigned i=1;i<NEOCLR_GC_PUBLICATION_BUDGET;i++) assert(!neoclr_paced_boundary(&state,&c,0));
    assert(neoclr_paced_boundary(&state,&c,0)==17 && calls==1);
    state.gc.pacing.boundaries=0;
    state.gc.statistics.allocations=NEOCLR_GC_ALLOCATION_BUDGET-1;
    assert(!neoclr_paced_boundary(&state,&c,0));
    state.gc.statistics.allocations++;
    assert(neoclr_paced_boundary(&state,&c,0)==17 && calls==2);
    state.gc.statistics.allocations=0;state.gc.pacing.context=0;
    assert(neoclr_paced_boundary(&state,&c,0)==17 && calls==3);
    state.gc.pacing.context=&c;state.gc.statistics.collections++;
    assert(neoclr_paced_boundary(&state,&c,0)==17 && calls==4);
}
'''
with tempfile.TemporaryDirectory() as tmp:
    c=Path(tmp)/'test.c';exe=Path(tmp)/'test';c.write_text(source)
    common=recipe[:recipe.index('-std=c11')]+['-std=c11','-O2','-Wall','-Wextra','-Werror','-DNEOCLR_GC_PROFILE','-I',str(root/'runtime/native/gc'),'-fsanitize=undefined,bounds','-fno-sanitize-recover=all',str(c),'-o',str(exe)]
    for allocation,publication in [(256,4096),(1024,16384),(3,4)]:
        subprocess.run(common+[f'-DNEOCLR_GC_ALLOCATION_BUDGET={allocation}',f'-DNEOCLR_GC_PUBLICATION_BUDGET={publication}'],check=True)
        subprocess.run([exe],check=True)
    bad=subprocess.run(common+['-DNEOCLR_GC_ALLOCATION_BUDGET=0'],capture_output=True,text=True)
    assert bad.returncode!=0 and 'GC pacing budgets must be positive' in bad.stderr
print('PASS default/custom thresholds, epoch/context slow paths, status propagation, invalid budget; UB/bounds')

"""Exercise a watermark-triggered finish only at a published root boundary."""
import json,subprocess,tempfile,signal
from pathlib import Path
base=Path(__file__).resolve().parent;root=base.parents[2]
recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
main=r'''
#include <assert.h>
#include <stdio.h>
int main(void) {
 uint64_t heap[512]={0};neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
 assert(!neoclr_gc_heap_acquire_v1(&c.text));
 uint64_t *root,*child;
 assert(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,(void **)&root));
 assert(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,(void **)&child));
 root[0]=17;child[0]=17;root[1]=(uintptr_t)child;
 neoclr_probe_storage slot={&root,8,0};
 neoclr_probe_frame frame={.context=&c,.storage=&slot,.storage_count=1};
 experiment_gc_shadow_gate(1);
 assert(!experiment_gc_begin(&c,&frame,0,0));
 mutation_gate *g=shadow_owner->job->gate;
 uint64_t zero=0;mutation_patch patch={8,8,&zero};
 for(unsigned i=0;i<15;i++)assert(!experiment_gc_write(&c,root,&patch,1));
 assert(!g->finish_requested && gc_state.pacing.boundaries==0);
 assert(!experiment_gc_write(&c,root,&patch,1));
 assert(g->finish_requested && gc_state.pacing.boundaries==NEOCLR_GC_PUBLICATION_BUDGET);
 assert(candidate_active && candidate_counts[1]==0 && *child==17);
 /* A drain may lower occupancy, but cannot race/reset the owner-written hint. */
 uint64_t words[GATE_WORDS];size_t count=0;
 assert(!gate_drain(g,shadow_owner->job->epoch,words,GATE_WORDS,&count));
 assert(count==32 && g->finish_requested);
 for(size_t i=0;i<count;i++)shadow_mark(shadow_owner->job,words[i]);
 experiment_gc_shadow_gate(0);
 assert(!neoclr_published_boundary(&neoclr_execution_state_v1,&c,&frame));
 assert(!candidate_active && candidate_counts[1]==1 && candidate_counts[2]==0 && *child==17);
 assert(!gc_state.pacing.boundaries);
 assert(!experiment_gc_begin(&c,&frame,0,0));
 assert(!shadow_owner->job->gate->finish_requested);
 assert(!experiment_gc_finish(&c,&frame,0,0));
 assert(!neoclr_gc_heap_release_v1(&c.text));
 puts("PASS threshold, deferred completion, sticky drain, retained deletion edge, next-epoch reset");
}
'''
with tempfile.TemporaryDirectory() as d:
 p=Path(d);generated=p/'generated.c'
 subprocess.run(['python3',root/'benchmarks/execution/concurrent-live/automatic.py','--output',generated],check=True)
 original=generated.read_text();source=p/'test.c';source.write_text(original+main)
 common=recipe[:recipe.index('-std=c11')]+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DGATE_WORDS=64','-DNEOCLR_LIVE_FINISH_WATERMARK=50','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PROFILE','-DNEOCLR_SHADOW_TEST','-I',str(root/'benchmarks/execution/concurrent-mutation-gate'),'-I',str(root/'runtime/native/gc'),str(root/'benchmarks/execution/concurrent-mutation-gate/gate.c'),str(root/'runtime/native/gc/root-probe.c'),str(source)]
 for sanitize in [False,True]:
  exe=p/str(sanitize);cmd=common+['-o',str(exe)]
  if sanitize:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
  subprocess.run(cmd,check=True);subprocess.run([exe],check=True)
 # Scheduling must depend on the hint; removing publication pressure must fail.
 source.write_text(original.replace('gc_state.pacing.boundaries=NEOCLR_GC_PUBLICATION_BUDGET;','gc_state.pacing.boundaries=0;')+main)
 exe=p/'negative';subprocess.run(common+['-o',str(exe)],check=True)
 r=subprocess.run([exe],capture_output=True);assert r.returncode==-signal.SIGABRT
 print('PASS disabled hint negative control')
 for flags,message in [(['-DNEOCLR_GATE_WORKERS'],'Finish watermark requires the owner-only gate'),(['-UNEOCLR_LIVE_FINISH_WATERMARK','-DNEOCLR_LIVE_FINISH_WATERMARK=0'],'Finish watermark must be a percentage')]:
  r=subprocess.run(common+flags+['-o',str(p/'invalid')],capture_output=True,text=True)
  assert r.returncode!=0 and message in r.stderr
 print('PASS shared-writer admission and invalid threshold rejected')

"""Compare generated direct seeding against canonical mark transitions."""
import json
from pathlib import Path
import subprocess
import signal
import tempfile
base=Path(__file__).resolve().parent
root=base.parents[2]
recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
with tempfile.TemporaryDirectory() as directory:
    p=Path(directory);generated=p/'generated.c'
    subprocess.run(['python3',base/'generate.py','--output',generated],check=True)
    s=generated.read_text()
    mark=s[s.index('static void mark('):s.index('\nstatic void mark_slots(')]
    start=s.index('static void candidate_mark_validated(')
    direct=s[start:s.index('\n#endif',start)]
    prefix='''#include <stdint.h>
#include <assert.h>
#include <stdio.h>
#define NEOCLR_GC_PROFILE 1
#define LOOKUP_COUNT(x) ((void)0)
enum {ALLOCATED=1,MARKED=2,SCANNED=4,NEOCLR_GC_TEXT=1,NEOCLR_GC_BYTES=2};
typedef struct {uint64_t span,bytes,kind,state;} block;
typedef struct {uint64_t atomic_marks;} block_index;
typedef struct {block *b;} neoclr_aot_text_arena;
static block *indexed_block(neoclr_aot_text_arena *a,block_index *index,uint64_t word) {
 (void)index;(void)word;return a->b;
}
'''
    main='''
int main(void) {
 unsigned cases=0;block tail={0};
 for(unsigned kind=1;kind<=16;kind++)
 for(unsigned bytes=0;bytes<=128;bytes++)
 for(unsigned state=0;state<3;state++)
 for(unsigned linked=0;linked<2;linked++) {
  block a={160,bytes,kind,ALLOCATED},b;
  if(state==1)a.state=(uint64_t)(uintptr_t)&tail|ALLOCATED|MARKED;
  if(state==2)a.state=ALLOCATED|MARKED|SCANNED;
  b=a;block *pa=linked?&tail:NULL,*pb=pa;
  block_index ia={7},ib={7};neoclr_aot_text_arena arena={&a};
  mark(&arena,&ia,(uintptr_t)(&a+1),&pa);
  candidate_mark_validated(&b,&ib,&pb);
  assert(a.state==b.state && ia.atomic_marks==ib.atomic_marks);
  assert((pa==&a && pb==&b) || (pa!=&a && pb!=&b && pa==pb));
  cases++;
 }
 printf("PASS %u mark-transition comparisons\\n",cases);
}
'''
    c=p/'test.c';c.write_text(prefix+mark+direct+main)
    for sanitize in [False,True]:
        exe=p/str(sanitize)
        command=recipe[:recipe.index('-std=c11')]+['-std=c11','-O2','-Wall','-Wextra','-Werror']
        if sanitize:command+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
        subprocess.run(command+[str(c),'-o',str(exe)],check=True)
        subprocess.run([exe],check=True)

    # A broken atomic fixed-point count must be detected independently of liveness.
    assert direct.count('index->atomic_marks++;')==1
    c.write_text(prefix+mark+direct.replace('index->atomic_marks++;','/* broken control */')+main)
    exe=p/'negative'
    subprocess.run(recipe[:recipe.index('-std=c11')]+['-std=c11','-O2','-Wno-unused-parameter',str(c),'-o',str(exe)],check=True)
    result=subprocess.run([exe],capture_output=True,text=True)
    assert result.returncode == -signal.SIGABRT
    print('PASS broken atomic-count negative control')

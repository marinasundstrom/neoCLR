"""Bounded owner scratch reset, growth, spill and shutdown contracts."""
import json
from pathlib import Path
import signal
import subprocess
import tempfile
base=Path(__file__).resolve().parent
root=base.parents[2]
recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
prefix='''#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <assert.h>
#include <stdio.h>
#define NEOCLR_LIVE_REUSE_SCRATCH
#define NEOCLR_LIVE_SCRATCH_LIMIT 1024
static unsigned allocations;
static void *shadow_alloc(size_t count,size_t bytes) {
 if(bytes && count>SIZE_MAX/bytes) abort();
 allocations++;
 void *p=calloc(count?count:1,bytes);if(!p)abort();return p;
}
'''
main='''
int main(int argc,char **argv) {
 (void)argv;
 unsigned char *p=live_scratch_alloc(0,128,1);
 memset(p,255,128);
 if(argc==2) {live_scratch_stop();return 1;}
 if(argc==3) {live_scratch_alloc(0,1,1);return 1;}
 live_scratch_free(0,p);
 for(unsigned i=0;i<100;i++) {
  unsigned char *q=live_scratch_alloc(0,64,1);assert(p==q);
  for(unsigned n=0;n<64;n++)assert(!q[n]);
  memset(q,99,64);live_scratch_free(0,q);
 }
 assert(allocations==1 && live_scratch_retained==128);
 p=live_scratch_alloc(0,256,1);assert(allocations==2);
 for(unsigned n=0;n<256;n++)assert(!p[n]);
 live_scratch_free(0,p);
 void *q=live_scratch_alloc(1,768,1);assert(live_scratch_retained==1024);
 void *r=live_scratch_alloc(2,8,1);assert(live_scratch_retained==1024);
 live_scratch_free(2,r);live_scratch_free(1,q);
 r=live_scratch_alloc(0,2048,1);assert(live_scratch_retained==1024);
 live_scratch_free(0,r);
 assert(live_scratch[0].data==p && !live_scratch[0].leased);
 live_scratch_stop();assert(!live_scratch_retained);
 for(unsigned i=0;i<6;i++)assert(!live_scratch[i].data);
 p=live_scratch_alloc(0,0,1);assert(p && !p[0]);
 live_scratch_free(0,p);live_scratch_stop();
 printf("PASS scratch: 100 reuse cycles, growth, aggregate cap, spill, zero count, shutdown\\n");
}
'''
with tempfile.TemporaryDirectory() as d:
 p=Path(d);source=p/'test.c';source.write_text(prefix+(base.parents[2]/'runtime/native/gc/concurrent/scratch.h').read_text()+main)
 for sanitized in [False,True]:
  exe=p/str(sanitized)
  command=recipe[:recipe.index('-std=c11')]+['-std=c11','-O2','-Wall','-Wextra','-Werror']
  if sanitized:command+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
  subprocess.run(command+[str(source),'-o',str(exe)],check=True)
  subprocess.run([exe],check=True)
  for args in [['active'],['duplicate','lease']]:
   result=subprocess.run([exe,*args],capture_output=True)
   assert result.returncode==-signal.SIGABRT
 print('PASS active shutdown and duplicate lease rejected')

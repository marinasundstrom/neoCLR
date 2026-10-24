"""Route the existing loopback fixture through generated socket result wrappers."""
import argparse
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
root=Path(__file__).resolve().parents[3]
s='''#include "socket-listener.h"
#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define REQUIRE(x) do { if(!(x)) { fprintf(stderr,"socket adapter line %d: %s\\n",__LINE__,#x); abort(); } } while(0)
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern int32_t experiment_gc_store(neoclr_aot_context *,void *,uint64_t,uint32_t);
extern void experiment_gc_store_counts(uint64_t *);
static void *const *functions;
static uint64_t *result_owner, result_handle, before[5], calls[13];
static void prepare(neoclr_aot_context *c) {
    REQUIRE(!experiment_gc_store(c,result_owner+1,111,8));
    REQUIRE(!experiment_gc_store(c,result_owner+2,222,8));
    experiment_gc_store_counts(before);
}
static int finish(neoclr_aot_context *c,void *out,int status,unsigned bytes) {
    uint64_t after[5];experiment_gc_store_counts(after);
    REQUIRE(after[1]+after[2]-before[1]-before[2]==(status?0:bytes==16?2:1));
    if(status) REQUIRE(result_owner[1]==111 && result_owner[2]==222);
    else memcpy(out,result_owner+1,bytes);
    /* Native fixture continues after expected errors; preserve its explicit
     * recovery policy while checking the generated status and output above. */
    c->fault.code=0;
    return status;
}
static int collect_and_begin(neoclr_aot_context *c) {
    int status=neoclr_gc_collect_v1(c,NULL);
    return status?status:experiment_gc_begin(c,NULL,NULL,0);
}
'''
# Parameter spellings are the existing native ABI. Generated outputs precede context.
rows=[('listen',[('const neoclr_aot_text *','address'),('int32_t','port'),('int32_t','backlog')],'void *',16),('local_port',[('uint64_t','handle')],'void *',16),('close',[('uint64_t','handle')],'void *',16),('accept',[('uint64_t','listener'),('void *','callback')],'void *',16),('connect_result',[('uint64_t','operation')],'void *',16),('cancel',[('uint64_t','operation')],'int32_t *',4),('deadline_after',[('int32_t','milliseconds')],'int64_t *',8),('deadline_expired',[('int64_t','stamp')],'int32_t *',4)]
rows += [('receive',[('uint64_t','socket'),('void *','array'),('int32_t','offset'),('int32_t','count'),('void *','callback')],'void *',16),
         ('send',[('uint64_t','socket'),('void *','array'),('int32_t','offset'),('int32_t','count'),('void *','callback')],'void *',16),
         ('transfer_result',[('uint64_t','operation')],'void *',16),
         ('receive_until',[('uint64_t','socket'),('void *','array'),('int32_t','offset'),('int32_t','count'),('int64_t','deadline'),('void *','callback')],'void *',16),
         ('send_until',[('uint64_t','socket'),('void *','array'),('int32_t','offset'),('int32_t','count'),('int64_t','deadline'),('void *','callback')],'void *',16)]
for i,(name,args,out_type,width) in enumerate(rows):
    types=', '.join(t for t,n in args)
    params=', '.join(t+' '+n for t,n in args)
    names=', '.join(n for t,n in args)
    s+=f'''typedef int32_t (*wrapper_{i})({types},void *,neoclr_aot_context *);
static int32_t shim_{name}({params},neoclr_aot_context *c,{out_type}out) {{
    calls[{i}]++;prepare(c);
    int status=((wrapper_{i})functions[{i}])({names},result_owner+1,c);
    return finish(c,out,status,{width});
}}
#define neoclr_socket_{name}_v1 shim_{name}
'''
original=root/'docs/experiments/aot-console/socket-transfer-test.c'
t=original.read_text().replace('int main(void)','static int run_socket_case(void)').replace('buffer[129]','buffer[513]').replace('buffer[128]','buffer[512]').replace('(unsigned char *)buffer, 1024, 0','(unsigned char *)buffer, 4096, 0')
a1='    neoclr_socket_scope scope;';assert t.count(a1)==1
t=t.replace(a1,'''    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    CHECK(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,(void **)&result_owner));
    result_owner[0]=17;
    CHECK(!neoclr_gc_host_root_create_v1(&c,result_owner,&result_handle));
    CHECK(!experiment_gc_begin(&c,NULL,NULL,0));
'''+a1)
t=t.replace('neoclr_gc_collect_v1(&c, NULL)','collect_and_begin(&c)')
a1='    c.fault.code = 4;';assert t.count(a1)==1
t=t.replace(a1,'''    CHECK(!neoclr_socket_close_v1(listener,&c,result) && result[0]==0);
    CHECK(!neoclr_socket_close_v1(listener,&c,result) && result[0]==0);
    CHECK(!neoclr_socket_local_port_v1(listener,&c,result) && result[0]==2 && result[1]==1);
'''+a1)
a1='    CHECK(!collect_and_begin(&c) && !c.text.used';assert t.count(a1)==1
t=t.replace(a1,'    CHECK(!neoclr_gc_host_root_release_v1(&c,result_handle));\n'+a1)
a1='    return 0;\n}';assert t.count(a1)==1
t=t.replace(a1,'    CHECK(!neoclr_gc_heap_release_v1(&c.text));\n'+a1)
s+=t+'''
int managed_socket_consumer(void *const *table) {
    functions=table;
    for(unsigned i=0;i<32;i++) { int status=run_socket_case(); if(status) {fprintf(stderr,"socket case line %d\\n",status);return status;} }
    for(unsigned i=0;i<13;i++) REQUIRE(calls[i]);
    puts("PASS 32 generated socket loopback cases: protected heap results, errors, accept, transfer and teardown");
    return 0;
}
#ifdef MANAGED_MAIN
'''
for i,(name,args,_,_) in enumerate(rows):
    s+='extern int32_t live_socket_'+name+'('+', '.join(t for t,n in args)+',void *,neoclr_aot_context *);\n'
s+='int main(void) { void *table[]={'+','.join('live_socket_'+r[0] for r in rows)+'};return managed_socket_consumer(table); }\n#endif\n'
a.output.write_text(s)

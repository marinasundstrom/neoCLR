#include "native-gc.h"
#include "tls-managed.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"TLS wrappers line %d: %s\n",__LINE__,#x); abort(); } } while(0)
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern void experiment_gc_store_counts(uint64_t *);
typedef int32_t (*one)(void *,void *,void *);
typedef int32_t (*three)(void *,void *,void *,void *,void *);
typedef int32_t (*four)(void *,void *,void *,void *,void *,void *);
typedef int32_t (*five)(void *,void *,void *,void *,void *,void *,void *);
static int32_t invoke(const void *const *f,int op,neoclr_aot_context *c,void *owner,void *a,void *b,void *d,void *e,void *result) {
    switch(op) {
        case 1:case 3:return ((three)f[op-1])(owner,a,b,result,c);
        case 2:return ((four)f[1])(owner,a,b,d,result,c);
        case 4:case 5:return ((one)f[op-1])(owner,result,c);
        case 6:return ((five)f[5])(owner,a,b,d,e,result,c);
        default:abort();
    }
}
static void *bytes(neoclr_aot_context *c,const void *data,unsigned count) {
    uint64_t *p;CHECK(!neoclr_gc_allocate_v1(&c->text,16+count,NEOCLR_GC_BYTES,(void **)&p));
    p[0]=UINT64_C(0x80000001);p[1]=count;if(data && count) memcpy(p+2,data,count);return p;
}
static void *file(neoclr_aot_context *c,const char *path) {
    unsigned char data[65536];FILE *f=fopen(path,"rb");CHECK(f);
    size_t n=fread(data,1,sizeof(data),f);CHECK(!ferror(f) && feof(f));fclose(f);return bytes(c,data,(unsigned)n);
}
int tls_wrappers_consumer(const void *const *functions) {
    enum { CERT,KEY,EMPTY,CONFIG,SESSION,NAME,INPUT,OUTPUT,PLAIN,CONTROL,RESULT,BAD,EXTRA,N };
    const char *cert_path=getenv("NEOCLR_TLS_CERT"),*key_path=getenv("NEOCLR_TLS_KEY");CHECK(cert_path && key_path);
    for(unsigned round=0;round<100;round++) {
        uint64_t heap[32768]={0};neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        void *root[N]={0};root[CERT]=file(&c,cert_path);root[KEY]=file(&c,key_path);
        for(unsigned i=EMPTY;i<N;i++) root[i]=bytes(&c,NULL,0);
        root[NAME]=bytes(&c,"localhost",9);root[OUTPUT]=bytes(&c,NULL,17);
        unsigned char control[16]={1};root[CONTROL]=bytes(&c,control,16);
        CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,&root[RESULT]));
        ((uint64_t *)root[RESULT])[0]=17;int32_t *result=(int32_t *)((uint64_t *)root[RESULT]+1);*result=777;
        /* Reserved bytes with an unset initialized flag must reject provider access. */
        root[BAD]=bytes(&c,NULL,2);((uint64_t *)root[BAD])[0]=UINT64_C(0x80000002);((uint64_t *)root[BAD])[1]=1;
        neoclr_probe_storage slots[N];for(unsigned i=0;i<N;i++) slots[i]=(neoclr_probe_storage){&root[i],8,0};
        neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,N};neoclr_probe_frame frame;
        neoclr_gc_enter_map_v1(&frame,&c,0,slots,N,&map,sizeof(map));
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
#define CALL(op,owner,a,b,d,e) invoke(functions,op,&c,owner,a,b,d,e,result)
        CHECK(CALL(4,root[CERT],NULL,NULL,NULL,NULL)==3 && *result==777);
        CHECK(CALL(1,root[CONFIG],root[BAD],root[EMPTY],NULL,NULL)==3 && *result==777);
        CHECK(!CALL(1,root[EXTRA],root[EMPTY],root[EMPTY],NULL,NULL) && *result==-14);
        uint64_t before[5],after[5];experiment_gc_store_counts(before);
        int32_t created=round&1?CALL(1,root[CONFIG],root[CERT],root[EMPTY],NULL,NULL):CALL(2,root[CONFIG],root[CERT],root[KEY],root[EMPTY],NULL);
        if(created || *result) fprintf(stderr,"TLS create round %u: fault %d provider %d\n",round,created,*result);
        CHECK(!created && !*result);
        CHECK(CALL(2,root[CONFIG],root[CERT],root[KEY],root[EMPTY],NULL)==3 && !*result);
        CHECK(!CALL(3,root[SESSION],root[CONFIG],round&1?root[NAME]:root[EMPTY],NULL,NULL) && !*result);
        CHECK(!CALL(3,root[EXTRA],root[SESSION],root[EMPTY],NULL,NULL) && *result==-18);
        CHECK(CALL(6,root[SESSION],root[INPUT],root[OUTPUT],root[PLAIN],root[OUTPUT])==3 && *result==-18);
        CHECK(!CALL(6,root[SESSION],root[INPUT],root[OUTPUT],root[PLAIN],root[CONTROL]) && *result==-1);
        CHECK(((unsigned char *)root[CONTROL])[28]==(round&1?17:0));
        experiment_gc_store_counts(after);CHECK(after[1]+after[2]>before[1]+before[2]);
        void *old_config=root[CONFIG];root[CERT]=root[KEY]=root[CONFIG]=root[NAME]=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame));
        CHECK(CALL(4,old_config,NULL,NULL,NULL,NULL)==3);
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
        CHECK(!CALL(5,root[SESSION],NULL,NULL,NULL,NULL) && !*result);
        if(round%3==0) {
            c.fault.code=1;neoclr_tls_managed_dispose_v1(&c);CHECK(c.fault.code==1);
            neoclr_tls_managed_dispose_v1(&c);
            CHECK(!CALL(5,root[SESSION],NULL,NULL,NULL,NULL) && *result==-18);
        } else if(round%3==1) {
            CHECK(!CALL(4,root[SESSION],NULL,NULL,NULL,NULL) && !*result);
            CHECK(!CALL(4,root[SESSION],NULL,NULL,NULL,NULL) && !*result);
        }
        CHECK(neoclr_root_probe_head_v1()==&frame);
        memset(root,0,sizeof(root));CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        neoclr_gc_leave_v1(&frame);CHECK(!neoclr_gc_entry_check_v1(&c));
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
#undef CALL
    }
    puts("PASS 100 TLS wrapper cycles: real client/server sessions, buffered flights, invalid buffers, heap results, GC ownership, release/abort and teardown");
    return 0;
}
#ifdef MANAGED_MAIN
extern int32_t live_tls_wrappers_0(void *,void *,void *,void *,void *);
extern int32_t live_tls_wrappers_1(void *,void *,void *,void *,void *,void *);
extern int32_t live_tls_wrappers_2(void *,void *,void *,void *,void *);
extern int32_t live_tls_wrappers_3(void *,void *,void *);
extern int32_t live_tls_wrappers_4(void *,void *,void *);
extern int32_t live_tls_wrappers_5(void *,void *,void *,void *,void *,void *,void *);
int main(void) {
    const void *functions[]={live_tls_wrappers_0,live_tls_wrappers_1,live_tls_wrappers_2,live_tls_wrappers_3,live_tls_wrappers_4,live_tls_wrappers_5};
    return tls_wrappers_consumer(functions);
}
#endif

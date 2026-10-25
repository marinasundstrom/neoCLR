#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#define ALLOCATED 1
#define NEOCLR_LIVE_LINEAR_SEED
/* Same fields used by validation; independent exact-boundary search below. */
typedef struct { uint64_t span,bytes,kind,state; } block;
typedef struct { unsigned char *data; uint64_t used; } neoclr_aot_text_arena;
typedef struct { size_t count; block *headers; uint64_t *offsets; } shadow_snapshot;
#include "seed-linear.h"
static unsigned state=987654321;
static unsigned next(void) { state=state*1664525u+1013904223u;return state; }
static int oracle(neoclr_aot_text_arena *a,shadow_snapshot *s) {
 for(size_t i=0;i<s->count;i++) {
  if(i && s->offsets[i]<=s->offsets[i-1])return 3;
  if(!(s->headers[i].state&1))continue;
  const block *found=0;
  for(uint64_t at=0;at<a->used;) {
   const block *b=(const void *)(a->data+at);
   if(at==s->offsets[i])found=b;
   at+=b->span;
  }
  const block *old=&s->headers[i];
  if(!found || !(found->state&1) || found->span!=old->span || found->bytes!=old->bytes || found->kind!=old->kind)return 3;
 }
 return 0;
}
int main(void) {
 unsigned comparisons=0;
 for(unsigned trial=0;trial<2000;trial++) {
  _Alignas(8) unsigned char heap[16384]={0};block headers[64];uint64_t offsets[64];
  uint64_t at=0;
  for(unsigned i=0;i<64;i++) {
   unsigned allocated=(next()>>24)&1; if(!i)allocated=1;
   offsets[i]=at;headers[i]=(block){allocated?48:96,allocated?8:0,allocated?1:0,allocated};at+=headers[i].span;
  }
  uint64_t used=at;
  for(unsigned i=0;i<64;i++) {
   block b=headers[i];at=offsets[i];
   if(b.state)memcpy(heap+at,&b,sizeof(b));
   else if(i+1<64 && !headers[i+1].state && (next()&1)) {
    b.span+=headers[++i].span;memcpy(heap+at,&b,sizeof(b));
   } else {
    b=(block){48,0,0,(next()>>24)&1};b.bytes=b.state?8:0;b.kind=b.state?1:0;memcpy(heap+at,&b,sizeof(b));
    b.state=(next()>>24)&1;b.bytes=b.state?8:0;b.kind=b.state?1:0;memcpy(heap+at+48,&b,sizeof(b));
   }
  }
  block appended={48,8,1,1};memcpy(heap+used,&appended,sizeof(appended));used+=48;
  neoclr_aot_text_arena a={heap,used};shadow_snapshot s={64,headers,offsets};
  assert(candidate_validate_old_linear(&a,&s)==0);comparisons++;
  for(unsigned mutation=0;mutation<8;mutation++) {
   block old=headers[0],second=headers[1],current=*(block *)(void *)heap;uint64_t offset=offsets[0],offset1=offsets[1];
   if(mutation==0)headers[0].span+=8;
   if(mutation==1)headers[0].bytes++;
   if(mutation==2)headers[0].kind++;
   if(mutation==3)offsets[0]+=8;
   if(mutation==4) { block *b=(void *)heap;b->state=0;b->bytes=0;b->kind=0; }
   if(mutation==5) { offsets[0]=offset1;offsets[1]=offset; }
   if(mutation==6) { offsets[1]=offset;headers[1].state=0; }
   if(mutation==7)offsets[0]=UINT64_MAX;
   assert(oracle(&a,&s)==3);
   assert(candidate_validate_old_linear(&a,&s)==oracle(&a,&s));comparisons++;
   headers[0]=old;headers[1]=second;offsets[0]=offset;offsets[1]=offset1;*(block *)(void *)heap=current;
  }
 }
 printf("PASS %u fragmented-layout and invalid-identity comparisons\n",comparisons);
}

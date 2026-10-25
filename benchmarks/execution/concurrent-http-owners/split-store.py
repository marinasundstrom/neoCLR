"""Outline managed-heap stores, retaining all admission predicates in the entry."""
import argparse
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--source',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--pass-state',action='store_true');a=p.parse_args();assert not a.output.exists()
s=a.source.read_text()
start='    if (address<base || address-base>=context->text.used) goto rejected;'
end='\n#ifdef NEOCLR_LIVE_STORE_HANDLE\n#undef live_owner'
assert s.count(start)==s.count(end)==1
lo=s.index(start);hi=s.index(end,lo)
body=s[lo:hi];assert body.endswith('}')
entry='''    return live_store_heap(context,destination,bits,width);
rejected:
    live_store_counts[4]++;
    return 3;
}
'''
s=s[:lo]+entry+s[hi:]
marker='#ifdef NEOCLR_LIVE_STORE_HANDLE\n#ifdef NEOCLR_LIVE_SPLIT_TLS';assert s.count(marker)==1
helper='''#ifdef NEOCLR_LIVE_STORE_HANDLE
#error Heap outlining screen does not support borrowed stores
#endif
/* Private continuation: caller validates ownership/context/width and arena.
 * No callback, allocation, safepoint or epoch transition intervenes. */
__attribute__((noinline)) static int live_store_heap(neoclr_aot_context *context,
    void *destination, uint64_t bits, uint32_t width) {
    uintptr_t address=(uintptr_t)destination, base=(uintptr_t)context->text.data;
'''+body
s=s.replace(marker,helper+'\n'+marker,1)
if a.pass_state:
 start=s.index('__attribute__((noinline)) static int live_store_heap')
 end=s.index(marker,start)
 part=s[start:end].replace('void *destination, uint64_t bits, uint32_t width) {','void *destination, uint64_t bits, uint32_t width, live_owner_state *store_state) {\n#define live_owner (*store_state)',1)
 part+='\n#undef live_owner\n'
 s=s[:start]+part+s[end:]
 s=s.replace('return live_store_heap(context,destination,bits,width);','return live_store_heap(context,destination,bits,width,&live_owner);')
a.output.write_text(s)

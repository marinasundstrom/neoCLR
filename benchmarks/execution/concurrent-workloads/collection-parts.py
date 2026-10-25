"""Attribute selected, disjoint work inside concurrent final collection only."""
import re

HEADER = r'''
static _Thread_local unsigned finish_parts_active;
static _Thread_local finish_row finish_parts[6];
static void finish_part_end(unsigned kind, finish_stamp start) {
    uint64_t cpu=FINISH_CPU_NOW(), wall=FINISH_WALL_NOW();
    if(kind>=6 || cpu<start.cpu || wall<start.wall) abort();
    uint64_t w=wall-start.wall,c=cpu-start.cpu;
    finish_row *r=&finish_parts[kind];
    if(r->count==UINT64_MAX || w>UINT64_MAX-r->wall || c>UINT64_MAX-r->cpu) abort();
    r->count++;r->wall+=w;r->cpu+=c;
    if(w>r->wall_max)r->wall_max=w;
    if(c>r->cpu_max)r->cpu_max=c;
    unsigned b=0;for(uint64_t n=w;n>1;n>>=1)b++;
    r->bins[b]++;
}
void experiment_collection_parts_report(void) {
    if(finish_parts_active) abort();
    for(unsigned i=0;i<6;i++) {
        finish_row *r=&finish_parts[i];
        fprintf(stderr,"%s %u %llu %llu %llu %llu %llu",i<4?"COLLECTION_PART":"SEED_PART",i<4?i:i-4,
            (unsigned long long)r->count,(unsigned long long)r->wall,(unsigned long long)r->wall_max,
            (unsigned long long)r->cpu,(unsigned long long)r->cpu_max);
        for(unsigned j=0;j<64;j++)fprintf(stderr," %llu",(unsigned long long)r->bins[j]);
        fprintf(stderr,"\n");
    }
}
'''

def instrument(source):
    assert 'collection_impl_' not in source
    # Split successful seed calls without per-object clocks or counters. These
    # rows nest inside collection part 2 and must never be added to its total.
    start=source.index('static int candidate_seed(')
    end=source.index('\n}',start)
    body=source[start:end]
    boundary='    size_t previous = 0;'
    assert body.count(boundary)==1
    brace=body.index('{')+1
    body=body[:brace]+'\n    finish_stamp seed_start={0,0};if(finish_parts_active)seed_start=finish_start();'+body[brace:]
    body=body.replace(boundary,'    if(finish_parts_active)finish_part_end(4,seed_start);\n    if(finish_parts_active)seed_start=finish_start();\n'+boundary)
    assert body.count('    return 0;')==1
    body=body.replace('    return 0;','    if(finish_parts_active)finish_part_end(5,seed_start);\n    return 0;')
    source=source[:start]+body+source[end:]
    wrappers=[]
    for name,args,kind,void in [('build_index','a, index',0,False),('mark_frames','context, index, head, pending',1,True),('candidate_seed','context, index, pending',2,False)]:
        matches=list(re.finditer(r'(?m)^(static (?:int|void) )'+name+r'(\([^;{}]*\)) \{',source))
        assert len(matches)==1,(name,len(matches))
        m=matches[0];signature=m[1]+name+m[2]
        replacement=signature+';\n'+m[1]+'collection_impl_'+name+m[2]+' {'
        source=source[:m.start()]+replacement+source[m.end():]
        call='collection_impl_'+name+'('+args+')'
        result=call+';' if void else 'int result='+call+';'
        tail='' if void else 'return result;'
        wrappers.append(signature+' {\n    unsigned active=finish_parts_active;\n    finish_stamp start={0,0};if(active)start=finish_start();\n    '+result+'\n    if(active)finish_part_end(%d,start);\n    '%kind+tail+'\n}\n')
    # The same sweep can be used in begin/sync: active gates both clocks and counts.
    needle='    /* Sweep and coalesce in one forward walk.'
    assert source.count(needle)==1
    source=source.replace(needle,'    finish_stamp sweep_start={0,0};if(finish_parts_active)sweep_start=finish_start();\n'+needle)
    needle='    gc_state.statistics.collections++;'
    assert source.count(needle)==1
    source=source.replace(needle,'    if(finish_parts_active)finish_part_end(3,sweep_start);\n'+needle)
    return source+'\n'+'\n'.join(wrappers)

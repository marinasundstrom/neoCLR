#!/usr/bin/env python3
"""Export owner-thread context switches without counting nested work twice."""
import argparse
import json
from pathlib import Path
NAMES={1:'operation create',2:'operation complete',3:'context enter',4:'context leave',
       5:'park roots published',6:'context finish',7:'context abandoned',8:'scheduler wait begin',9:'scheduler wait end'}
def convert(raw):
    if raw['schema']!='async-1': raise ValueError('unsupported schema')
    events=raw['events']; lost=raw['dropped']; origin=raw['origin_ns']
    if raw['attempted']!=len(events)+lost or lost<0 or len(events)>raw['capacity']:
        raise ValueError('invalid loss accounting')
    output=[]; stack=[]; running=0; since=0; previous=origin; seq=0
    parked={}; parking={}; completions={}; terminal=set(); identities=set(); intervals=0; unknown=0; wait=None
    def span(context,start,end,label):
        if end<start: raise ValueError('negative duration')
        if context:
            output.append(dict(ph='X',pid=1,tid=context,ts=start,dur=end-start,name=label))
    for ns,number,context,related,kind,detail in events:
        if ns<previous or number<=seq or number>raw['attempted'] or kind not in NAMES:
            raise ValueError('invalid order/kind')
        ts=(ns-origin)/1000
        if number!=seq+1:
            stack=[]; running=0; parked={}; parking={}; wait=None; unknown+=1
        previous=ns; seq=number
        output.append(dict(ph='i',s='t',pid=1,tid=context,ts=ts,name=NAMES[kind],args=dict(related=related,detail=detail,sequence=number)))
        if kind==2: completions[related]=ts
        if kind==3:
            if context in terminal or not context: raise ValueError('invalid resumed identity')
            if running!=related and not lost: raise ValueError('wrong parent')
            span(running,since,ts,'executing (wall time)')
            if context in parked:
                start,op=parked.pop(context); complete=completions.get(op)
                if complete is not None and start<=complete<=ts:
                    span(context,start,complete,'waiting for operation')
                    span(context,complete,ts,'completed; awaiting dispatch')
                else:
                    span(context,start,ts,'parked; completion unknown'); unknown+=1
            stack.append(related); running=context; since=ts; identities.add(context); intervals+=1
        elif kind==4:
            if (running!=context or not stack or stack[-1]!=related) and not lost:
                raise ValueError('unbalanced context return')
            span(running,since,ts,'executing (wall time)')
            if context in parking:
                parked[context]=(ts,parking.pop(context))
            if stack: stack.pop()
            running=related; since=ts
        elif kind==5: parking[context]=related
        elif kind in (6,7):
            terminal.add(context)
            if kind==7 and context in parked:
                start,_=parked.pop(context); span(context,start,ts,'parked; abandoned')
        elif kind==8: wait=ts
        elif kind==9:
            if wait is None: unknown+=1
            else: output.append(dict(ph='X',pid=1,tid=0,ts=wait,dur=ts-wait,name='scheduler wait (wall time)'))
            wait=None
    unknown+=len(parked)+len(parking)+len(identities-terminal)+bool(stack)+bool(running)+(wait is not None)
    for context in [0,*sorted(identities)]:
        output.append(dict(ph='M',pid=1,tid=context,name='thread_name',args={'name':f'context {context}' if context else 'owner scheduler'}))
    summary=dict(contexts=len(identities),executionIntervals=intervals,dropped=lost,
                 unknownIntervals=unknown,completeCapture=not lost and not unknown,
                 warning='Logical context tracks on one owner thread; wall intervals are not CPU time. No GC/JIT phase hooks.')
    return dict(traceEvents=output,neoclr=summary),summary
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('input',type=Path);p.add_argument('output',type=Path);a=p.parse_args()
    trace,summary=convert(json.loads(a.input.read_text()));a.output.write_text(json.dumps(trace)+'\n');print(json.dumps(summary,indent=2))

#!/usr/bin/env python3
"""Join async context tracks and GC request/epoch spans from one native session."""
import argparse
import importlib.util
import json
from pathlib import Path
spec=importlib.util.spec_from_file_location('async_export',Path(__file__).with_name('async-export.py'))
async_export=importlib.util.module_from_spec(spec);spec.loader.exec_module(async_export)
def convert(raw):
    if raw['schema']!='session-1' or raw['clock'] not in ('CLOCK_MONOTONIC', 'mach_absolute_time/ns'):raise ValueError('unsupported session clock/schema')
    if sorted(l['lane'] for l in raw['lanes'])!=list(range(raw['lane_count'])):raise ValueError('missing producer lane')
    any_loss=any(l['dropped'] for l in raw['lanes'])
    origin=raw['origin_ns'];output=[];groups={};pending={};unknown=0;lost=0;tracks={};requests=epochs=0
    def track(key,label):
        if key not in tracks:
            tracks[key]=len(tracks)+1
            output.append(dict(ph='M',pid=1,tid=tracks[key],name='thread_name',args={'name':label}))
        return tracks[key]
    lanes=set()
    for lane in raw['lanes']:
        n=lane['lane']
        if n in lanes:raise ValueError('duplicate lane')
        lanes.add(n);lost+=lane['dropped'];previous=origin;seq=0
        if lane['dropped']<0 or lane['attempted']!=len(lane['events'])+lane['dropped'] or len(lane['events'])>lane['capacity']:
            raise ValueError('invalid loss accounting')
        for ns,sequence,domain,kind,source,subject,related,detail in lane['events']:
            if ns<previous or sequence<=seq or sequence>lane['attempted']:raise ValueError('invalid chronology')
            previous=ns;seq=sequence;ts=(ns-origin)/1000
            if domain==2:
                groups.setdefault((n,source),[]).append([ns,0,subject,related,kind,detail])
            elif domain==3:
                family='request' if kind in (100,101) else 'epoch'
                key=(n,family,source,subject)
                tid=track((n,family),f'producer {n}: GC {family}')
                output.append(dict(ph='i',s='t',pid=1,tid=tid,ts=ts,name=f'GC event {kind}',args=dict(id=subject,detail=detail)))
                if kind in (100,110):
                    if key in pending:raise ValueError('duplicate GC begin')
                    pending[key]=(ts,tid,detail)
                elif kind in (101,111,112):
                    begin=pending.pop(key,None)
                    if begin is None:unknown+=1;continue
                    start,tid,reason=begin
                    name='owner collection request (wall)' if family=='request' else 'concurrent epoch lifetime (not CPU or pause)'
                    output.append(dict(ph='X',pid=1,tid=tid,ts=start,dur=ts-start,name=name,args=dict(id=subject,reason=reason,status=detail,cancelled=kind==112)))
                    if family=='request':requests+=1
                    else:epochs+=1
                else:raise ValueError('unknown GC event')
            else:
                unknown+=1
    for (lane,source),events in groups.items():
        if any_loss:
            for ns,_,context,related,kind,detail in events:
                tid=track((lane,source,context),f'producer {lane}, async session {source}: context {context}')
                output.append(dict(ph='i',s='t',pid=1,tid=tid,ts=(ns-origin)/1000,name=async_export.NAMES.get(kind,'unknown async event'),args=dict(related=related,detail=detail)))
            continue
        for i,event in enumerate(events):event[1]=i+1
        view=dict(schema='async-1',origin_ns=origin,events=events,attempted=len(events)+lost,dropped=lost,capacity=len(events))
        converted,summary=async_export.convert(view);unknown+=summary['unknownIntervals']
        for event in converted['traceEvents']:
            if event['ph']=='M':continue
            old=event['tid'];event['tid']=track((lane,source,old),f'producer {lane}, async session {source}: '+(f'context {old}' if old else 'scheduler'))
            output.append(event)
    unknown+=len(pending)
    if any_loss:
        output=[e for e in output if e['ph']!='X']
        requests=epochs=0
        unknown=max(1,unknown)
    summary=dict(session=raw['session'],producers=len(lanes),asyncSessions=len(groups),gcRequests=requests,gcEpochs=epochs,dropped=lost,unknownIntervals=unknown,completeCapture=not lost and not unknown,
        warning='Epoch spans are lifetime, not marker CPU or stop-the-world pauses. JIT and worker providers are not integrated yet.')
    return dict(traceEvents=output,neoclr=summary),summary
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('input',type=Path);p.add_argument('output',type=Path);a=p.parse_args()
    trace,summary=convert(json.loads(a.input.read_text()));a.output.write_text(json.dumps(trace)+'\n');print(json.dumps(summary,indent=2))

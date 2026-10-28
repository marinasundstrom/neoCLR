#!/usr/bin/env python3
"""Convert the private worker schema to Chrome JSON for Perfetto, offline."""
import argparse
import json
from pathlib import Path

NAMES = {1: 'worker start', 2: 'worker stop', 3: 'idle begin', 4: 'idle end',
         5: 'enqueue', 6: 'run', 7: 'callback return', 8: 'settle', 9: 'cancel requested'}

def convert(raw):
    if raw['schema'] != 1 or raw['clock'] != 'CLOCK_MONOTONIC' or raw['units'] != 'ns':
        raise ValueError('unsupported trace schema/clock')
    output, jobs, waits = [], {}, []
    if sorted(lane['lane'] for lane in raw['lanes']) != list(range(raw['workers']+1)):
        raise ValueError('missing or duplicate producer lane')
    if len({(lane['pool'],lane['origin_ns']) for lane in raw['lanes']}) != 1:
        raise ValueError('inconsistent session clock/identity')
    lost = sum(lane['dropped'] for lane in raw['lanes'])
    incomplete = 0
    for lane in raw['lanes']:
        tid, pid, origin = lane['lane'], lane['pool'], lane['origin_ns']
        output.append(dict(ph='M', name='thread_name', pid=pid, tid=tid,
                           args={'name': f'worker {tid}' if tid else 'external callers (serialized)'}))
        previous_seq, previous_ns = 0, origin
        idle = None
        for ns, seq, generation, kind, slot, detail in lane['events']:
            if seq <= previous_seq or ns < previous_ns or kind not in NAMES:
                raise ValueError('invalid event order/kind')
            if seq != previous_seq + 1:
                incomplete += 1
                idle = None
            previous_seq, previous_ns = seq, ns
            event = dict(ph='i', s='t', name=NAMES[kind], pid=pid, tid=tid,
                         ts=(ns-origin)/1000, args=dict(slot=slot,generation=generation,detail=detail,sequence=seq))
            output.append(event)
            key = (pid, slot, generation)
            if generation:
                job = jobs.setdefault(key, {})
                if kind in job: raise ValueError('duplicate job transition')
                job[kind] = event
            if kind == 3: idle = event
            if kind == 4:
                if idle:
                    output.append(dict(ph='X',name='condition wait (includes reacquire)',pid=pid,tid=tid,
                                       ts=idle['ts'],dur=event['ts']-idle['ts']))
                else: incomplete += 1
                idle = None
        if idle: incomplete += 1
        if (lane['attempted'] != len(lane['events']) + lane['dropped'] or
                previous_seq > lane['attempted'] or lane['dropped'] < 0 or
                len(lane['events']) > lane['capacity']):
            raise ValueError('inconsistent loss accounting')
    for key, job in jobs.items():
        enqueue, run, returned, settle = (job.get(k) for k in (5,6,7,8))
        if enqueue and run:
            wait = run['ts'] - enqueue['ts']
            if wait < 0: raise ValueError('run precedes enqueue')
            waits.append(wait)
            flow = ':'.join(map(str,key))
            for ph, e in [('s',enqueue),('f',run)]:
                output.append(dict(ph=ph,name='queued work',cat='scheduler',id=flow,
                                   pid=e['pid'],tid=e['tid'],ts=e['ts'],**({'bp':'e'} if ph=='f' else {})))
        if run and returned:
            if returned['tid'] != run['tid'] or returned['ts'] < run['ts']:
                raise ValueError('invalid callback interval')
            output.append(dict(ph='X',name='callback wall interval',pid=run['pid'],tid=run['tid'],
                               ts=run['ts'],dur=returned['ts']-run['ts'],args=dict(slot=key[1],generation=key[2])))
        # Queued cancellation legitimately has no run/return.
        complete = enqueue and settle and ((run and returned) or (not run and 9 in job and settle['args']['detail']==1))
        if not complete: incomplete += 1
    summary = dict(jobsObserved=len(jobs),queueWaitSamples=len(waits),
                   queueWaitMeanUs=sum(waits)/len(waits) if waits else None,
                   dropped=lost,incompleteIntervals=incomplete,
                   completeCapture=not lost and not incomplete,
                   warning='Wall intervals are not CPU utilization. No GC/JIT/I/O hooks in schema 1.')
    return dict(traceEvents=output,displayTimeUnit='ms',neoclr=summary), summary

if __name__ == '__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('input',type=Path); p.add_argument('output',type=Path)
    a=p.parse_args()
    trace,summary=convert(json.loads(a.input.read_text()))
    a.output.write_text(json.dumps(trace,separators=(',',':'))+'\n')
    print(json.dumps(summary,indent=2))

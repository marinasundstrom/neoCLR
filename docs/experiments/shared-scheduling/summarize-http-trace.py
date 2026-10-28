#!/usr/bin/env python3
"""Summarize bounded native carrier/GC traces; not managed task-queue residence."""
import argparse
from collections import Counter
import json
import math
from pathlib import Path
import statistics

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('results', type=Path)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
results = json.loads(a.results.read_text())
assert results['passed'] and results['kind'] == 'profile'

def distribution(values):
    values = sorted(values)
    if not values: return dict(count=0)
    return dict(count=len(values), minNs=values[0], medianNs=statistics.median(values),
                p95Ns=values[math.ceil(len(values)*.95)-1],
                p99Ns=values[math.ceil(len(values)*.99)-1], maxNs=values[-1])

rows = []
for trial in results['trials']:
    traces = [json.loads(line[6:]) for line in trial['stderr'].splitlines() if line.startswith('TRACE ')]
    assert len(traces) == 1
    trace = traces[0]
    assert trace['dropped'] == trace['asyncDropped'] == 0, 'incomplete trace'
    events = trace['events']
    assert trace['attempted'] == len(events)
    kinds = Counter()
    yielded = {}; parked = {}; completed = {}; gc = []
    transfers = []; finished = []; ready = []; requests = []
    queued = {}; dequeued = {}; dispatch = {}; queue_wait = []; dispatch_setup = []
    root_hold = []; high_water = 0; admitted = abandoned = io_dispatch = 0
    previous = 0
    for index, (ns, sequence, domain, kind, source, subject, related, detail) in enumerate(events):
        assert sequence == index+1 and ns >= previous
        previous = ns; kinds[f'{domain}:{kind}'] += 1
        key = (source, subject)
        if domain == 2:
            if kind in (5, 6):
                yielded[key] = (ns, kind)
                if kind == 5: parked[key] = related
            elif kind == 4 and key in yielded:
                start, origin = yielded.pop(key)
                (transfers if origin == 5 else finished).append(ns-start)
            elif kind == 2:
                completed[(source, related)] = ns
            elif kind == 3 and key in parked:
                operation = parked.pop(key)
                completed_at = completed.get((source, operation))
                if completed_at is not None: ready.append(ns-completed_at)
        elif domain == 3:
            if kind == 100: gc.append(ns)
            elif kind == 101:
                assert gc, 'unmatched GC request end'
                requests.append(ns-gc.pop())
        elif domain == 4:
            if kind == 200:
                assert key not in queued
                queued[key] = ns; admitted += 1
                high_water = max(high_water, len(queued))
            elif kind == 201:
                assert key in queued, 'dequeue without admitted runnable'
                queue_wait.append(ns-queued.pop(key)); dequeued[key] = ns
            elif kind == 202:
                if key in dequeued: dispatch_setup.append(ns-dequeued.pop(key))
                else: io_dispatch += 1
                ticket = (source, related)
                assert ticket not in dispatch
                dispatch[ticket] = ns
            elif kind == 203:
                assert key in dispatch, 'release without dispatch ownership'
                root_hold.append(ns-dispatch.pop(key))
            elif kind == 204:
                assert key in queued
                queued.pop(key); abandoned += 1
            else: raise AssertionError('unknown scheduler event')
    assert not gc
    assert not queued and not dequeued and not dispatch, 'incomplete scheduler ownership lifecycle'
    stacks = next(line for line in trial['stderr'].splitlines() if line.startswith('ASYNC '))
    switches = next(line for line in trial['stderr'].splitlines() if line.startswith('OS_SWITCHES '))
    rows.append(dict(mode=trial['mode'], sample=trial['sample'], events=len(events), kinds=dict(kinds),
                     clock=trace['clock'], schedulerAdmitted=admitted, schedulerAbandoned=abandoned,
                     schedulerQueueHighWater=high_water, otherDispatches=io_dispatch,
                     schedulerQueueResidence=distribution(queue_wait),
                     schedulerDequeueToDispatch=distribution(dispatch_setup),
                     schedulerDispatchRootHold=distribution(root_hold),
                     instrumentationEventStorageBytes=trace['capacity']*(trace['recordBytes']+trace['asyncRecordBytes']),
                     parkedCarrierReturn=distribution(transfers), finishedCarrierReturn=distribution(finished),
                     completedOperationToCarrierEnter=distribution(ready), gcRequest=distribution(requests),
                     carrierCounters=list(map(int, stacks.split()[1:])),
                     osSwitchCounters=list(map(int, switches.split()[1:])),
                     processMemory=trial.get('processMemory')))
report = dict(scope=__doc__, source=str(a.results.resolve()), rows=rows,
              limitations=['Traced short qualification runs, not headline performance or capacity.',
                           'Park/finish to leave includes trace emission and swapcontext return; root parking occurs before its start.',
                           'Completion to enter includes pump selection; not pure CPU context-switch cost.',
                           'Runnable queue events cover shared scheduler submissions, not every state-machine await or external I/O wait.',
                           'Dispatch-root release can precede callback execution; root-hold duration is not execution time.',
                           'OS switches cover the whole process including JIT/GC helper threads.',
                           'Process RSS includes JIT/compiler memory; virtual stack reservation is not resident memory.'])
a.output.write_text(json.dumps(report, indent=2)+'\n')
print(json.dumps(rows, indent=2))

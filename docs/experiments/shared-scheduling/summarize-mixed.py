#!/usr/bin/env python3
"""Summarize repeated untraced mixed HTTP runs; no capacity inference."""
import argparse
from collections import defaultdict
import hashlib
import json
import math
from pathlib import Path
import statistics


def distribution(values):
    values = sorted(values)
    assert values and all(math.isfinite(v) and v >= 0 for v in values)
    return dict(count=len(values), minimum=values[0], median=statistics.median(values),
                p95=values[math.ceil(len(values)*.95)-1],
                p99=values[math.ceil(len(values)*.99)-1], maximum=values[-1])


def summarize(source):
    report = json.loads(source.read_text())
    assert report['passed'] and report['kind'] == 'qualification', 'untraced successful runs required'
    rows = []
    identities = set()
    for trial in report['trials']:
        assert trial['passed'] and trial['exitCode'] == 0
        identity = (trial['mode'], trial['repetition'])
        assert identity not in identities
        identities.add(identity)
        assert len(trial['firstUse']) == 24
        assert len(trial['requests']) == report['clients'] * report['cycles'] * 24
        assert trial['batchSeconds'] > 0
        families = defaultdict(list)
        for request in trial['requests']:
            families[request['route'].split('/')[1]].append(request['seconds'] * 1000)
        assert set(families) == {'short', 'allocate', 'file-read', 'file-write', 'network', 'fanout'}
        profile = [line.split()[1:] for line in trial['stderr'].splitlines() if line.startswith('PROFILE ')]
        assert len(profile) == 1 and len(profile[0]) == 7
        assert not any(line.startswith('TRACE ') for line in trial['stderr'].splitlines())
        cpu, wall, allocations, allocated, collections, live, arena = map(float, profile[0])
        rows.append(dict(mode=trial['mode'], repetition=trial['repetition'],
                         readySeconds=trial['readySeconds'],
                         firstUseSeconds=sum(x['seconds'] for x in trial['firstUse']),
                         warmRequests=len(trial['requests']), batchSeconds=trial['batchSeconds'],
                         requestsPerSecond=len(trial['requests']) / trial['batchSeconds'],
                         latencyMs=distribution([x['seconds'] * 1000 for x in trial['requests']]),
                         familiesMs={k: distribution(v) for k, v in families.items()},
                         invocationCpuSeconds=cpu, invocationWallSeconds=wall,
                         invocationAllocations=int(allocations), invocationAllocatedBytes=int(allocated),
                         invocationCollections=int(collections), invocationPeakLiveBytes=int(live),
                         invocationPeakArenaBytes=int(arena)))
    modes = sorted({row['mode'] for row in rows})
    summaries = []
    for mode in modes:
        trials = [row for row in rows if row['mode'] == mode]
        assert {row['repetition'] for row in trials} == set(range(report['repetitions']))
        summary = dict(mode=mode, trials=len(trials))
        for key in ('readySeconds', 'firstUseSeconds', 'requestsPerSecond', 'invocationCpuSeconds',
                    'invocationAllocatedBytes', 'invocationCollections', 'invocationPeakLiveBytes'):
            summary[key] = distribution([row[key] for row in trials])
        summary['trialP95Ms'] = distribution([row['latencyMs']['p95'] for row in trials])
        summary['trialP99Ms'] = distribution([row['latencyMs']['p99'] for row in trials])
        summary['familyTrialP95Ms'] = {family: distribution([row['familiesMs'][family]['p95'] for row in trials])
                                      for family in trials[0]['familiesMs']}
        summaries.append(summary)
    return dict(scope=__doc__, source=str(source.resolve()), sourceSha256=hashlib.sha256(source.read_bytes()).hexdigest(),
                clients=report['clients'], cycles=report['cycles'], repetitions=report['repetitions'],
                trials=rows, summaries=summaries,
                limitations=['Closed-loop requests include downstream delay and client overhead; not offered-load capacity.',
                             'Ready includes process launch, metadata selection/verification and JIT runtime setup; filesystem cache is uncontrolled.',
                             'Invocation CPU/allocation/GC counts include first use, warm work and teardown, not warm-only CPU/request.',
                             'Three trials are descriptive; percentiles of trial summaries are not pooled request percentiles.',
                             'File payloads are small and cache state is uncontrolled; no cold-storage performance claim.'])


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('results', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    result = summarize(args.results)
    args.output.write_text(json.dumps(result, indent=2) + '\n')
    for row in result['summaries']:
        print(row['mode'], 'ready', round(row['readySeconds']['median'], 3),
              'req/s', round(row['requestsPerSecond']['median'], 1),
              'p99 ms', round(row['trialP99Ms']['median'], 2))

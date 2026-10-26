"""Summarize offered load, useful completions, tails and per-app interference.

Highest passing tested rate is a bounded observation, not a capacity estimate.
All failed arrivals remain in the denominator. Do not pool different workloads,
warmup policies, client limits, dependency delays or traffic shares.
"""
import argparse
from collections import Counter
import hashlib
import json
import math
from pathlib import Path
import statistics

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--measurement', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
data = json.loads(a.measurement.read_text())
assert data['completed'] and not data.get('diagnosticOnly')


def distribution(values):
    present = [value for value in values if value is not None]
    return dict(median=statistics.median(present) if present else None,
                minimum=min(present) if present else None, maximum=max(present) if present else None,
                missing=len(values)-len(present), values=values)


trials, groups = [], {}
for trial in data['trials']:
    assert trial['cleanShutdown']
    servers = trial['servers']
    outcomes = Counter()
    for server in servers:
        outcomes.update(server['measurement']['outcomes'])
    offered = sum(s['measurement']['offered'] for s in servers)
    success = outcomes['ok']
    within = sum(s['measurement']['completedWithinWindow'] for s in servers)
    per_app = []
    for server in servers:
        m = server['measurement']
        requests = m['requests']
        timeline = []
        for second in range(math.ceil(data['durationSeconds'])):
            timeline.append(dict(second=second,
                completed=sum(r['result'] == 'ok' and second <= r['endSeconds'] < second+1 for r in requests),
                outstandingAtEnd=sum('startSeconds' in r and r['startSeconds'] <= second+1 < r['endSeconds'] for r in requests),
                missed=sum(r['result'] == 'generator_inflight_limit' and second <= r['intendedSeconds'] < second+1 for r in requests)))
        per_app.append(dict(offered=m['offered'], outcomes=m['outcomes'],
                            completionRate=m['successfulCompletionsPerSecond'],
                            successFraction=m['successfulOfferedFraction'],
                            offeredLatencySeconds=m['offeredLatencySeconds'],
                            dispatchLagSeconds=m['dispatchLagSeconds'],
                            peakInflight=m['peakClientInflight'], drainSeconds=m['drainSeconds'],
                            meetsTarget=m['meetsTarget'], timeline=timeline))
    row = dict(mode=trial['mode'], apps=trial['apps'], rate=trial['aggregateOfferedRate'],
               round=trial['round'], outcomes=dict(outcomes), offered=offered,
               completionRate=within/data['durationSeconds'], successFraction=success/offered,
               worstAppP99Seconds=(max(s['measurement']['offeredLatencySeconds']['p99'] for s in servers)
                   if all(s['measurement']['offeredLatencySeconds']['p99'] is not None for s in servers) else None),
               worstDispatchP99Seconds=max(s['measurement']['dispatchLagSeconds']['p99'] for s in servers),
               cpuSeconds=sum(s['cpuSecondsDuringTrafficAndDrain'] for s in servers),
               rssAfterKiB=sum(s['rssAfterKiB'] for s in servers),
               drainSeconds=max(s['measurement']['drainSeconds'] for s in servers),
               allAppsMeetTarget=all(s['measurement']['meetsTarget'] for s in servers), perApp=per_app)
    trials.append(row)
    groups.setdefault((row['mode'], row['apps'], row['rate']), []).append(row)
rows = []
for (mode, apps, rate), values in sorted(groups.items()):
    assert len(values) == data['rounds']
    rows.append(dict(mode=mode, apps=apps, rate=rate, trials=len(values),
                     allTrialsMeetTarget=all(v['allAppsMeetTarget'] for v in values),
                     metrics={name: distribution([v[name] for v in values]) for name in
                              ['completionRate', 'successFraction', 'worstAppP99Seconds',
                               'worstDispatchP99Seconds', 'cpuSeconds', 'rssAfterKiB', 'drainSeconds']}))
passing = []
for mode, apps in sorted({(r['mode'], r['apps']) for r in rows}):
    rates = [r['rate'] for r in rows if r['mode'] == mode and r['apps'] == apps and r['allTrialsMeetTarget']]
    passing.append(dict(mode=mode, apps=apps, highestPassingTestedRate=max(rates) if rates else None))
report = dict(completed=True, scope=__doc__, rounds=data['rounds'], workload=data['workload'],
              backendOptimization=data.get('backendOptimization','explicit retained recipe'),
              reusedProcessAcrossRates=data.get('reusedProcessAcrossRates',False),
              target=data['target'], headerGapMs=data['headerGapMs'],
              dependencyDelayMs=data['dependencyDelayMs'], loadShares=data['loadShares'],
              warmupRequestsPerApp=data['warmupRequestsPerApp'],
              perAppClientInflightLimit=data['perAppClientInflightLimit'],
              rows=rows, trials=trials, highestPassingTestedRates=passing,
              inputs={str(path.resolve()): hashlib.sha256(path.read_bytes()).hexdigest()
                      for path in [a.measurement, Path(__file__)]})
a.output.write_text(json.dumps(report, indent=2)+'\n')

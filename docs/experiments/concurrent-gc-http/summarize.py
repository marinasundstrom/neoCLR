#!/usr/bin/env python3
"""Derive comparison rows and coarse CPU peaks from retained HTTP load samples."""
import argparse
import json
from pathlib import Path
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('comparison', type=Path)
a = p.parse_args()
manifest = json.loads((a.comparison / 'comparison.json').read_text())
assert manifest['completed']
rows = []
for run in manifest['runs']:
    name = f"{run['mode']}-{run['round']}-{run['rate']}"
    trial = json.loads((a.comparison / name / 'results.json').read_text())['trials'][0]
    assert trial['cleanShutdown'] and trial['serverCompletedMinusClientSuccesses'] == 0
    m = trial['measurement']
    samples = [s for s in m['resources'] if 'cpuSeconds' in s and s['atSeconds'] <= m['durationSeconds']]
    assert len(samples) > 1
    # Difference adjacent cumulative CPU samples. No claim about sub-sample peaks.
    peaks = [(b['cpuSeconds'] - s['cpuSeconds']) / (b['atSeconds'] - s['atSeconds'])
             for s, b in zip(samples, samples[1:])]
    rows.append(dict(name=name, mode=run['mode'], round=run['round'], rate=run['rate'],
        offered=m['offered'], outcomes=m['outcomes'], completedPerSecond=m['successfulCompletionsPerSecond'],
        cpuMillisecondsPerCompletion=m['serverCpuSecondsPerCompletion'] * 1000,
        meanCpuCores=m['serverCpuSecondsInWindow'] / m['durationSeconds'],
        sampledPeakCpuCores=max(peaks), sampledPeakRssKiB=m['sampledPeakRssKiB'],
        p99Milliseconds=m['offeredLatencySeconds']['p99'] * 1000,
        generatorP99Milliseconds=m['dispatchLagSeconds']['p99'] * 1000,
        passed=m['meetsProvisionalTarget'], serverTotals=trial['serverTotals'], gc=run.get('gc')))
print(json.dumps(dict(contract='CPU cores = process CPU seconds / wall seconds. Peak uses adjacent approximately 0.5-second samples inside the window; misses shorter spikes. Successful-response p99 must be read with outcomes. GC totals include warmup and teardown.', rows=rows), indent=2))

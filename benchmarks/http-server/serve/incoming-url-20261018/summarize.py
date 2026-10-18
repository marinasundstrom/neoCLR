#!/usr/bin/env python3
"""Summarize retained offered-load trials without discarding missed arrivals or lag."""
import argparse
import json
from pathlib import Path
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('directory', type=Path)
p.add_argument('--output', type=Path)
a = p.parse_args()
rows = []
for path in sorted(a.directory.glob('*/results.json')):
    mode, round_number, rate = path.parent.name.split('-')
    trial = json.loads(path.read_text())['trials'][0]
    m = trial['measurement']
    samples = m['resources']
    cpu_peaks = [(right['cpuSeconds']-left['cpuSeconds']) /
                 (right['atSeconds']-left['atSeconds'])
                 for left,right in zip(samples,samples[1:])
                 if right['atSeconds'] > left['atSeconds']]
    row = dict(mode=mode, round=int(round_number), rate=int(rate),
               offered=m['offered'], admitted=m['admitted'], outcomes=m['outcomes'],
               completedWithinWindow=m['completedWithinWindow'],
               throughput=m['successfulCompletionsPerSecond'],
               cpuMsPerCompletion=1000*m['serverCpuSecondsPerCompletion'],
               offeredP99Ms=1000*m['offeredLatencySeconds']['p99'],
               dispatchLagP99Ms=1000*m['dispatchLagSeconds']['p99'],
               peakRssKiB=m['sampledPeakRssKiB'],
               peakSampledCpuCores=max(cpu_peaks, default=0),
               provisionalPass=m['meetsProvisionalTarget'],
               cleanShutdown=trial['cleanShutdown'],
               accountingDifference=trial['serverCompletedMinusClientSuccesses'])
    rows.append(row)
report = dict(rows=rows, cpuPeakMethod='Largest process CPU delta / elapsed delta between resource samples; coarse ~0.5-second samples, not instantaneous peaks. One core is 1.0. Timing excludes allocation instrumentation.')
if a.output:
    a.output.write_text(json.dumps(report,indent=2)+'\n')
for row in rows:
    print(row['mode'],row['round'],row['rate'],row['outcomes'],
          f"CPU {row['cpuMsPerCompletion']:.3f}ms p99 {row['offeredP99Ms']:.2f}ms lag {row['dispatchLagP99Ms']:.2f}ms pass {row['provisionalPass']}")

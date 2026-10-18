#!/usr/bin/env python3
"""Summarize retained offered-load reports without dropping failed trials/arrivals."""
import argparse
from collections import Counter
import importlib.util
import json
from pathlib import Path

spec = importlib.util.spec_from_file_location('load_driver', Path(__file__).with_name('run.py'))
load = importlib.util.module_from_spec(spec)
spec.loader.exec_module(load)


def summarize(report):
    trials = []
    for trial in report['trials']:
        m = trial.get('measurement')
        if m is None:
            trials.append(dict(rate=trial['rate'], error=trial.get('error', 'incomplete')))
            continue
        records = m['requests']
        phases = {}
        for label, end, start in [('connect', 'connectedSeconds', None),
                                  ('responseWait', 'firstByteSeconds', 'sentSeconds'),
                                  ('responseRead', 'eofSeconds', 'firstByteSeconds'),
                                  ('clientClose', 'closedSeconds', 'eofSeconds')]:
            values = [r['phases'][end] - (r['phases'][start] if start else 0)
                      for r in records if end in r.get('phases', {}) and
                      (start is None or start in r['phases'])]
            phases[label] = dict(samples=len(values), seconds=load.percentiles(values))
        seconds = []
        for second in range(int(m['durationSeconds'])):
            arrivals = [r for r in records if second <= r['intendedSeconds'] < second + 1]
            completed = [r for r in records if r['result'] == 'ok' and second <= r['endSeconds'] < second + 1]
            seconds.append(dict(second=second, outcomes=dict(Counter(r['result'] for r in arrivals)),
                                completions=len(completed),
                                dispatchLag=load.percentiles([r['dispatchLagSeconds'] for r in arrivals]),
                                offeredLatency=load.percentiles([r['offeredLatencySeconds'] for r in arrivals if r['result'] == 'ok'])))
        rss = [r for r in m['resources'] if 'rssKiB' in r]
        trials.append(dict(round=trial['round'], rate=trial['rate'], concurrency=trial['concurrency'],
                           outcomes=m['outcomes'], completionsPerSecond=m['successfulCompletionsPerSecond'],
                           target=m['meetsProvisionalTarget'], latency=m['offeredLatencySeconds'],
                           dispatchLag=m['dispatchLagSeconds'], cpuPerCompletion=m['serverCpuSecondsPerCompletion'],
                           rssFirstKiB=rss[0]['rssKiB'] if rss else None,
                           rssLastKiB=rss[-1]['rssKiB'] if rss else None,
                           rssPeakKiB=m['sampledPeakRssKiB'], phaseLatencies=phases, seconds=seconds,
                           slowest=sorted((r for r in records if r['result'] == 'ok'),
                                          key=lambda r: r['offeredLatencySeconds'], reverse=True)[:10]))
    return dict(completed=report['completed'], trials=trials)


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('report', type=Path)
    p.add_argument('--output', required=True, type=Path)
    a = p.parse_args()
    a.output.write_text(json.dumps(summarize(json.loads(a.report.read_text())), indent=2) + '\n')

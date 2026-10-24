#!/usr/bin/env python3
"""Summarize unprofiled matched-host capacity, preserving the CPU window."""
import gzip
import json
from pathlib import Path
from statistics import mean


def summarize(report):
    if not report['passed'] or len(report['trials']) != 8:
        raise ValueError('Expected the complete eight-trial capacity cohort')
    summary = {}
    for mode in ['aot', 'jit']:
        values = {}
        for ownership in ['external', 'owned']:
            rows = [x for x in report['trials'] if x['label'] == f'{mode}-{ownership}']
            if len(rows) != 2:
                raise ValueError('Expected two trials per configuration')
            for row in rows:
                count = sum(x['completed'] + x['drained'] for x in [row['warmup'], row['trial']])
                expected = row['windowCpuSeconds'] * 1e6 / count
                if abs(expected - row['windowCpuMicrosecondsPerResponse']) > 1e-6:
                    raise ValueError('CPU denominator mismatch')
            values[ownership] = dict(requestsPerSecond=mean(x['trial']['requestsPerSecond'] for x in rows),
                                     cpuMicrosecondsPerResponse=mean(x['windowCpuMicrosecondsPerResponse'] for x in rows))
        values['throughputIncreasePercent'] = 100 * (values['owned']['requestsPerSecond'] / values['external']['requestsPerSecond'] - 1)
        values['cpuReductionPercent'] = 100 * (1 - values['owned']['cpuMicrosecondsPerResponse'] / values['external']['cpuMicrosecondsPerResponse'])
        summary[mode] = values
    return summary


if __name__ == '__main__':
    import argparse
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('report', type=Path)
    a = p.parse_args()
    data = a.report.read_bytes()
    print(json.dumps(summarize(json.loads(gzip.decompress(data) if a.report.suffix == '.gz' else data)), indent=2))

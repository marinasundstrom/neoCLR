#!/usr/bin/env python3
"""Summarize checked matched-host trials; refuse incomplete measurements."""
import json
import statistics
import sys


def summarize(report):
    if not report.get('passed'):
        raise ValueError('incomplete measurement')
    groups = {}
    for row in report['trials']:
        if row['exitCode'] != 0:
            raise ValueError('failed server')
        groups.setdefault(row['label'], []).append(row)
    if set(groups) != {'baseline', 'long-range', 'dynamic', 'jit'}:
        raise ValueError('missing configuration')
    result = {}
    for label, rows in groups.items():
        if len(rows) != 2:
            raise ValueError('expected two trials per configuration')
        rates = [row['trial']['requestsPerSecond'] for row in rows]
        result[label] = dict(requestsPerSecond=statistics.mean(rates),
                             rateRange=[min(rates), max(rates)],
                             cpuMicrosecondsPerResponse=statistics.mean(
                                 row['cpuMicrosecondsPerResponse'] for row in rows))
    return result


if __name__ == '__main__':
    print(json.dumps(summarize(json.load(open(sys.argv[1]))), indent=2))

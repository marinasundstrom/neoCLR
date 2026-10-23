#!/usr/bin/env python3
"""Summarize successful runs without mixing diagnostics, startup and warm CPU."""
import argparse
import gzip
import json
from pathlib import Path
from statistics import median


def read(path):
    data = Path(path).read_bytes()
    return json.loads(gzip.decompress(data) if str(path).endswith('.gz') else data)


def summarize(ordinary, http):
    if not ordinary['passed'] or not http['passed']:
        raise ValueError('Cannot summarize an incomplete cohort as successful')
    result = {'ordinary': [], 'http': []}
    for case in ordinary['cases']:
        for mode, policy in [('interpreter', None), ('jit', 'forced'),
                             ('jit', 'paced'), ('aot', 'forced'), ('aot', 'paced')]:
            rows = [x for x in case['runs'] if x['mode'] == mode
                    and x.get('policy') == policy and not x['diagnostic']]
            if len(rows) != 3 or any(x['exitCode'] or not x['outputMatches'] for x in rows):
                raise ValueError('Expected three checked ordinary trials')
            row = {'sample': case['sample'], 'mode': mode, 'policy': policy,
                   'processWallSeconds': median(x['wallSeconds'] for x in rows),
                   'processCpuSeconds': median(x['processCpuSeconds'] for x in rows)}
            if mode == 'jit':
                for field in ['metadataLoadSeconds', 'selectionAndVerificationSeconds',
                              'compileAndFinalizeSeconds', 'invocationSeconds']:
                    row[field] = median(x['jit'][field] for x in rows)
            elif mode == 'aot':
                row['invocationSeconds'] = median(float(next(
                    line.split()[1] for line in x['stderr'].splitlines()
                    if line.startswith('MODETIME '))) for x in rows)
            result['ordinary'].append(row)
    for label in ['aot-forced', 'jit-forced', 'aot-paced', 'jit-paced']:
        rows = [x for x in http['trials'] if x['label'] == label]
        if len(rows) != 2:
            raise ValueError('Expected two HTTP trials per mode/policy')
        result['http'].append({
            'label': label,
            'medianRequestsPerSecond': median(x['trial']['requestsPerSecond'] for x in rows),
            'medianWindowCpuMicrosecondsPerResponse': median(
                x['windowCpuMicrosecondsPerResponse'] for x in rows),
            'medianReadySeconds': median(x['readySeconds'] for x in rows),
        })
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('ordinary')
    parser.add_argument('http')
    args = parser.parse_args()
    print(json.dumps(summarize(read(args.ordinary), read(args.http)), indent=2))

#!/usr/bin/env python3
"""Run isolated Raven workers; measure external batch time, including transport."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import statistics

from process import Guest


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def sample(command, operations, timeout=120):
    guest = Guest(command, timeout=timeout)
    try:
        _, ready = guest.line(b'NEOBENCH/1 READY')
        _, begin = guest.line(b'NEOBENCH/1 BEGIN')
        _, end = guest.line(b'NEOBENCH/1 END')
        guest.line(str(operations).encode())
        guest.finish()
        if len(guest.output) != 4 or end <= begin:
            raise RuntimeError('Invalid worker output or nonpositive batch duration')
        return dict(startupSeconds=ready-guest.started, warmupSeconds=begin-ready,
                    batchSeconds=end-begin, operations=operations,
                    operationsPerSecond=operations/(end-begin))
    finally:
        guest.close()


def run(manifest, output, rounds, warmups, operations, selected=None):
    if rounds < 3 or not 0 <= warmups <= 10000 or not 1 <= operations <= 10000:
        raise ValueError('Requires >=3 rounds, 0..10000 warmups and 1..10000 operations')
    if output.exists():
        raise ValueError('Use a new output file')
    config = json.loads(manifest.read_text())
    if config.get('schemaVersion') != 1 or set(config['commands']) != {'interpreter', 'jit', 'aot'}:
        raise ValueError('Expected version 1 manifest with all three execution modes')
    cases = config['cases'] if selected is None else [selected]
    if not cases or len(set(cases)) != len(cases) or any(c not in config['cases'] for c in cases):
        raise ValueError('Unknown or duplicate benchmark case')
    inputs = dict(config['inputs'])
    for path in [manifest.resolve(), Path(__file__).resolve(), Path(__file__).with_name('process.py').resolve()]:
        inputs[str(path)] = sha(path)
    def check_inputs():
        for path, expected in inputs.items():
            if sha(path) != expected:
                raise ValueError('Benchmark input changed: '+path)
    check_inputs()
    report = dict(schemaVersion=1, passed=False, platform=platform.platform(),
                  measurement='External batch latency including protocol, correctness checks and scheduling; not internal operation timing',
                  rounds=rounds, warmupsPerProcess=warmups, operationsPerProcess=operations,
                  inputs=inputs, commands=config['commands'], samples=[], summary={})
    try:
        modes = ['interpreter', 'jit', 'aot']
        for index in range(rounds):
            for case in cases:
                for mode in modes[index % 3:] + modes[:index % 3]:
                    command = [*config['commands'][mode], *([] if mode == 'aot' else ['--']),
                               case, str(warmups), str(operations)]
                    report["currentJob"] = dict(case=case, mode=mode, round=index, command=command)
                    value = sample(command, operations)
                    value.update(case=case, mode=mode, round=index)
                    report['samples'].append(value)
                    print(f'{case} {mode} round {index}: {value["operationsPerSecond"]:.2f} ops/s', flush=True)
        for case in cases:
            report['summary'][case] = {}
            for mode in modes:
                rows = [x for x in report['samples'] if x['case']==case and x['mode']==mode]
                report['summary'][case][mode] = {
                    key: dict(median=statistics.median(values), minimum=min(values), maximum=max(values))
                    for key in ['startupSeconds', 'batchSeconds', 'operationsPerSecond']
                    for values in [[row[key] for row in rows]]}
        check_inputs()
        report.pop('currentJob', None)
        report['passed'] = True
    except Exception as error:
        report['error'] = str(error)
        raise
    finally:
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(json.dumps(report, indent=2)+'\n')
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--rounds', type=int, default=3)
    parser.add_argument('--warmups', type=int, default=2)
    parser.add_argument('--operations', type=int, default=8)
    parser.add_argument('--filter', help='Exact registered case name')
    args = parser.parse_args()
    run(args.manifest, args.output, args.rounds, args.warmups, args.operations, args.filter)

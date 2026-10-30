#!/usr/bin/env python3
"""Run isolated Raven workers; measure external batch time, including transport."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import statistics

from process import Guest, RETENTION_WARNING
from configuration import resolve, MEASUREMENTS


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def sample(command, operations, timeout=120, mode=None):
    guest = Guest(command, timeout=timeout)
    try:
        _, ready = guest.line(b'NEOBENCH/1 READY')
        _, begin = guest.line(b'NEOBENCH/1 BEGIN')
        _, end = guest.line(b'NEOBENCH/1 END')
        guest.line(str(operations).encode())
        stderr = guest.finish(allowed_stderr=(b'', RETENTION_WARNING) if mode == 'jit' else (b'',))
        if len(guest.output) != 4 or end <= begin:
            raise RuntimeError('Invalid worker output or nonpositive batch duration')
        return dict(stderr=stderr.decode(), startupSeconds=ready-guest.started, warmupSeconds=begin-ready,
                    batchSeconds=end-begin, operations=operations,
                    operationsPerSecond=operations/(end-begin))
    finally:
        guest.close()


def run(manifest, output, rounds=None, warmups=None, operations=None, selected=None,
        config_path=None, measurement=None, modes=None, timeout=None):
    if output.exists():
        raise ValueError('Use a new output file')
    config = json.loads(manifest.read_text())
    if config.get('schemaVersion') != 1 or not isinstance(config.get('commands'), dict):
        raise ValueError('Expected version 1 manifest with mode commands')
    external = None
    if config_path:
        external = json.loads(config_path.read_text())
        if not isinstance(external, dict) or external.get('schemaVersion') != 1:
            raise ValueError('Expected version 1 benchmark configuration')
        external = {key:value for key,value in external.items() if key != 'schemaVersion'}
    overrides = {key:value for key,value in dict(rounds=rounds, warmups=warmups,
        operations=operations, measurement=measurement, modes=modes, timeout=timeout).items() if value is not None}
    effective, origins = resolve(config.get('benchmarkConfig'), external, overrides)
    rounds, warmups, operations = (effective[k] for k in ['rounds', 'warmups', 'operations'])
    modes = effective['modes']
    for mode in modes:
        command = config['commands'].get(mode)
        if not isinstance(command, list) or not command or any(not isinstance(part,str) or not part for part in command):
            raise ValueError('Missing or invalid command for selected mode: '+mode)
    metric, unit, description = MEASUREMENTS[effective['measurement']]
    cases = config['cases'] if selected is None else [selected]
    if not cases or len(set(cases)) != len(cases) or any(c not in config['cases'] for c in cases):
        raise ValueError('Unknown or duplicate benchmark case')
    inputs = dict(config['inputs'])
    for path in [manifest.resolve(), Path(__file__).resolve(), Path(__file__).with_name('process.py').resolve(),
                 Path(__file__).with_name('configuration.py').resolve(), *([config_path.resolve()] if config_path else [])]:
        inputs[str(path)] = sha(path)
    def check_inputs():
        for path, expected in inputs.items():
            if sha(path) != expected:
                raise ValueError('Benchmark input changed: '+path)
    check_inputs()
    report = dict(schemaVersion=2, passed=False, platform=platform.platform(),
                  measurement=description, effectiveConfiguration=effective, configurationOrigins=origins,
                  primaryMetric=dict(name=metric, unit=unit), selectedCases=cases,
                  rounds=rounds, warmupsPerProcess=warmups, operationsPerProcess=operations,
                  inputs=inputs, commands=config['commands'], samples=[], summary={})
    try:
        for index in range(rounds):
            for case in cases:
                for mode in modes[index % len(modes):] + modes[:index % len(modes)]:
                    command = [*config['commands'][mode], *([] if mode == 'aot' else ['--']),
                               case, str(warmups), str(operations)]
                    report["currentJob"] = dict(case=case, mode=mode, round=index, command=command)
                    value = sample(command, operations, timeout=effective['timeout'], mode=mode)
                    value.update(case=case, mode=mode, round=index)
                    report['samples'].append(value)
                    print(f'{case} {mode} round {index}: {value[metric]:.6f} {unit}', flush=True)
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
    parser.add_argument('--config', type=Path, help='Version 1 JSON configuration; overrides manifest defaults')
    parser.add_argument('--measurement', help='startup, batch-latency, batch-throughput, first-invocation')
    parser.add_argument('--mode', action='append', help='Execution mode; repeat to select multiple modes')
    parser.add_argument('--timeout', type=float, help='Seconds per output/exit wait')
    parser.add_argument('--rounds', type=int)
    parser.add_argument('--warmups', type=int)
    parser.add_argument('--operations', type=int)
    parser.add_argument('--filter', help='Exact registered case name')
    args = parser.parse_args()
    run(args.manifest, args.output, args.rounds, args.warmups, args.operations, args.filter,
        args.config, args.measurement, args.mode, args.timeout)

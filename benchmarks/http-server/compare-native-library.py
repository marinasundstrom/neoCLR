#!/usr/bin/env python3
"""Compare two rebuilt plain/JSON HTTP fixtures without profiling instrumentation."""
import argparse
import json
from pathlib import Path
import platform
import statistics
import benchmark


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--baseline', type=Path, required=True, help='Directory with plain/native and json/native builds')
    parser.add_argument('--candidate', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--rounds', type=int, default=7)
    parser.add_argument('--workloads', nargs='+', choices=['plain', 'json', 'bulk'], default=['plain', 'json'])
    args = parser.parse_args()
    if args.rounds < 5:
        parser.error('At least five alternating rounds required')
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, platform=platform.platform(), rounds=args.rounds,
                  workloads=args.workloads, loadAverageBefore=benchmark.load_average(),
                  processesBefore=benchmark.process_snapshot(),
                  scope='AOT HTTP/1.1 loopback; concurrency 1; connection close; 32 requests/process, first eight discarded; one warmup process per fixture; no allocation instrumentation.',
                  limitations=['Short sequential measurements, not saturation throughput or runtime-await qualification.',
                               'Caller must rebuild both fixtures with matching tools and settings; input hashes retain provenance.'],
                  inputs={}, builds={}, warmups=[], samples=[], summary={})
    def retain(path):
        path = path.resolve()
        report['inputs'][str(path)] = benchmark.sha(path)
    retain(Path(__file__))
    retain(Path(benchmark.__file__))
    retain(Path(benchmark.workbench.__file__))
    commands = {}
    try:
        for variant in ('baseline', 'candidate'):
            for case in args.workloads:
                build = getattr(args, variant).resolve() / case / 'native'
                evidence = build / 'build.json'
                data = json.loads(evidence.read_text())
                if not data['passed']:
                    raise ValueError('Build did not pass: ' + str(evidence))
                retain(evidence)
                report['builds'][case + '-' + variant] = {
                    'inputs': data['inputs'], 'artifacts': data['artifacts'],
                    'commands': [{k: c[k] for k in ('phase', 'command', 'exitCode') if k in c}
                                 for c in data['commands']],
                    'profile': data['profile'], 'byteArrayLimit': data['byteArrayLimit'],
                    'httpHeapBytes': data['httpHeapBytes'],
                }
                for name, digest in data['artifacts'].items():
                    path = build / name
                    if benchmark.sha(path) != digest:
                        raise ValueError('Build artifact drift: ' + str(path))
                    retain(path)
                commands[(case, variant)] = [str(build / 'app')]
        for (case, variant), command in commands.items():
            report['warmups'].append(dict(case=case, variant=variant, **benchmark.server_sample(command, case)))
        for index in range(args.rounds):
            for case in args.workloads:
                for variant in (('baseline', 'candidate') if index % 2 == 0 else ('candidate', 'baseline')):
                    report['samples'].append(dict(round=index, case=case, variant=variant,
                        **benchmark.server_sample(commands[(case, variant)], case)))
            print('Round', index + 1, flush=True)
        for case in args.workloads:
            row = {}
            for variant in ('baseline', 'candidate'):
                values = [s['requestsPerSecond'] for s in report['samples'] if s['case'] == case and s['variant'] == variant]
                row[variant] = dict(median=statistics.median(values), min=min(values), max=max(values))
            row['throughputRatio'] = row['candidate']['median'] / row['baseline']['median']
            report['summary'][case] = row
        for path, digest in report['inputs'].items():
            if benchmark.sha(Path(path)) != digest:
                raise ValueError('Input changed during measurements: ' + path)
        report['passed'] = True
    finally:
        report['loadAverageAfter'] = benchmark.load_average()
        (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report['summary'], indent=2))


if __name__ == '__main__':
    main()

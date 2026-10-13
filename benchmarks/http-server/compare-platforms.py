#!/usr/bin/env python3
"""Compare retained HTTP terminal handlers across neoCLR, Kestrel, Go and Node."""
import argparse
import json
from pathlib import Path
import platform
import statistics
import subprocess
import benchmark


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--neoclr', type=Path, required=True, help='Matched library report; use its candidate commands')
    parser.add_argument('--kestrel', type=Path, required=True, help='aspnet/prepare.py build.json')
    parser.add_argument('--go', type=Path, required=True, help='go/prepare.py build.json')
    parser.add_argument('--node', type=Path, required=True, help='node/prepare.py build.json')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    neo = json.loads(args.neoclr.read_text())
    kestrel = json.loads(args.kestrel.read_text())
    go = json.loads(args.go.read_text())
    node = json.loads(args.node.read_text())
    assert neo['passed'] and all(p['prepared'] for p in [kestrel, go, node])
    report = dict(passed=False, commands={}, inputs={}, warmups=[], samples=[], summary={},
                  platform=platform.platform(),
                  scope='Terminal handlers, no endpoint routing; 32 requests/process, discard eight, five rotating rounds. Short-run loopback, not saturation or steady-state JIT.',
                  environment={'dotnetInfo': kestrel['dotnetInfo'],
                               'go': go['version'], 'node': node['versions'],
                               'cpu': subprocess.check_output(['sysctl', '-n', 'machdep.cpu.brand_string'], text=True).strip(),
                               'loadBefore': subprocess.check_output(['uptime'], text=True).strip()})
    report['inputs'].update(neo['inputs'])
    report['inputs'].update(kestrel['inputs'])
    report['inputs'].update(kestrel['outputs'])
    for prepared in [go, node]:
        report['inputs'].update(prepared['inputs'])
        report['inputs'].update(prepared['outputs'])
    for p in [args.neoclr, args.kestrel, args.go, args.node, Path(__file__), Path(benchmark.__file__)]:
        report['inputs'][str(p.resolve())] = benchmark.sha(p)
    for path, digest in report['inputs'].items():
        assert benchmark.sha(Path(path)) == digest, path
    configurations = ['neoclr-jit', 'neoclr-aot', 'kestrel-jit', 'kestrel-aot', 'go-native', 'node-v8']
    for case in ['plain', 'json']:
        for mode in ['jit', 'aot']:
            report['commands'][case+'-neoclr-'+mode] = neo['commands'][case+'-'+mode+'-candidate']
            report['commands'][case+'-kestrel-'+mode] = kestrel['commands'][mode] + [case]
        report['commands'][case+'-go-native'] = go['command'] + [case]
        report['commands'][case+'-node-v8'] = node['command'] + [case]
    try:
        for key, command in report['commands'].items():
            case, configuration = key.split('-', 1)
            report['warmups'].append(dict(case=case, configuration=configuration,
                                         **benchmark.server_sample(command, case)))
        for n in range(5):
            # Rotate the six configurations so each gets a different round position.
            order = configurations[n:] + configurations[:n]
            for case in ['plain', 'json']:
                for configuration in order:
                    row = benchmark.server_sample(report['commands'][case+'-'+configuration], case)
                    report['samples'].append(dict(case=case, configuration=configuration, round=n, **row))
            print('Round', n+1, flush=True)
        for case in ['plain', 'json']:
            row = {}
            for configuration in configurations:
                samples = [s for s in report['samples'] if (s['case'], s['configuration']) == (case, configuration)]
                values = [s['requestsPerSecond'] for s in samples]
                row[configuration] = dict(median=statistics.median(values), min=min(values), max=max(values),
                                          startupMedianSeconds=statistics.median(s['startupSeconds'] for s in samples))
            report['summary'][case] = row
        for path, digest in report['inputs'].items():
            assert benchmark.sha(Path(path)) == digest, path
        report['environment']['loadAfter'] = subprocess.check_output(['uptime'], text=True).strip()
        report['passed'] = True
    finally:
        (out/'results.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report['summary'], indent=2))


if __name__ == '__main__':
    main()

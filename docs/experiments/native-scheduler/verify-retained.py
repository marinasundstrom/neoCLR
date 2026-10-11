#!/usr/bin/env python3
"""Run the retained HTTP contract against an already-built native session host."""
import argparse
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location('retained', ROOT / 'scripts/validate-retained-http.py')
retained = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(retained)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--build', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        raise ValueError('Use a fresh report path')
    build = json.loads((args.build / 'build.json').read_text())
    if not build['passed']:
        raise ValueError('Requires a passing retained HTTP build')
    executable = (args.build / ('app.exe' if (args.build / 'app.exe').exists() else 'app')).resolve()
    report = dict(passed=False, cases=[], inputs=dict(build['inputs']))
    report['inputs'][str(executable)] = retained.sha(executable)
    for path in [Path(__file__).resolve(), ROOT / 'scripts/validate-retained-http.py', ROOT / 'scripts/validate-native-http-project.py']:
        report['inputs'][str(path)] = retained.sha(path)

    def check_inputs():
        for path, digest in report['inputs'].items():
            if retained.sha(Path(path)) != digest:
                raise ValueError('Input changed: ' + path)

    try:
        check_inputs()
        expected = [(b'HTTP/1.1 200 OK\r\nContent-Length: 9\r\nConnection: close\r\n'
                     b'Content-Type: text/plain; charset=utf-8\r\n\r\nRequest ' + str(n).encode()).hex()
                    for n in range(1, 4)]
        for name, targets, fragmented in [('normal', ['/count'] * 3, False),
                                          ('fragmented', ['/count'] * 3, True),
                                          ('fault', ['/count', '/fault'], False)]:
            result = retained.serve([executable], targets, fragmented, ROOT)
            report['cases'].append(dict(name=name, result=result))
            if name == 'fault':
                valid = (result['exitCode'] == 1 and result['stdout'] == 'Completed 1\n'
                         and result['responses'] == [expected[0], '']
                         and 'Retained HTTP handler fault' in result['stderr'])
            else:
                valid = result == dict(exitCode=0, stdout='Completed 1\nCompleted 2\nCompleted 3\n',
                                       stderr='', responses=expected)
            if not valid:
                raise ValueError('Retained HTTP contract failed: ' + name)
        check_inputs()
        report['passed'] = True
    finally:
        args.output.write_text(json.dumps(report, indent=2) + '\n')
    print('Retained scheduler consumer: PASS')


if __name__ == '__main__':
    main()

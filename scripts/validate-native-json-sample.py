#!/usr/bin/env python3
"""Qualify packaged JSON mapping and composed errors in all execution modes."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('bundle', 'aot', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    bundle, backend, output = args.bundle.resolve(), args.aot.resolve(), args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    manifest = json.loads((bundle / 'manifest.json').read_text())
    runtime = bundle / manifest['runtime']
    sample = bundle / 'samples/json'
    report = dict(passed=False, platform=platform.platform(), commands=[], cases=[], inputs={})

    def run(command):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True, timeout=600)
        report['commands'].append(dict(command=result.args, exitCode=result.returncode,
                                       stdout=result.stdout, stderr=result.stderr))
        if result.returncode:
            raise RuntimeError(result.stderr or result.stdout)
        return result

    try:
        native = output / 'native'
        names = manifest['samples']['json']['reflectionTypes']
        run([sys.executable, ROOT / 'scripts/build-native-project.py', '--profile',
             'windows-http' if os.name == 'nt' else 'http', '--project', sample / 'App.rvnproj',
             '--bundle', bundle, '--aot', backend, '--output', native,
             *[arg for name in names for arg in ('--reflection-type', name)]])
        assembly = native / 'app.dll'
        reflection = ['--reflection-roots', native / 'app.reflection.json']
        context = ['--system', bundle / manifest['runtimeSeed'], '--object-root', bundle / manifest['objectRoot']]
        for module in manifest['modules']:
            context += ['--module', bundle / module]
        commands = {
            'interpreter': [runtime, 'run', assembly, '--execution', 'interpreter', '--instructions', '100000000', *context],
            'jit': [runtime, 'run', assembly, '--jit-compiler', backend, *reflection],
            'aot': [native / ('app.exe' if os.name == 'nt' else 'app')],
        }
        for mode, command in commands.items():
            result = run(command)
            if result.stdout != manifest['samples']['json']['expected']:
                raise RuntimeError(mode + ' produced unexpected JSON sample output: ' + result.stdout)
            report['cases'].append(dict(mode=mode, passed=True))
        for path in [Path(__file__), runtime, backend, bundle / 'manifest.json',
                     sample / 'Main.rvn', sample / 'Mapping.rvn', native / 'build.json']:
            report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
        report['passed'] = True
    except Exception as error:
        report['error'] = str(error)
    finally:
        (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(dict(passed=report['passed'], cases=report['cases'], error=report.get('error'))))
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    sys.exit(main())

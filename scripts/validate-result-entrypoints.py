#!/usr/bin/env python3
"""Qualify native Result entry points: process status, stderr, await and conversions."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'runtime/System.Runtime/test/ResultEntryPointTests/ResultEntryPointTests.rvnproj'
CASES = {
    'AsyncInt': [((), 7, ''), (('fail',), 1, 'async int failure\n'), (('null',), 1, '\n')],
    'AsyncUnit': [((), 0, ''), (('fail',), 1, 'interface failure\n')],
    'SyncInt': [((), 9, ''), (('fail',), 1, 'converted failure\n')],
    'AsyncUnion': [((), 0, ''), (('first',), 1, 'first union failure\n'), (('second',), 1, 'second union failure\n')],
    'SyncUnion': [((), 0, ''), (('first',), 1, 'first union failure\n'), (('second',), 1, 'second union failure\n')],
    'PlatformUnion': [((), 0, ''), (('file',), 1, 'NotFound\n'), (('number',), 1, 'NumberParseError.InvalidFormat\n')],
    'ScalarUnion': [((), 8, ''), (('text',), 1, 'scalar union failure\n'), (('number',), 1, '42\n')],
    'SyncUnit': [((), 0, ''), (('fail',), 1, 'sync unit failure\n')],
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', required=True, type=Path)
    parser.add_argument('--runtime', required=True, type=Path)
    parser.add_argument('--aot', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--fixtures', nargs='+', choices=CASES, default=list(CASES))
    args = parser.parse_args()
    out, bundle = args.output.resolve(), args.bundle.resolve()
    out.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, platform=platform.platform(), cases={}, sources={
        str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
        for p in [PROJECT, *PROJECT.parent.rglob('*.rvn')]})
    try:
        for fixture in args.fixtures:
            cases = CASES[fixture]
            env = {**os.environ, 'NeoClrEntryFixture': fixture}
            build = out / fixture
            compiled = subprocess.run([sys.executable, str(ROOT / 'scripts/build-native-project.py'),
                '--profile', 'windows-http' if os.name == 'nt' else 'http', '--project', str(PROJECT),
                '--bundle', str(bundle), '--aot', str(args.aot.resolve()), '--output', str(build)],
                cwd=ROOT, env=env, capture_output=True, text=True, timeout=600)
            (out / (fixture + '-build.log')).write_text(compiled.stdout + compiled.stderr)
            if compiled.returncode:
                raise RuntimeError(f'{fixture} build failed; see {out}')
            lib = bundle / 'lib'
            commands = {
                'native': [str(build / ('app.exe' if os.name == 'nt' else 'app'))],
                'interpreter': [str(args.runtime.resolve()), 'run', str(build / 'app.dll'),
                    '--execution', 'interpreter', '--system', str(lib / 'System.runtime.neox'),
                    '--object-root', str(lib / 'System.Runtime.dll'), '--module', str(lib / 'System.Runtime.dll'), '--'],
            }
            for mode, command in commands.items():
                for arguments, expected_code, expected_error in cases:
                    run = subprocess.run([*command, *arguments], cwd=ROOT, env=env,
                        capture_output=True, text=True, timeout=180)
                    key = fixture + '/' + mode + '/' + (arguments[0] if arguments else 'success')
                    report['cases'][key] = dict(exitCode=run.returncode, stdout=run.stdout, stderr=run.stderr)
                    if (run.returncode, run.stdout, run.stderr) != (expected_code, '', expected_error):
                        raise RuntimeError(f'{key}: {report["cases"][key]}')
                    print('PASS ' + key, flush=True)
        report['passed'] = True
    finally:
        (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()

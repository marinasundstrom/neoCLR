#!/usr/bin/env python3
"""Verify and execute the extracted native POC bundle without a source checkout."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('bundle', nargs='?', type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument('--report', required=True, type=Path)
    args = parser.parse_args()
    root = args.bundle.resolve()
    if args.report.exists():
        raise FileExistsError(args.report)
    report = dict(passed=False, root=str(root), commands=[])
    try:
        manifest = json.loads((root / 'manifest.json').read_text())
        for relative, expected in manifest['files'].items():
            path = (root / relative).resolve()
            if root not in path.parents or not path.is_file():
                raise ValueError('Invalid bundle file: ' + relative)
            if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
                raise ValueError('Bundle hash mismatch: ' + relative)
        report['verifiedFiles'] = len(manifest['files'])
        compiler = root / 'sdk/tools/rvnc/rvnc.dll'
        runtime = root / manifest['runtime']

        def run(command, expected_stdout=None):
            result = subprocess.run([str(p) for p in command], cwd=root, capture_output=True, text=True, timeout=180)
            item = dict(command=[str(p) for p in command], exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr)
            report['commands'].append(item)
            if result.returncode or expected_stdout is not None and (result.stdout != expected_stdout or result.stderr):
                raise RuntimeError('Bundle command failed: ' + json.dumps(item))

        for name, sample in manifest['samples'].items():
            folder = root / 'samples' / name
            run(['dotnet', compiler, 'neoclr', '--project', folder / 'App.rvnproj'])
            assembly = folder / 'bin/neoclr/App.dll'
            if not assembly.is_file():
                raise RuntimeError('Missing native sample output: ' + name)
            if sample.get('expected') is not None:
                run([runtime, 'run', assembly, '--system', root / 'lib/System.neox',
                     '--module', root / 'lib/Numbers.dll', '--module', root / 'lib/Http.dll',
                     '--instructions', '100000000'], sample['expected'])
        http_report = args.report.with_name(args.report.stem + '-http.json')
        run([sys.executable, root / 'tools/verify-native-http-json.py', '--runtime', runtime,
             '--server', root / 'samples/http-json-server/bin/neoclr/App.dll',
             '--client', root / 'samples/http-json-client/bin/neoclr/App.dll',
             '--seed', root / 'lib/System.neox', '--module', root / 'lib/Numbers.dll',
             '--module', root / 'lib/Http.dll', '--output', http_report])
        report['http'] = json.loads(http_report.read_text())
        report['passed'] = True
    except Exception as error:
        report['error'] = str(error)
        raise
    finally:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, indent=2) + '\n')
    print(args.report)


if __name__ == '__main__':
    main()

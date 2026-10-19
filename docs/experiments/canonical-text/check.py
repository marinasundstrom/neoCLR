#!/usr/bin/env python3
"""Check built canonical text consumers against one matched macOS bundle."""
import argparse
import json
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('bundle', type=Path)
parser.add_argument('public', type=Path)
parser.add_argument('null_receiver', type=Path)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
bundle = args.bundle.resolve()
modules = ['System.Runtime', 'System.Data', 'System.Networking', 'System.Web', 'System.Data.PostgreSql']
common = ['--system', str(bundle / 'lib/System.runtime.neox'), '--object-root', str(bundle / 'lib/System.Runtime.dll')]
for module in modules:
    common += ['--module', str(bundle / 'lib' / (module + '.dll'))]
rows = []
for name, directory, fault in [('public', args.public, False), ('null_receiver', args.null_receiver, True)]:
    for engine in ['interpreter', 'jit', 'aot']:
        directory = directory.resolve()
        if engine == 'aot':
            command = [str(directory / 'app')]
        else:
            command = [str(bundle / 'bin/neoclr'), 'run', str(directory / 'app.dll'), '--execution', engine, *common]
            if engine == 'interpreter':
                command += ['--instructions', '100000000']
            if engine == 'jit':
                command += ['--jit-compiler', str(bundle / 'bin/neoclr-codegen'), '--jit-host', str(bundle / 'bin/neoclr-jit-host.dylib')]
        result = subprocess.run(command, text=True, capture_output=True, timeout=120)
        output = result.stdout + result.stderr
        passed = (result.returncode > 0 and 'NullReference' in output) if fault else (result.returncode == 0 and 'PASS canonical text' in output)
        rows.append(dict(fixture=name, engine=engine, command=command, exitCode=result.returncode, passed=passed, stdout=result.stdout, stderr=result.stderr))
        print(f'{name} {engine}: {"PASS" if passed else "FAIL"}', flush=True)
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(json.dumps(dict(passed=all(row['passed'] for row in rows), executions=rows), indent=2) + '\n')
raise SystemExit(0 if all(row['passed'] for row in rows) else 1)

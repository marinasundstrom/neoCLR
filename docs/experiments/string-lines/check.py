#!/usr/bin/env python3
"""Check built string lines consumer against one matched macOS bundle."""
import argparse
import json
import hashlib
import sys
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'benchmarks/framework'))
from process import RETENTION_WARNING

EXPECTED = 'PASS Recognized line endings normalize without rescanning replacements\nPASS Other text bytes survive line ending replacement\nPASS Unchanged line endings retain the original string\nTests: 3, passed: 3, failed: 0, skipped: 0\n'

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('bundle', type=Path)
parser.add_argument('public', type=Path)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
bundle = args.bundle.resolve()
modules = ['System.Runtime', 'System.Data', 'System.Networking', 'System.Web', 'System.Data.PostgreSql']
common = ['--system', str(bundle / 'lib/System.runtime.neox'), '--object-root', str(bundle / 'lib/System.Runtime.dll')]
for module in modules:
    common += ['--module', str(bundle / 'lib' / (module + '.dll'))]
rows = []
for name, directory, fault in [('lines', args.public, False)]:
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
        passed = (result.returncode == 0 and result.stdout == EXPECTED and
                  result.stderr in (('', RETENTION_WARNING.decode()) if engine == 'jit' else ('',)))
        rows.append(dict(fixture=name, engine=engine, command=command, exitCode=result.returncode, passed=passed, stdout=result.stdout, stderr=result.stderr))
        print(f'{name} {engine}: {"PASS" if passed else "FAIL"}', flush=True)
args.output.parent.mkdir(parents=True, exist_ok=True)
inputs = [Path(__file__), *Path(__file__).parent.glob('Main.rvn*'),
          *(ROOT / 'tools/testing/NeoCLR.Testing/NeoClr/Testing').glob('*.rvn'),
          bundle / 'manifest.json', args.public / 'app', args.public / 'app.dll']
args.output.write_text(json.dumps(dict(passed=all(row['passed'] for row in rows), executions=rows,
    inputs={str(p.resolve()): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}), indent=2) + '\n')
raise SystemExit(0 if all(row['passed'] for row in rows) else 1)

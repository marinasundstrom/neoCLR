#!/usr/bin/env python3
"""Run the built-in Decimal storage test in its currently supported interpreter mode."""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parents[3]
bundle = Path(sys.argv[1]).resolve()
output = Path(sys.argv[2]).resolve()
output.mkdir(parents=True, exist_ok=False)
spec = importlib.util.spec_from_file_location('discovery', root / 'scripts/discover-runtime-tests.py')
discovery = importlib.util.module_from_spec(spec)
spec.loader.exec_module(discovery)
project = root / 'runtime/System.Runtime/test/DecimalStorageTests/DecimalStorageTests.rvnproj'
registry = discovery.discover(project, bundle, output / 'discovery')
environment = dict(os.environ, NeoClrBundleRoot=str(bundle), NeoClrTestRegistry=str(registry))
compiled = subprocess.run(['dotnet', str(bundle / 'sdk/tools/rvnc/rvnc.dll'), 'neoclr', '--project', str(project)], env=environment, capture_output=True, text=True, check=True)
(output / 'compile.log').write_text(compiled.stdout + compiled.stderr)
assembly = Path(next(line.removeprefix('Native build output: ') for line in compiled.stdout.splitlines() if line.startswith('Native build output: ')))
discovery.verify_registration(assembly, bundle, output / 'discovery')
lib = bundle / 'lib'
catalog = json.loads((lib / 'bundle.json').read_text())
command = [str(bundle / 'bin/neoclr'), 'run', str(assembly), '--system', str(lib / catalog['runtimeSeed']), *[arg for name in catalog['assemblyNames'] for arg in ('--module', str(lib / (name + '.dll')))], '--object-root', str(lib / 'System.Runtime.dll'), '--execution', 'interpreter', '--instructions', '100000000']
result = subprocess.run(command, capture_output=True, text=True, timeout=300)
expected = 'PASS Decimal GetBits writes only the caller supplied slice\nTests: 1, passed: 1, failed: 0, skipped: 0\n'
passed = result.returncode == 0 and result.stdout == expected and not result.stderr
(output / 'validation.json').write_text(json.dumps(dict(passed=passed, command=command, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr), indent=2) + '\n')
print(result.stdout, end='')
if not passed:
    raise RuntimeError('Decimal storage test failed: ' + result.stderr)

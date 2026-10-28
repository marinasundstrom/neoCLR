#!/usr/bin/env python3
"""Qualify exact socket memory ownership through the production Raven library in all modes."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bundle', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
bundle = args.bundle.resolve()
output = args.output.resolve()
output.mkdir(parents=True, exist_ok=False)
build = output / 'build'
command = [sys.executable, str(ROOT / 'scripts/build-native-project.py'),
           '--profile', 'windows-http' if os.name == 'nt' else 'http',
           '--project', str(Path(__file__).with_name('Tests.rvnproj')),
           '--bundle', str(bundle), '--aot', str(bundle / 'bin' / ('neoclr-codegen.exe' if os.name == 'nt' else 'neoclr-codegen')),
           '--output', str(build)]
result = subprocess.run(command, capture_output=True, timeout=600)
(output / 'build.stdout.log').write_bytes(result.stdout)
(output / 'build.stderr.log').write_bytes(result.stderr)
if result.returncode:
    raise RuntimeError('Socket consumer build failed; see retained logs')
lib = bundle / 'lib'
catalog = json.loads((lib / 'bundle.json').read_text())
runtime = bundle / 'bin' / ('neoclr.exe' if os.name == 'nt' else 'neoclr')
base = [str(runtime), 'run', str(build / 'app.dll'), '--system', str(lib / catalog['runtimeSeed'])]
for name in catalog['assemblyNames']:
    base += ['--module', str(lib / (name + '.dll'))]
base += ['--object-root', str(lib / 'System.Runtime.dll')]
commands = {
    'interpreter': base + ['--execution', 'interpreter'],
    'jit': base + ['--jit-host', str(bundle / 'bin' / ('neoclr-jit-host.dll' if os.name == 'nt' else 'neoclr-jit-host.dylib')),
                   '--jit-compiler', str(bundle / 'bin' / ('neoclr-codegen.exe' if os.name == 'nt' else 'neoclr-codegen'))],
    'aot': [str(build / ('app.exe' if os.name == 'nt' else 'app'))],
}
cases = []
for mode, run in commands.items():
    result = subprocess.run(run, capture_output=True, timeout=120)
    passed = result.returncode == 0 and result.stdout.strip() == b'PASS socket memory windows, snapshots, GC, cancellation and EOF' and result.stderr == b''
    cases.append(dict(mode=mode, passed=passed, command=run, exitCode=result.returncode,
                      stdoutHex=result.stdout.hex(), stderrHex=result.stderr.hex()))
report = dict(passed=all(case['passed'] for case in cases), cases=cases, buildCommand=command,
              runtimeSha256=hashlib.sha256(runtime.read_bytes()).hexdigest(),
              consumerSha256=hashlib.sha256((build / 'app.dll').read_bytes()).hexdigest(),
              compilerSha256=hashlib.sha256((bundle / 'sdk/tools/rvnc/rvnc.dll').read_bytes()).hexdigest())
(output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
if not report['passed']:
    raise RuntimeError('Socket memory contract failed; see validation.json')
print('PASS socket memory ownership and transfers in interpreter/JIT/AOT')

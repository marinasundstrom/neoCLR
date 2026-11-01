#!/usr/bin/env python3
"""Qualify exact HTTP memory ownership through the production Raven library in all modes."""
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
           '--project', str(Path(__file__).with_name('HttpMemoryTests.rvnproj')),
           '--bundle', str(bundle), '--aot', str(bundle / 'bin' / ('neoclr-codegen.exe' if os.name == 'nt' else 'neoclr-codegen')),
           '--output', str(build)]
result = subprocess.run(command, capture_output=True, timeout=600)
(output / 'build.stdout.log').write_bytes(result.stdout)
(output / 'build.stderr.log').write_bytes(result.stderr)
if result.returncode:
    raise RuntimeError('HTTP consumer build failed; see retained logs')
lib = bundle / 'lib'
catalog = json.loads((lib / 'bundle.json').read_text())
runtime = bundle / 'bin' / ('neoclr.exe' if os.name == 'nt' else 'neoclr')
base = [str(runtime), 'run', str(build / 'app.dll'), '--system', str(lib / catalog['runtimeSeed'])]
for name in catalog['assemblyNames']:
    base += ['--module', str(lib / (name + '.dll'))]
base += ['--object-root', str(lib / 'System.Runtime.dll')]
commands = {
    'interpreter': base + ['--execution', 'interpreter', '--instructions', '100000000'],
    'jit': base + ['--jit-host', str(bundle / 'bin' / ('neoclr-jit-host.dll' if os.name == 'nt' else 'neoclr-jit-host.dylib')),
                   '--jit-compiler', str(bundle / 'bin' / ('neoclr-codegen.exe' if os.name == 'nt' else 'neoclr-codegen'))],
    'aot': [str(build / ('app.exe' if os.name == 'nt' else 'app'))],
}
RETENTION_WARNING = b'warning: DAM2004: dynamic-dispatch implementation bodies are not analyzed; reflection inside them may require explicit retention\n'
EXPECTED = (b'PASS HTTP memory windows preserve framing ownership cancellation and read ahead\n'
            b'Tests: 1, passed: 1, failed: 0, skipped: 0\n')
cases = []
for mode, run in commands.items():
    result = subprocess.run(run, capture_output=True, timeout=300)
    diagnostic = RETENTION_WARNING if mode == 'jit' and result.stderr == RETENTION_WARNING else b''
    passed = result.returncode == 0 and result.stdout == EXPECTED and result.stderr == diagnostic
    cases.append(dict(mode=mode, passed=passed, command=run, exitCode=result.returncode,
                      stdoutHex=result.stdout.hex(), stderrHex=result.stderr.hex(),
                      compilerDiagnostic=diagnostic.decode()))
report = dict(passed=all(case['passed'] for case in cases), cases=cases, buildCommand=command,
              testAdapter='Await the shared-server lifecycle, then report through a bound reference owner; not async discovery',
              sources={str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
                       for p in [Path(__file__).resolve(), Path(__file__).with_name('Main.rvn').resolve(),
                                 Path(__file__).with_name('HttpMemoryTests.rvnproj').resolve(),
                                 Path(__file__).parent.resolve() / 'NeoClr/RuntimeTests/Tests.rvn']},
              bundleManifestSha256=hashlib.sha256((bundle / 'manifest.json').read_bytes()).hexdigest(),
              runtimeSha256=hashlib.sha256(runtime.read_bytes()).hexdigest(),
              consumerSha256=hashlib.sha256((build / 'app.dll').read_bytes()).hexdigest(),
              compilerSha256=hashlib.sha256((bundle / 'sdk/tools/rvnc/rvnc.dll').read_bytes()).hexdigest())
(output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
if not report['passed']:
    raise RuntimeError('HTTP memory contract failed; see validation.json')
print('PASS HTTP parser and body windows in interpreter/JIT/AOT')

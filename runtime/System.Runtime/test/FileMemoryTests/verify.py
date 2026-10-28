#!/usr/bin/env python3
"""Compile the Raven file consumer and exercise the interpreter's Memory boundary."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bundle', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
bundle = args.bundle.resolve()
output = args.output.resolve()
output.mkdir(parents=True, exist_ok=False)
project = Path(__file__).with_name('Tests.rvnproj').resolve()
compiler = bundle / 'sdk/tools/rvnc/rvnc.dll'
command = ['dotnet', str(compiler), 'neoclr', '--project', str(project)]
compiled = subprocess.run(command, env=dict(os.environ, NeoClrBundleRoot=str(bundle)),
                          capture_output=True, text=True, timeout=120)
(output / 'compile.stdout.log').write_text(compiled.stdout)
(output / 'compile.stderr.log').write_text(compiled.stderr)
if compiled.returncode:
    raise RuntimeError('File consumer compilation failed; see retained logs')
assemblies = [line.removeprefix('Native build output: ') for line in compiled.stdout.splitlines()
              if line.startswith('Native build output: ')]
if len(assemblies) != 1 or not Path(assemblies[0]).is_file():
    raise RuntimeError('Expected one compiled file consumer')
lib = bundle / 'lib'
catalog = json.loads((lib / 'bundle.json').read_text())
runtime = bundle / 'bin' / ('neoclr.exe' if os.name == 'nt' else 'neoclr')
run = [str(runtime), 'run', assemblies[0], '--system', str(lib / catalog['runtimeSeed'])]
for name in catalog['assemblyNames']:
    run += ['--module', str(lib / (name + '.dll'))]
run += ['--object-root', str(lib / 'System.Runtime.dll'), '--execution', 'interpreter']
with tempfile.TemporaryDirectory(prefix='neoclr-file-memory-') as temporary:
    path = Path(temporary) / 'bytes.bin'
    run += ['--', str(path)]
    result = subprocess.run(run, capture_output=True, text=True, timeout=120)
    contents = path.read_bytes().hex() if path.exists() else None
    passed = (result.returncode == 0 and contents == '2802' and result.stderr == '' and
              result.stdout.strip() == 'PASS file memory windows, EOF, sentinels, ranges and cancellation')
    report = dict(passed=passed, compileCommand=command, command=run,
                  exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr,
                  fileHex=contents, runtimeSha256=hashlib.sha256(runtime.read_bytes()).hexdigest(),
                  compilerSha256=hashlib.sha256(compiler.read_bytes()).hexdigest(),
                  consumerSha256=hashlib.sha256(Path(assemblies[0]).read_bytes()).hexdigest())
    (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    if not passed:
        raise RuntimeError('File memory contract failed; see validation.json')
print('PASS file memory consumer and exact file contents')

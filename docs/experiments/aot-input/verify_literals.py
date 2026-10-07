#!/usr/bin/env python3
"""Compile Raven immutable UTF-8 literal transport into a standalone native executable."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]
p = argparse.ArgumentParser(description=__doc__)
for key in ('runtime', 'aot', 'bundle', 'output'):
    p.add_argument('--' + key, type=Path, required=True)
p.add_argument('--compiler', type=Path, required=True)
p.add_argument('--compiler-revision', required=True)
a = p.parse_args()
compiler = a.compiler.resolve()
runtime, aot, bundle, output = (getattr(a, k).resolve() for k in ('runtime', 'aot', 'bundle', 'output'))
output.mkdir(parents=True, exist_ok=False)
source = ROOT / 'docs/experiments/aot-input/literals.rvn'
probe = ROOT / 'docs/experiments/aot-input/ReadByte.pe'
host = ROOT / 'docs/experiments/aot-hello/main.c'
seed, library = (bundle / 'lib' / n for n in ('System.runtime.neox', 'System.Runtime.dll'))
sha = lambda b: hashlib.sha256(b).hexdigest()
report = dict(profile='aot-literal-transport-v1', declaredCompilerRevision=a.compiler_revision, SDKROOT=os.environ.get('SDKROOT'),
              baseRevision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              inputs={str(f): sha(f.read_bytes()) for f in (runtime, aot, source, probe, host, seed, library, compiler, compiler.parent / 'Raven.CodeAnalysis.dll', compiler.parent / 'Raven.CodeAnalysis.NeoClr.dll', compiler.parent / 'NeoCLR.Metadata.Experimental.dll')}, commands=[])
def save():
    (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
def run(args, success=True, expected=None):
    r = subprocess.run(list(map(str, args)), cwd=ROOT, capture_output=True, text=True, timeout=120)
    report['commands'].append(dict(command=r.args, exit=r.returncode, stderr=r.stderr,
        stdout=r.stdout if len(r.stdout) < 2000 else dict(bytes=len(r.stdout.encode()), sha256=sha(r.stdout.encode()))))
    save()
    assert (r.returncode == expected if expected is not None else (r.returncode == 0) == success), r.stderr
    return r.stdout
assembly = output / 'Literals.dll'
run(['dotnet', compiler, 'neoclr', '-o', assembly, source])
assert run([runtime, 'run', assembly], expected=42) == ''
inspection = json.loads(run([aot, '--inspect', assembly, '@entry', '--closed-world']))
assert inspection['admission']['accepted'] is True
obj, binary = output / 'transport.o', output / 'transport'
selection = json.loads(run([aot, '--closed-world', assembly, '@entry', obj]))
assert selection == inspection['selection']
report['selection'] = selection
run(['clang', '-arch', 'arm64', '-Wall', '-Wextra', '-Werror', host, obj, '-o', binary])
assert run(['nm', '-u', obj]) == ''
dependencies = [line.split()[0] for line in run(['otool', '-L', binary]).splitlines()[1:]]
assert dependencies == ['/usr/lib/libSystem.B.dylib']
with tempfile.TemporaryDirectory(prefix='neoclr-erased-transport-') as directory:
    installed = Path(directory) / 'transport'
    shutil.copy2(binary, installed)
    r = subprocess.run([installed], cwd=directory, env={}, capture_output=True, timeout=10)
    assert r.returncode == 42 and r.stdout == b'' and r.stderr == b''
report['native'] = dict(executableOnlyDirectory=True, emptyEnvironment=True, exit=42, dynamicDependencies=dependencies)
context = ['--system', seed, '--module', library, '--object-root', library, '--compile-system']
boundary = json.loads(run([aot, '--inspect', probe, '@entry', '--closed-world'] + context))['admission']
assert boundary['accepted'] is False and boundary['phase'] == 'compilation'
assert boundary['firstError'] == 'neoCLR.Runtime.ConsoleReadByte: unsupported value member contract'
run([aot, '--closed-world', probe, '@entry', output / 'read-byte.o'] + context, success=False)
assert not (output / 'read-byte.o').exists()
report['readByteBoundary'] = boundary
report['artifacts'] = {f.name: sha(f.read_bytes()) for f in (assembly, obj, binary)}
save()
print('Passed: Raven literals standalone execution and ReadByte native-service boundary')

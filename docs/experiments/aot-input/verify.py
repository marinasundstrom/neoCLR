#!/usr/bin/env python3
"""Reproduce nested input outcomes and the real Console.ReadByte admission boundary."""
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
for key in ('compiler', 'bundle', 'runtime', 'aot', 'output'):
    p.add_argument('--' + key, type=Path, required=True)
p.add_argument('--compiler-revision', required=True)
a = p.parse_args()
compiler, bundle, runtime, aot, output = (getattr(a, key).resolve() for key in ('compiler', 'bundle', 'runtime', 'aot', 'output'))
output.mkdir(parents=True, exist_ok=False)
core, seed, library, ownership = (bundle / 'lib' / n for n in ('Core.dll', 'System.runtime.neox', 'System.Runtime.dll', 'ownership.json'))
sources = [ROOT / 'docs/experiments/aot-input' / n for n in ('outcomes.rvn', 'read-byte.rvn')]
host = ROOT / 'docs/experiments/aot-hello/main.c'
inputs = [compiler, core, seed, library, ownership, runtime, aot, host] + sources
inputs += [compiler.parent / n for n in ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
sha = lambda b: hashlib.sha256(b).hexdigest()
report = dict(profile='aot-input-outcomes-v1', baseRevision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              declaredCompilerRevision=a.compiler_revision, SDKROOT=os.environ.get('SDKROOT'),
              inputs={str(f): sha(f.read_bytes()) for f in inputs}, commands=[])
def save():
    (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
def run(command, data='', success=True):
    r = subprocess.run(list(map(str, command)), cwd=ROOT, input=data, capture_output=True, text=True, timeout=120)
    report['commands'].append(dict(command=r.args, stdin=data, exit=r.returncode, stderr=r.stderr,
        stdout=r.stdout if len(r.stdout) < 2000 else dict(bytes=len(r.stdout.encode()), sha256=sha(r.stdout.encode()))))
    save()
    assert (r.returncode == 0) == success, r.stderr
    return r.stdout
context = ['--system', seed, '--module', library, '--object-root', library]
for source in sources:
    assembly = output / (source.stem + '.dll')
    run(['dotnet', compiler, 'neoclr', '--core-reference', core, '--runtime-seed', seed,
         '--reference', library, '--bootstrap-intrinsics', '--bootstrap-ownership', ownership,
         '--object-library', 'System.Runtime', '-o', assembly, source])
    assert run([runtime, 'run', assembly] + context) == ''
    if source.stem == 'read-byte':
        assert run([runtime, 'run', assembly] + context, '*') == ''
        run([runtime, 'run', assembly] + context, 'x', success=False)
    inspection = json.loads(run([aot, '--inspect', assembly, '@entry', '--closed-world'] + context))
    obj = output / (source.stem + '.o')
    if source.stem == 'read-byte':
        assert inspection['admission']['accepted'] is False
        assert inspection['admission']['phase'] == 'selection'
        assert 'reference=true' in inspection['admission']['firstError']
        assert 'T_53797374656D_436F6E736F6C65;' in inspection['admission']['firstError']
        run([aot, '--closed-world', assembly, '@entry', obj] + context, success=False)
        assert not obj.exists()
        report['readByteBoundary'] = inspection['admission']
        continue
    assert inspection['admission']['accepted'] is True, inspection['admission']
    selection = json.loads(run([aot, '--closed-world', assembly, '@entry', obj] + context))
    assert selection == inspection['selection']
    relationships = selection['verifiedInterfaceRelationships']
    assert len(relationships) == 2
    assert 'TypeParameter' not in json.dumps(relationships)
    assert any(row['interfaces'][0]['Constructed']['arguments'][-1] == 'Void' for row in relationships)
    report['selection'] = {key: selection[key] for key in ('functions', 'types', 'specialization', 'verifiedInterfaceRelationships', 'interfacePolicy')}
    binary = output / 'outcomes'
    run(['clang', '-arch', 'arm64', '-Wall', '-Wextra', '-Werror', host, obj, '-o', binary])
    assert run(['nm', '-u', obj]) == ''
    dependencies = [line.split()[0] for line in run(['otool', '-L', binary]).splitlines()[1:]]
    assert dependencies == ['/usr/lib/libSystem.B.dylib']
    with tempfile.TemporaryDirectory(prefix='neoclr-input-outcomes-') as directory:
        installed = Path(directory) / 'outcomes'
        shutil.copy2(binary, installed)
        r = subprocess.run([installed], cwd=directory, env={}, capture_output=True, timeout=10)
        assert r.returncode == 0 and r.stdout == b'' and r.stderr == b''
    report['native'] = dict(executableOnlyDirectory=True, emptyEnvironment=True, exit=0, dynamicDependencies=dependencies)
report['artifacts'] = {f.name: sha(f.read_bytes()) for f in output.iterdir() if f.suffix in ('.dll', '.o') or f.name == 'outcomes'}
save()
print('Passed: nested input outcomes native execution and real input admission boundary')

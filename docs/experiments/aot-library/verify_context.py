#!/usr/bin/env python3
"""Check standalone Pair and library Result with an explicit runtime-owned Object context."""
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
for key in ('aot', 'runtime', 'bundle', 'output'):
    p.add_argument('--' + key, type=Path, required=True)
a = p.parse_args()
aot, runtime, bundle, output = (getattr(a, key).resolve() for key in ('aot', 'runtime', 'bundle', 'output'))
output.mkdir(parents=True, exist_ok=False)
seed, library = bundle / 'lib/System.runtime.neox', bundle / 'lib/System.Runtime.dll'
app, values = (ROOT / 'docs/experiments/aot-library' / n for n in ('GenericApp.pe', 'GenericValues.pe'))
result = ROOT / 'docs/experiments/aot-values/LibraryResultApp.pe'
host = ROOT / 'docs/experiments/aot-hello/main.c'
sha = lambda data: hashlib.sha256(data).hexdigest()
report = dict(profile='aot-runtime-context-v1', inputs={str(f): sha(f.read_bytes()) for f in (aot, runtime, seed, library, app, values, result, host)},
              baseRevision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              SDKROOT=os.environ.get('SDKROOT'), commands=[])
def save():
    (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
def run(command, success=True):
    r = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True, timeout=120)
    report['commands'].append(dict(command=r.args, exit=r.returncode, stderr=r.stderr,
                                  stdout=r.stdout if len(r.stdout) < 2000 else dict(bytes=len(r.stdout.encode()), sha256=sha(r.stdout.encode()))))
    save()
    assert (r.returncode == 0) == success, r.stderr
    return r.stdout
context = ['--system', seed, '--module', library, '--object-root', library]
assert run([runtime, 'run', app, '--module', values] + context) == ''
assert run([runtime, 'run', result] + context) == ''
inspection = json.loads(run([aot, '--inspect', app, '@entry', '--closed-world', '--module', values] + context))
assert inspection['admission']['accepted'] is True
obj, binary = output / 'pair.o', output / 'pair'
selection = json.loads(run([aot, '--closed-world', app, '@entry', obj, '--module', values] + context))
assert selection == inspection['selection']
report['selected'] = {key: selection[key] for key in ('loadSet', 'functions', 'types', 'specialization')}
report['excludedCounts'] = {key: len(selection[key]) for key in ('excludedFunctions', 'excludedTypes')}
run(['clang', '-arch', 'arm64', '-Wall', '-Wextra', '-Werror', host, obj, '-o', binary])
assert run(['nm', '-u', obj]) == ''
dependencies = [line.split()[0] for line in run(['otool', '-L', binary]).splitlines()[1:]]
assert dependencies == ['/usr/lib/libSystem.B.dylib'], dependencies
with tempfile.TemporaryDirectory(prefix='neoclr-aot-context-') as directory:
    installed = Path(directory) / 'pair'
    shutil.copy2(binary, installed)
    r = subprocess.run([installed], cwd=directory, env={}, capture_output=True, timeout=10)
    assert r.returncode == 0 and r.stdout == b'' and r.stderr == b''
probe = json.loads(run([aot, '--inspect', result, '@entry', '--closed-world'] + context))
assert probe['admission']['accepted'] is True
result_obj, result_binary = output / 'result.o', output / 'result'
result_selection = json.loads(run([aot, '--closed-world', result, '@entry', result_obj] + context))
assert result_selection == probe['selection']
assert result_selection['verifiedInterfaceRelationships']
run(['clang', '-arch', 'arm64', '-Wall', '-Wextra', '-Werror', host, result_obj, '-o', result_binary])
assert run(['nm', '-u', result_obj]) == ''
result_dependencies = [line.split()[0] for line in run(['otool', '-L', result_binary]).splitlines()[1:]]
assert result_dependencies == ['/usr/lib/libSystem.B.dylib']
with tempfile.TemporaryDirectory(prefix='neoclr-aot-result-') as directory:
    installed = Path(directory) / 'result'
    shutil.copy2(result_binary, installed)
    r = subprocess.run([installed], cwd=directory, env={}, capture_output=True, timeout=10)
    assert r.returncode == 0 and r.stdout == b'' and r.stderr == b''
report['resultProbe'] = dict(admission=probe['admission'],
    selection={key: result_selection[key] for key in ('functions', 'types', 'specialization', 'verifiedInterfaceRelationships', 'interfacePolicy')},
    executableOnlyDirectory=True, emptyEnvironment=True, exit=0, dynamicDependencies=result_dependencies,
    artifacts={f.name: sha(f.read_bytes()) for f in (result_obj, result_binary)})
report['native'] = dict(executableOnlyDirectory=True, emptyEnvironment=True, exit=0, dynamicDependencies=dependencies,
                       artifacts={f.name: sha(f.read_bytes()) for f in (obj, binary)})
save()
print('Passed: explicit runtime context, standalone value executable, and standalone library Result executable')

#!/usr/bin/env python3
"""Validate bounded erased CIL transport and the next actual ReadByte boundary."""
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
a = p.parse_args()
runtime, aot, bundle, output = (getattr(a, k).resolve() for k in ('runtime', 'aot', 'bundle', 'output'))
output.mkdir(parents=True, exist_ok=False)
source = ROOT / 'docs/experiments/aot-input/erased-transport.neoil'
probe = ROOT / 'docs/experiments/aot-input/ReadByte.pe'
host = ROOT / 'docs/experiments/aot-hello/main.c'
seed, library = (bundle / 'lib' / n for n in ('System.runtime.neox', 'System.Runtime.dll'))
sha = lambda b: hashlib.sha256(b).hexdigest()
report = dict(profile='aot-erased-primitive-transport-v1', SDKROOT=os.environ.get('SDKROOT'),
              baseRevision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              inputs={str(f): sha(f.read_bytes()) for f in (runtime, aot, source, probe, host, seed, library)}, commands=[])
def save():
    (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
def run(args, success=True):
    r = subprocess.run(list(map(str, args)), cwd=ROOT, capture_output=True, text=True, timeout=120)
    report['commands'].append(dict(command=r.args, exit=r.returncode, stderr=r.stderr,
        stdout=r.stdout if len(r.stdout) < 2000 else dict(bytes=len(r.stdout.encode()), sha256=sha(r.stdout.encode()))))
    save()
    assert (r.returncode == 0) == success, r.stderr
    return r.stdout
assert run([runtime, 'run', source]) == ''
inspection = json.loads(run([aot, '--inspect', source, '@entry', '--closed-world']))
assert inspection['admission']['accepted'] is True
obj, binary = output / 'transport.o', output / 'transport'
selection = json.loads(run([aot, '--closed-world', source, '@entry', obj]))
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
    assert r.returncode == 0 and r.stdout == b'' and r.stderr == b''
report['native'] = dict(executableOnlyDirectory=True, emptyEnvironment=True, exit=0, dynamicDependencies=dependencies)
context = ['--system', seed, '--module', library, '--object-root', library]
boundary = json.loads(run([aot, '--inspect', probe, '@entry', '--closed-world'] + context))['admission']
assert boundary == dict(accepted=False, phase='selection', firstError='generic methods are outside this specialization profile')
run([aot, '--closed-world', probe, '@entry', output / 'read-byte.o'] + context, success=False)
assert not (output / 'read-byte.o').exists()
report['readByteBoundary'] = boundary
report['artifacts'] = {f.name: sha(f.read_bytes()) for f in (obj, binary)}
save()
print('Passed: standalone erased transport and actual ReadByte generic-method boundary')

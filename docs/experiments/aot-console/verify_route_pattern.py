#!/usr/bin/env python3
"""Qualify the real RoutePattern consumer, recording admission failures as dependencies."""
import argparse
import hashlib
import json
import os
import shutil
import tempfile
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
parser = argparse.ArgumentParser(description=__doc__)
for key in ('compiler', 'runtime', 'aot', 'bundle', 'output'):
    parser.add_argument('--' + key, type=Path, required=True)
a = parser.parse_args()
compiler, runtime, aot, bundle, output = (getattr(a, key).resolve() for key in ('compiler', 'runtime', 'aot', 'bundle', 'output'))
output.mkdir(parents=True, exist_ok=False)
source = Path(__file__).with_name('route-pattern.rvn')
seed, core, ownership = (bundle / 'lib' / name for name in ('System.runtime.neox', 'Core.dll', 'ownership.json'))
libraries = [bundle / 'lib' / name for name in ('System.Runtime.dll', 'System.Web.dll', 'System.Networking.dll', 'System.Data.dll')]
inputs = {compiler, runtime, aot, source, seed, core, ownership, Path(__file__).resolve(), *libraries,
          *compiler.parent.glob('*.dll'), *compiler.parent.glob('*.deps.json'), *compiler.parent.glob('*.runtimeconfig.json')}
report = dict(profile='route-pattern-standalone-v1', baseRevision=subprocess.check_output(
    ['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), SDKROOT=os.environ.get('SDKROOT'),
    inputs={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(inputs)}, commands=[])

def save():
    (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')

def run(command):
    result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, timeout=180)
    report['commands'].append(dict(command=result.args, exit=result.returncode,
                                   stdout=result.stdout.decode() if len(result.stdout) < 4000 else dict(
                                       bytes=len(result.stdout), sha256=hashlib.sha256(result.stdout).hexdigest()),
                                   stderr=result.stderr.decode()))
    save()
    assert result.returncode == 0, result.stderr
    return result

assembly = output / 'Route.dll'
run(['dotnet', compiler, 'neoclr', '--core-reference', core, '--runtime-seed', seed,
     *[item for lib in libraries for item in ('--reference', lib)], '--bootstrap-intrinsics',
     '--bootstrap-ownership', ownership, '--object-library', 'System.Runtime', '-o', assembly, source])
report['assemblySha256'] = hashlib.sha256(assembly.read_bytes()).hexdigest()
context = ['--system', seed, *[item for lib in libraries for item in ('--module', lib)], '--object-root', libraries[0]]
result = run([runtime, 'run', assembly, *context, '--instructions', '100000000'])
assert result.stdout == b'42\n' and not result.stderr, result
flags = [*context, '--compile-system', '--bind-user-fault', '--bind-console-write-line',
         '--bind-utf8-text', '--bind-integer-text', '--bind-int32-to-string', '--reference-arena']
inspection = json.loads(run([aot, '--inspect', assembly, '@entry', '--closed-world', *flags]).stdout)
report['admission'] = inspection['admission']
save()
assert report['admission']['accepted'], report['admission']
obj, binary = output / 'route.o', output / 'route'
run([aot, '--closed-world', assembly, '@entry', obj, *flags])
base = Path(__file__).resolve().parent
faults = base.parent / 'aot-fault-details'
adapters = [base / 'text-host.c', faults / 'render.c', base / 'console.c', base.parent / 'aot-scalar/console.c', base / 'text-arena.c']
for path in [base / 'text-arena.h', *adapters]:
    report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
run(['clang', '-arch', 'arm64', '-std=c11', '-Wall', '-Wextra', '-Werror', *adapters, obj, '-o', binary])
imports = set(run(['nm', '-u', obj]).stdout.decode().split())
report['nativeImports'] = sorted(imports)
deps = [line.split()[0] for line in run(['otool', '-L', binary]).stdout.decode().splitlines()[1:]]
assert deps == ['/usr/lib/libSystem.B.dylib'], deps
with tempfile.TemporaryDirectory() as folder:
    installed = Path(folder) / 'route'
    shutil.copy2(binary, installed)
    native = subprocess.run([installed], cwd=folder, env={}, capture_output=True, timeout=10)
    assert (native.returncode, native.stdout, native.stderr) == (0, result.stdout, b''), native
    read_end, write_end = os.pipe()
    os.close(read_end)
    try:
        failed = subprocess.run([runtime, 'run', assembly, *context, '--instructions', '100000000'],
            cwd=ROOT, capture_output=True, pass_fds=(write_end,), preexec_fn=lambda: os.dup2(write_end, 1), timeout=180)
        native_failed = subprocess.run([installed], cwd=folder, env={}, capture_output=True,
            pass_fds=(write_end,), preexec_fn=lambda: os.dup2(write_end, 1), timeout=10)
        assert failed.returncode == native_failed.returncode == 1
        assert failed.stderr == native_failed.stderr and failed.stderr and not failed.stdout and not native_failed.stdout
    finally:
        os.close(write_end)
report['native'] = dict(stdout=native.stdout.decode(), dynamicDependencies=deps, executableOnlyDirectory=True,
    emptyEnvironment=True, brokenPipeFault=failed.stderr.decode(), faultMatchesInterpreter=True,
    executableSha256=hashlib.sha256(binary.read_bytes()).hexdigest())
save()
print('Passed: standalone RoutePattern Parse/Match/GetInt32 and exact fault parity')

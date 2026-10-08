#!/usr/bin/env python3
"""Qualify route outcomes and measure retained native allocations across requests."""
import argparse
import hashlib
import json
import os
import re
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
source = Path(__file__).with_name('route-lifetime.rvn')
seed, core, ownership = (bundle / 'lib' / name for name in ('System.runtime.neox', 'Core.dll', 'ownership.json'))
libraries = [bundle / 'lib' / name for name in ('System.Runtime.dll', 'System.Web.dll', 'System.Networking.dll', 'System.Data.dll')]
inputs = {compiler, runtime, aot, source, seed, core, ownership, Path(__file__).resolve(), *libraries,
          *compiler.parent.glob('*.dll'), *compiler.parent.glob('*.deps.json'), *compiler.parent.glob('*.runtimeconfig.json')}
report = dict(profile='route-lifetime-diagnostic-v1', baseRevision=subprocess.check_output(
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
result = run([runtime, 'run', assembly, *context, '--instructions', '100000000', '--gc-stats', '--gc-events'])
assert result.stdout == b'16\n', result
stats = re.search(r'GC: allocated=(\d+) live=(\d+) peak=(\d+) collections=(\d+) reclaimed=(\d+)', result.stderr.decode())
assert stats, result.stderr
report['interpreterGC'] = dict(zip(('allocated', 'live', 'peak', 'collections', 'reclaimed'), map(int, stats.groups())))
report['interpreterGC']['events'] = result.stderr.decode().splitlines()[1:]
save()
flags = [*context, '--compile-system', '--bind-user-fault', '--bind-console-write-line',
         '--bind-utf8-text', '--bind-integer-text', '--bind-int32-to-string', '--reference-arena']
inspection = json.loads(run([aot, '--inspect', assembly, '@entry', '--closed-world', *flags]).stdout)
report['admission'] = inspection['admission']
save()
assert report['admission']['accepted'], report['admission']
entry = next(f for f in inspection['functions'] if f['name'] == inspection['root'])
roots = [call['target'] for call in entry['calls'] if call['target']['parameters'] == ['Int32'] and not call['target'].get('instance')]
assert len(roots) == 1, roots
native_root = roots[0]['name']
report['nativeRoot'] = native_root
obj, binary = output / 'route.o', output / 'route'
run([aot, '--closed-world', assembly, native_root, obj, *flags])
base = Path(__file__).resolve().parent
faults = base.parent / 'aot-fault-details'
adapters = [base / 'route-lifetime-host.c', faults / 'render.c', base / 'console.c', base.parent / 'aot-scalar/console.c', base / 'text-arena.c']
for path in [base / 'text-arena.h', *adapters]:
    report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
run(['clang', '-arch', 'arm64', '-std=c11', '-Wall', '-Wextra', '-Werror', *adapters, obj, '-o', binary])
entry_obj, entry_binary = output / 'entry.o', output / 'entry'
run([aot, '--closed-world', assembly, '@entry', entry_obj, *flags])
run(['clang', '-arch', 'arm64', '-std=c11', '-Wall', '-Wextra', '-Werror', *adapters, entry_obj, '-o', entry_binary])
imports = set(run(['nm', '-u', obj]).stdout.decode().split())
report['nativeImports'] = sorted(imports)
deps = [line.split()[0] for line in run(['otool', '-L', binary]).stdout.decode().splitlines()[1:]]
assert deps == ['/usr/lib/libSystem.B.dylib'], deps
entry_deps = [line.split()[0] for line in run(['otool', '-L', entry_binary]).stdout.decode().splitlines()[1:]]
assert entry_deps == deps, entry_deps
with tempfile.TemporaryDirectory() as folder:
    installed = Path(folder) / 'route'
    shutil.copy2(binary, installed)
    installed_entry = Path(folder) / 'entry'
    shutil.copy2(entry_binary, installed_entry)
    native = subprocess.run([installed_entry], cwd=folder, env={}, capture_output=True, timeout=10)
    assert (native.returncode, native.stdout, native.stderr) == (0, result.stdout, b''), native
    measurements = []
    for count in (0, 1, 8, 16, 32, 64, 128):
        measured = subprocess.run([installed, str(count), '1048576', 'audit'], cwd=folder, env={}, capture_output=True, timeout=20)
        assert measured.returncode == 0 and measured.stdout == f'{count}\n'.encode(), measured
        measurements.append(json.loads(measured.stderr.decode().removeprefix('AOT_MEASURE ')))
    assert all(a['used'] < b['used'] for a, b in zip(measurements, measurements[1:])), measurements
    limited = subprocess.run([installed, '128', '65536', 'audit'], cwd=folder, env={}, capture_output=True, timeout=20)
    assert limited.returncode == 1 and not limited.stdout, limited
    diagnostic, measured = limited.stderr.decode().rsplit('AOT_MEASURE ', 1)
    exhausted = json.loads(measured)
    assert exhausted['status'] == 5 and exhausted['result'] == -99 and exhausted['used'] <= 65536, exhausted
    assert diagnostic.startswith('NativeMemoryLimitExceeded:'), diagnostic
    report['lifetime'] = dict(measurements=measurements, fixedBudgetFailure=exhausted, fault=diagnostic)
    save()
    read_end, write_end = os.pipe()
    os.close(read_end)
    try:
        failed = subprocess.run([runtime, 'run', assembly, *context, '--instructions', '100000000'],
            cwd=ROOT, capture_output=True, pass_fds=(write_end,), preexec_fn=lambda: os.dup2(write_end, 1), timeout=180)
        native_failed = subprocess.run([installed_entry], cwd=folder, env={}, capture_output=True,
            pass_fds=(write_end,), preexec_fn=lambda: os.dup2(write_end, 1), timeout=10)
        assert failed.returncode == native_failed.returncode == 1
        assert failed.stderr == native_failed.stderr and failed.stderr and not failed.stdout and not native_failed.stdout
    finally:
        os.close(write_end)
report['native'] = dict(stdout=native.stdout.decode(), dynamicDependencies=deps, executableOnlyDirectory=True,
    emptyEnvironment=True, brokenPipeFault=failed.stderr.decode(), faultMatchesInterpreter=True,
    executableSha256=hashlib.sha256(entry_binary.read_bytes()).hexdigest(),
    measurementExecutableSha256=hashlib.sha256(binary.read_bytes()).hexdigest())
save()
print('Passed: route outcomes, retained allocation measurements and exact output-fault parity')

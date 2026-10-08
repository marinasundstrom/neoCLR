#!/usr/bin/env python3
"""Compile and execute the same Raven listener lifecycle in both modes; no timing."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
p = argparse.ArgumentParser(description=__doc__)
for key in ('compiler', 'runtime', 'aot', 'bundle', 'output'):
    p.add_argument('--' + key, type=Path, required=True)
a = p.parse_args()
compiler, runtime, aot, bundle, output = (getattr(a, k).resolve() for k in ('compiler', 'runtime', 'aot', 'bundle', 'output'))
output.mkdir(parents=True, exist_ok=False)
base = ROOT / 'docs/experiments/aot-console'
seed, core, ownership = (bundle / 'lib' / n for n in ('System.runtime.neox', 'Core.dll', 'ownership.json'))
libs = [bundle / 'lib' / n for n in ('System.Runtime.dll', 'System.Web.dll', 'System.Networking.dll', 'System.Data.dll')]
context = ['--system', seed, *[x for lib in libs for x in ('--module', lib)], '--object-root', libs[0]]
flags = [*context, '--compile-system', '--bind-user-fault', '--reference-arena', '--native-gc']
adapters = [base / 'socket-host.c', base / 'socket-listener.c', base / 'root-probe.c', base / 'native-gc.c',
            base / 'text-arena.c', base.parent / 'aot-fault-details/render.c']
report = {'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
          'SDKROOT': os.environ.get('SDKROOT'), 'inputs': {}, 'commands': [], 'cases': {}}

def save():
    (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')

def run(command, status=0):
    r = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True, timeout=180)
    report['commands'].append({'command': r.args, 'exit': r.returncode,
                              'stdout': r.stdout if len(r.stdout) < 4000 else {'sha256': hashlib.sha256(r.stdout.encode()).hexdigest()}, 'stderr': r.stderr})
    save()
    if r.returncode != status:
        raise RuntimeError(f'{r.args}: {r.returncode}: {r.stderr}')
    return r

for f in [Path(__file__), compiler, runtime, aot, core, seed, ownership, *libs, *adapters,
          *base.glob('*.h'), *base.parent.joinpath('aot-fault-details').glob('*.h'), *compiler.parent.glob('*.dll')]:
    report['inputs'][str(f)] = hashlib.sha256(f.read_bytes()).hexdigest()
for name in ('Listen', 'ListenFault'):
    source = Path(__file__).with_name(name + '.rvn')
    report['inputs'][str(source)] = hashlib.sha256(source.read_bytes()).hexdigest()
    assembly, obj, native = (output / (name + suffix) for suffix in ('.dll', '.o', ''))
    run(['dotnet', compiler, 'neoclr', '--core-reference', core, '--runtime-seed', seed,
         *[x for lib in libs for x in ('--reference', lib)], '--bootstrap-intrinsics', '--bootstrap-ownership',
         ownership, '--object-library', 'System.Runtime', '-o', assembly, source])
    expected = 1 if name == 'ListenFault' else 0
    interpreted = run([runtime, 'run', assembly, *context], expected)
    if name == 'Listen':
        rejected = json.loads(run([aot, '--inspect', assembly, '@entry', '--closed-world', *flags]).stdout)['admission']
        if rejected['accepted']:
            raise RuntimeError('listener unexpectedly admitted without opt-in')
        report['withoutBinding'] = rejected
    inspection = json.loads(run([aot, '--inspect', assembly, '@entry', '--closed-world', *flags, '--bind-socket-listener']).stdout)
    if not inspection['admission']['accepted']:
        raise RuntimeError(inspection['admission'])
    run([aot, '--closed-world', assembly, '@entry', obj, *flags, '--bind-socket-listener'])
    run(['clang', '-arch', 'arm64', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror',
         '-fsanitize=undefined,bounds', '-DNEOCLR_NATIVE_GC', *adapters, obj, '-o', native])
    executed = run([native], expected)
    lines = executed.stderr.splitlines(keepends=True)
    if not lines or lines[0] != f'SOCKET_CLEANUP {expected}\n':
        raise RuntimeError(f'Unexpected cleanup: {executed.stderr}')
    if executed.stdout != interpreted.stdout or ''.join(lines[1:]) != interpreted.stderr:
        raise RuntimeError(f'Output/fault mismatch: {executed} vs {interpreted}')
    report['cases'][name] = {'status': expected, 'output': executed.stdout, 'fault': interpreted.stderr,
                             'hostClosedOutstandingListeners': expected, 'heapCapacity': 65536,
                             'bindings': inspection['selection']['nativeBindings']}
    # Check the distributable binary separately from sanitizer runtime linkage.
    plain = output / (name + '-standalone')
    run(['clang', '-arch', 'arm64', '-std=c11', '-O2', '-DNEOCLR_NATIVE_GC', *adapters, obj, '-o', plain])
    dependencies = run(['otool', '-L', plain]).stdout.splitlines()[1:]
    if [line.split()[0] for line in dependencies] != ['/usr/lib/libSystem.B.dylib']:
        raise RuntimeError(dependencies)
    run([plain], expected)
    for f in (assembly, obj, plain):
        report['inputs'][str(f)] = hashlib.sha256(f.read_bytes()).hexdigest()
    save()
print(json.dumps(report['cases'], indent=2))

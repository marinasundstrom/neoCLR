#!/usr/bin/env python3
"""Compile and execute the same Raven callback consumer in both modes; no timing."""
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
p.add_argument('--case', action='append', choices=('Callbacks', 'CallbackFault', 'CallbackList', 'ResultList', 'TaskResultList', 'EnumValues', 'ReferenceList', 'ValueDisplay', 'TaskQueue', 'QueuePump', 'QueuePumpFault', 'PrimitiveMembers', 'AsyncEntry', 'AsyncEntryFault', 'AsyncEntryPending', 'AsyncEntryCancelled'))
a = p.parse_args()
compiler, runtime, aot, bundle, output = (getattr(a, k).resolve() for k in ('compiler', 'runtime', 'aot', 'bundle', 'output'))
output.mkdir(parents=True, exist_ok=False)
base = ROOT / 'docs/experiments/aot-console'
library = bundle / 'lib' if (bundle / 'lib').is_dir() else bundle
seed, core, ownership = (library / n for n in ('System.runtime.neox', 'Core.dll', 'ownership.json'))
libs = [library / n for n in ('System.Runtime.dll', 'System.Web.dll', 'System.Networking.dll', 'System.Data.dll')]
context = ['--system', seed, *[x for lib in libs for x in ('--module', lib)], '--object-root', libs[0]]
flags = [*context, '--compile-system', '--bind-user-fault', '--reference-arena', '--native-gc', '--bind-int32-to-string', '--bind-utf8-text', '--bind-task-queue', '--native-stack-budget', '--bind-console-write-line']
adapters = [base / 'task-queue-host.c', base / 'root-probe.c', base / 'native-gc.c',
            base / 'task-queue.c', base / 'native-stack.c', base / 'console.c', base.parent / 'aot-scalar/console.c', base / 'text-arena.c', base.parent / 'aot-fault-details/render.c']
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
          *ROOT.joinpath('tools/aot-poc/src').glob('*.rs'), *base.glob('*.h'), *base.parent.joinpath('aot-fault-details').glob('*.h'), *compiler.parent.glob('*.dll')]:
    report['inputs'][str(f)] = hashlib.sha256(f.read_bytes()).hexdigest()
for name in (a.case or ('Callbacks', 'CallbackFault', 'CallbackList', 'ResultList')):
    source = {
        'AsyncEntry': ROOT / 'docs/experiments/raven-target/samples/library-async.rvn',
        'AsyncEntryPending': ROOT / 'docs/experiments/extended-cli-metadata/bootstrap/native-async-entry-pending.rvn',
        'AsyncEntryCancelled': ROOT / 'docs/experiments/extended-cli-metadata/bootstrap/native-async-entry-cancelled.rvn',
    }.get(name, Path(__file__).with_name(name + '.rvn'))
    report['inputs'][str(source)] = hashlib.sha256(source.read_bytes()).hexdigest()
    assembly, obj, native = (output / (name + suffix) for suffix in ('.dll', '.o', ''))
    run(['dotnet', compiler, 'neoclr', '--core-reference', core, '--runtime-seed', seed,
         *[x for lib in libs for x in ('--reference', lib)], '--bootstrap-intrinsics', '--bootstrap-ownership',
         ownership, '--object-library', 'System.Runtime', '--async-library', 'System.Runtime', '-o', assembly, source])
    expected = 1 if name in ('CallbackFault', 'QueuePumpFault', 'AsyncEntryFault', 'AsyncEntryPending', 'AsyncEntryCancelled') else 0
    host_flags = ['-DNEOCLR_HOST_TASK_PUMP'] if name in ('TaskQueue', 'QueuePump', 'QueuePumpFault') else []
    interpreted = run([runtime, 'run', assembly, *context, '--instructions', '100000000'], expected)
    case_flags = flags + (['--bind-integer-text'] if name == 'PrimitiveMembers' else [])
    inspection = json.loads(run([aot, '--inspect', assembly, '@entry', '--closed-world', *case_flags]).stdout)
    if not inspection['admission']['accepted']:
        raise RuntimeError(inspection['admission'])
    run([aot, '--closed-world', assembly, '@entry', obj, *case_flags])
    run(['clang', '-arch', 'arm64', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror',
         '-fsanitize=undefined,bounds', '-DNEOCLR_NATIVE_GC', *host_flags, *adapters, obj, '-o', native])
    executed = run([native], expected)
    if name == 'AsyncEntry':
        assert executed.stdout == 'Suspended\n42\n'
    if name == 'AsyncEntryFault':
        assert 'Async entry callback fault' in executed.stderr
    if name == 'PrimitiveMembers':
        assert executed.stdout == '1\n-1\n0\n-9223372036854775808\n9223372036854775807\n-1\n1\n0\n1\n-1\n0\n1\n42\n'
        report['primitiveInstanceProjections'] = inspection['selection']['primitiveInstanceProjections']
    if executed.stdout != interpreted.stdout or executed.stderr != interpreted.stderr:
        raise RuntimeError(f'Output/fault mismatch: {executed} vs {interpreted}')
    report['cases'][name] = {'status': expected, 'output': executed.stdout, 'fault': interpreted.stderr,
                             'heapCapacity': 65536,
                             'bindings': inspection['selection']['nativeBindings']}
    # Check the distributable binary separately from sanitizer runtime linkage.
    plain = output / (name + '-standalone')
    run(['clang', '-arch', 'arm64', '-std=c11', '-O2', '-DNEOCLR_NATIVE_GC', *host_flags, *adapters, obj, '-o', plain])
    dependencies = run(['otool', '-L', plain]).stdout.splitlines()[1:]
    if [line.split()[0] for line in dependencies] != ['/usr/lib/libSystem.B.dylib']:
        raise RuntimeError(dependencies)
    run([plain], expected)
    for f in (assembly, obj, plain):
        report['inputs'][str(f)] = hashlib.sha256(f.read_bytes()).hexdigest()
    save()
# Recompile the HTTP driver to record the next admission boundary, without claiming execution.
source = ROOT / 'docs/experiments/http-server/Server.rvn'
report['inputs'][str(source)] = hashlib.sha256(source.read_bytes()).hexdigest()
assembly = output / 'Server.dll'
run(['dotnet', compiler, 'neoclr', '--core-reference', core, '--runtime-seed', seed,
     *[x for lib in libs for x in ('--reference', lib)], '--bootstrap-intrinsics', '--bootstrap-ownership',
     ownership, '--object-library', 'System.Runtime', '--async-library', 'System.Runtime', '-o', assembly, source])
inspection = json.loads(run([aot, '--inspect', assembly, '@entry', '--closed-world', *flags,
                            '--bind-socket-listener', '--bind-socket-accept', '--bind-socket-transfer', '--bind-integer-text']).stdout)
report['serverAdmission'] = inspection['admission']
if hashlib.sha256(aot.read_bytes()).hexdigest() != report['inputs'][str(aot)]:
    raise RuntimeError('AOT executable changed during validation; rerun with a stable build')
save()
print(json.dumps({'cases': report['cases'], 'serverAdmission': report['serverAdmission']}, indent=2))

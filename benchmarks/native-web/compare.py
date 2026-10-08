#!/usr/bin/env python3
"""Build one Raven routing artifact, compare interpreter/native process wall time,
and record admission of the checked-in HTTP server. No network throughput claim.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
p = argparse.ArgumentParser(description=__doc__)
for key in ('compiler', 'runtime', 'aot', 'bundle', 'output'):
    p.add_argument('--' + key, type=Path, required=True)
p.add_argument('--rounds', type=int, default=5)
a = p.parse_args()
if a.rounds < 3:
    p.error('use at least three measured pairs')
compiler, runtime, aot, bundle, output = (getattr(a, k).resolve() for k in ('compiler', 'runtime', 'aot', 'bundle', 'output'))
output.mkdir(parents=True, exist_ok=False)
base = ROOT / 'docs/experiments/aot-console'
source = Path(__file__).with_name('Routing.rvn')
seed, core, ownership = (bundle / 'lib' / n for n in ('System.runtime.neox', 'Core.dll', 'ownership.json'))
libs = [bundle / 'lib' / n for n in ('System.Runtime.dll', 'System.Web.dll', 'System.Networking.dll', 'System.Data.dll')]
report = dict(revision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              host=platform.platform(), scope='process wall time: startup, metadata loading/verification (interpreter), routing and output; no network or build time',
              requests=1024, warmupPairs=1, measuredPairs=a.rounds, commands=[], inputs={}, SDKROOT=os.environ.get('SDKROOT'))

def save():
    (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')

def record(path):
    report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()

def run(command):
    r = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True, timeout=300)
    report['commands'].append(dict(command=r.args, exit=r.returncode, stdout=r.stdout, stderr=r.stderr))
    save()
    if r.returncode:
        raise RuntimeError(f'{r.args}: {r.stderr}')
    return r

for f in [Path(__file__), compiler, runtime, aot, source, seed, core, ownership, *libs,
          *compiler.parent.glob('*.dll'), *compiler.parent.glob('*.deps.json'), *compiler.parent.glob('*.runtimeconfig.json')]:
    record(f)
report['clang'] = run(['clang', '--version']).stdout
report['dotnet'] = run(['dotnet', '--version']).stdout
context = ['--system', seed, *[x for lib in libs for x in ('--module', lib)], '--object-root', libs[0]]
flags = [*context, '--compile-system', '--bind-user-fault', '--bind-console-write-line', '--bind-utf8-text',
         '--bind-integer-text', '--bind-int32-to-string', '--reference-arena', '--native-gc']

def compile_source(src, dest):
    record(src)
    run(['dotnet', compiler, 'neoclr', '--core-reference', core, '--runtime-seed', seed,
         *[x for lib in libs for x in ('--reference', lib)], '--bootstrap-intrinsics', '--bootstrap-ownership',
         ownership, '--object-library', 'System.Runtime', '-o', dest, src])
    record(dest)

assembly, obj, native = (output / n for n in ('Routing.dll', 'routing.o', 'routing'))
compile_source(source, assembly)
# Compile the identical Main entry used by the interpreter, not a different host entry.
run([aot, '--closed-world', assembly, '@entry', obj, *flags])
record(obj)
adapters = [base / 'route-lifetime-host.c', base.parent / 'aot-fault-details/render.c', base / 'console.c',
            base.parent / 'aot-scalar/console.c', base / 'text-arena.c', base / 'root-probe.c', base / 'native-gc.c']
for f in [*adapters, *base.glob('*.h'), *base.parent.joinpath('aot-fault-details').glob('*.h')]:
    record(f)
run(['clang', '-arch', 'arm64', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror',
     '-DNEOCLR_ROOT_PROBES', '-DNEOCLR_NATIVE_GC', *adapters, obj, '-o', native])
record(native)
report['nativeImageBytes'] = native.stat().st_size
report['nativeDependencies'] = run(['otool', '-L', native]).stdout.splitlines()[1:]
if [line.split()[0] for line in report['nativeDependencies']] != ['/usr/lib/libSystem.B.dylib']:
    raise RuntimeError('unexpected native dependency')
commands = {'interpreted': [runtime, 'run', assembly, *context, '--instructions', '100000000'],
            'native': [native, '1024', '65536']}
for mode, command in commands.items():
    result = run([*command, '--gc-stats'] if mode == 'interpreted' else [*command, 'audit'])
    if result.stdout != '1024\n':
        raise RuntimeError(f'{mode}: unexpected result {result.stdout!r}')
    report[mode + 'Audit'] = result.stderr
samples = {mode: [] for mode in commands}
for round in range(a.rounds + 1):
    for mode in (list(commands) if round % 2 == 0 else list(reversed(commands))):
        start = time.perf_counter()
        result = subprocess.run(list(map(str, commands[mode])), cwd=ROOT, capture_output=True, text=True, timeout=300)
        elapsed = time.perf_counter() - start
        if result.returncode or result.stdout != '1024\n' or result.stderr:
            raise RuntimeError(f'{mode} failed validation: {result}')
        if round:
            samples[mode].append(elapsed)
    report['samplesSeconds'] = samples
    save()
report['medianSeconds'] = {mode: statistics.median(values) for mode, values in samples.items()}
# Keep the actual server as the admission driver; a rejection is recorded evidence,
# never treated as a successful server run or a benchmark result.
server_source = ROOT / 'docs/experiments/http-server/Server.rvn'
server = output / 'Server.dll'
compile_source(server_source, server)
inspection = run([aot, '--inspect', server, '@entry', '--closed-world', *flags])
(output / 'server-inspection.json').write_text(inspection.stdout)
report['serverAdmission'] = json.loads(inspection.stdout)['admission']
save()
print(json.dumps({k: report[k] for k in ('medianSeconds', 'serverAdmission')}, indent=2))

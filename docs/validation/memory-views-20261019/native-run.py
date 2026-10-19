#!/usr/bin/env python3
"""Run the bounded Raven byte-memory consumer on interpreter, JIT and AOT (macOS ARM64)."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--compiler', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
output = args.output.resolve()
output.mkdir(parents=True, exist_ok=True)
report = {'passed': False, 'commands': []}


def run(label, command, expected=0):
    result = subprocess.run(list(map(str, command)), capture_output=True, text=True, timeout=300)
    log = result.stdout + result.stderr
    (output / (label + '.log')).write_text(log)
    report['commands'].append({'label': label, 'command': list(map(str, command)),
                               'exitCode': result.returncode,
                               'logSha256': hashlib.sha256(log.encode()).hexdigest(),
                               'output': log[-1000:] if 'execute' in label or label == 'execute-aot' else None})
    (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(label, result.returncode, flush=True)
    if result.returncode != expected:
        raise RuntimeError(log[-6000:])
    return result


run('raven', ['dotnet', args.compiler, 'neoclr', '--project', HERE / 'NativeMemoryConsumer.rvnproj'])
image = HERE / 'bin/neoclr/NativeMemoryConsumer.dll'
core = ROOT / 'runtime/raven/projects/System.Runtime/bin/neoclr/System.Runtime.dll'
# Use the same explicit minimal seed as the interpreter foundation consumer.
seed = output / 'System.neoil'
seed.write_text('.module System\n')
refs = ['--system', seed, '--object-root', core, '--module', core]
runtime = ROOT / 'target/debug/neoclr'
codegen = ROOT / 'target/debug/neoclr-codegen'
run('interpreter', [runtime, 'run', image, '--execution', 'interpreter', *refs])
options = [*refs, '--compile-system', '--reference-arena', '--native-gc',
           '--native-stack-budget', '--bind-user-fault']
obj = output / 'memory.o'
run('aot', [codegen, '--closed-world', image, '@entry', obj, *options])
sdk = subprocess.check_output(['xcrun', '--sdk', 'macosx', '--show-sdk-path'], text=True).strip()
clang = subprocess.check_output(['xcrun', '--sdk', 'macosx', '--find', 'clang'], text=True).strip()
console = ROOT / 'docs/experiments/aot-console'
host = HERE / 'native-host.c'
support = [console / f for f in ('native-gc.c', 'root-probe.c', 'text-arena.c', 'native-stack.c')]
support += [ROOT / 'docs/experiments/aot-fault-details/render.c']
flags = [clang, '-isysroot', sdk, '-arch', 'arm64', '-std=c11', '-O2', '-Wall', '-Wextra',
         '-Werror', '-DNEOCLR_NATIVE_GC', '-I', console]
run('link-aot', [*flags, '-fsanitize=undefined,bounds', host, obj, *support, '-o', output / 'memory'])
run('execute-aot', [output / 'memory'])
run('link-jit', [*flags, '-DJIT_HOST', '-dynamiclib', host, *support, '-o', output / 'host.dylib'])
jit = run('jit', [codegen, '--jit-managed', '--closed-world', image, '@entry', output / 'host.dylib', *options])
report['jit'] = json.loads(jit.stderr.split('JIT_RESULT ', 1)[1])
assert report['jit']['guestExitCode'] == 0
run('bounds-raven', ['dotnet', args.compiler, 'neoclr', '--project', HERE / 'NativeMemoryBounds.rvnproj'])
bounds_image = HERE / 'bin/neoclr/NativeMemoryBounds.dll'
bounds = run('bounds-interpreter', [runtime, 'run', bounds_image, '--execution', 'interpreter', *refs], 1)
assert 'IndexOutOfRange' in bounds.stdout + bounds.stderr
run('bounds-aot', [codegen, '--closed-world', bounds_image, '@entry', output / 'bounds.o', *options])
run('bounds-link', [*flags, '-DEXPECT_STATUS=8', '-fsanitize=undefined,bounds', host,
                   output / 'bounds.o', *support, '-o', output / 'bounds'])
run('bounds-execute', [output / 'bounds'])
run('bounds-jit-link', [*flags, '-DEXPECT_STATUS=8', '-DJIT_HOST', '-dynamiclib', host,
                       *support, '-o', output / 'bounds.dylib'])
bounds_jit = run('bounds-jit', [codegen, '--jit-managed', '--closed-world', bounds_image,
                              '@entry', output / 'bounds.dylib', *options])
report['boundsJit'] = json.loads(bounds_jit.stderr.split('JIT_RESULT ', 1)[1])
assert report['boundsJit']['guestExitCode'] == 0
# Compile isolated failures so every invalid start reaches the native boundary.
for kind in ('Memory', 'ReadOnlyMemory'):
    for case, start in (('past-end', '1'), ('min', '(-2147483647 - 1)'), ('max', '2147483647')):
        label = f'short-{kind}-{case}'
        source = output / (label + '.rvn')
        source.write_text(f'import System.*\nfunc Main() -> int {{\n'
                          f'    let memory = default({kind}<byte>)\n'
                          f'    return memory.Slice({start}).Length\n}}\n')
        project = output / (label + '.rvnproj')
        template = (HERE / 'NativeMemoryBounds.rvnproj').read_text()
        template = template.replace('NativeMemoryBounds', label).replace('native-bounds.rvn', source.name)
        template = template.replace('../../../runtime/', str(ROOT / 'runtime') + '/')
        project.write_text(template)
        run(label + '-raven', ['dotnet', args.compiler, 'neoclr', '--project', project])
        case_image = output / 'bin/neoclr' / (label + '.dll')
        failure = run(label + '-interpreter', [runtime, 'run', case_image,
                      '--execution', 'interpreter', *refs], 1)
        assert 'IndexOutOfRange' in failure.stdout + failure.stderr
        case_obj = output / (label + '.o')
        run(label + '-aot', [codegen, '--closed-world', case_image, '@entry', case_obj, *options])
        run(label + '-link', [*flags, '-DEXPECT_STATUS=8', '-fsanitize=undefined,bounds',
                             host, case_obj, *support, '-o', output / label])
        run(label + '-execute', [output / label])
        failure_jit = run(label + '-jit', [codegen, '--jit-managed', '--closed-world',
                         case_image, '@entry', output / 'bounds.dylib', *options])
        assert json.loads(failure_jit.stderr.split('JIT_RESULT ', 1)[1])['guestExitCode'] == 0
report['files'] = {str(p): hashlib.sha256(p.read_bytes()).hexdigest()
                   for p in (image, bounds_image, core, runtime, codegen, args.compiler, host,
                             HERE / 'native-run.py', HERE / 'native-memory.rvn', HERE / 'native-bounds.rvn')}
report['passed'] = True
(output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')

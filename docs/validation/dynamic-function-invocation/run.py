#!/usr/bin/env python3
"""Qualify dynamic function invocation against a native bundle on macOS ARM64."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
SOURCE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bundle', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--case', choices=['success', 'wrong-count', 'wrong-box', 'target-fault'], default='success')

args = parser.parse_args()
bundle, output = (p.resolve() for p in (args.bundle, args.output))
output.mkdir(parents=True, exist_ok=True)
image = output / 'DynamicFunction.neox'
source = (SOURCE / 'Main.neoil').read_text()
if args.case == 'wrong-count':
    source = source.replace('ldc.i4 1\narray.reserve', 'ldc.i4 2\narray.reserve')
if args.case == 'wrong-box':
    source = source.replace('ldc.i4 2\nbox Int32', 'ldc.bool true\nbox Boolean')
if args.case == 'target-fault':
    source = source.replace('ldarg 1\nadd\nret', 'fault "dynamic target entered"')
source_path = output / 'Main.neoil'
source_path.write_text(source)
negative = args.case != 'success'
expected = 2 if negative else 42
host = SOURCE.parent / 'interface-variance/native-host.c'
report = {'passed': False, 'commands': []}
def run(label, command, expected=0):
    command = list(map(str, command))
    result = subprocess.run(command, text=True, capture_output=True, timeout=300)
    # Selection manifests are large; retain their hash and raw output locally.
    log = output / (label + '.log')
    log.write_text(result.stdout + result.stderr)
    report['commands'].append(dict(label=label, command=command, exitCode=result.returncode,
        logSha256=hashlib.sha256(log.read_bytes()).hexdigest(),
        output=(result.stdout + result.stderr)[:3000] if label != 'aot' else '(selection manifest in local aot.log)'))
    (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(label, result.returncode, flush=True)
    if result.returncode != expected:
        raise RuntimeError(result.stdout[-3000:] + result.stderr[-3000:])
    return result

image.unlink(missing_ok=True)
references = ['--system', bundle / 'System.runtime.neox', '--object-root', bundle / 'System.Runtime.dll', '--module', bundle / 'System.Runtime.dll']
run('assemble', [ROOT / 'target/debug/neoclr', 'assemble', source_path, image, '--format', 'neox', *references])

interpreter = [ROOT / 'target/debug/neoclr', 'run', image, '--execution', 'interpreter', *references]
interpreted = run('interpreter', interpreter, 1 if negative else 42)
if negative:
    assert {'wrong-count': 'count mismatch', 'wrong-box': 'InvalidCast', 'target-fault': 'dynamic target entered'}[args.case] in interpreted.stderr
codegen = ROOT / 'target/debug/neoclr-codegen'
obj = output / 'app.o'
obj.unlink(missing_ok=True)
options = [*references, '--compile-system', '--reference-arena', '--bind-user-fault', '--native-gc', '--native-stack-budget']
run('aot', [codegen, '--closed-world', image, '@entry', obj, *options])
sdk = subprocess.check_output(['xcrun', '--sdk', 'macosx', '--show-sdk-path'], text=True).strip()
console = ROOT / 'docs/experiments/aot-console'
link = ['clang', '-isysroot', sdk, '-arch', 'arm64', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-I', console, '-DNEOCLR_NATIVE_GC']
support = [console / f for f in ('native-gc.c', 'root-probe.c', 'text-arena.c', 'native-stack.c')]
support += [ROOT / 'docs/experiments/aot-fault-details/render.c']
run('link-aot', [*link, '-fsanitize=undefined,bounds', host, obj, *support, '-o', output / 'app'])
run('execute-aot', [output / 'app'], expected)
run('link-jit', [*link, '-DJIT_HOST', '-dynamiclib', host, *support, '-o', output / 'host.dylib'])
result = run('jit', [codegen, '--jit-managed', '--closed-world', image, '@entry', output / 'host.dylib', *options])
report['jit'] = json.loads(result.stderr.split('JIT_RESULT ', 1)[1])
assert report['jit']['guestExitCode'] == expected
report['files'] = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in
    (SOURCE / 'Main.neoil', source_path, SOURCE / 'run.py', host, codegen, ROOT / 'target/debug/neoclr', image, bundle / 'bundle.json')}
report['passed'] = True
(output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')

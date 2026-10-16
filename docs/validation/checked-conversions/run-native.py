#!/usr/bin/env python3
"""Validate SDK checked conversions on AOT and selected managed JIT cases (macOS ARM64)."""
from concurrent.futures import ThreadPoolExecutor
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--images', type=Path, required=True, help='Existing SDK --checked-conversions output')
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--bundle', type=Path, required=True, help='Matched runtime library bundle for managed JIT')
args = parser.parse_args()
images, output = args.images.resolve(), args.output.resolve()
output.mkdir(parents=True, exist_ok=False)
bundle = args.bundle.resolve()
cases = json.loads((images / 'cases.json').read_text())
runtime = ROOT / 'target/debug/neoclr'
codegen = ROOT / 'target/debug/neoclr-codegen'
host = ROOT / 'docs/validation/checked-arithmetic/host.c'
report = {'passed': False, 'cases': [], 'commands': []}


def run(label, command, expected=0):
    command = list(map(str, command))
    result = subprocess.run(command, capture_output=True, text=True, timeout=120)
    log = result.stdout + result.stderr
    (output / (label + '.log')).write_text(log)
    report['commands'].append({'label': label, 'command': command, 'exitCode': result.returncode,
                               'logSha256': hashlib.sha256(log.encode()).hexdigest()})
    if expected is not None and result.returncode != expected:
        (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
        raise RuntimeError(label + '\n' + log)
    return result, log


sdk = subprocess.check_output(['xcrun', '--sdk', 'macosx', '--show-sdk-path'], text=True).strip()
clang = ['clang', '-isysroot', sdk, '-arch', 'arm64', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror']
console = ROOT / 'docs/experiments/aot-console'
support = [console / f for f in ('native-gc.c', 'root-probe.c', 'text-arena.c', 'native-stack.c')]
support += [ROOT / 'docs/experiments/aot-fault-details/render.c']
jit_options = ['--system', bundle / 'System.runtime.neox', '--object-root', bundle / 'System.Runtime.dll',
               '--module', bundle / 'System.Runtime.dll', '--compile-system', '--reference-arena', '--native-gc', '--native-stack-budget']
for overflow in (False, True):
    name = 'overflow' if overflow else 'valid'
    flags = ['-DEXPECT_OVERFLOW'] if overflow else []
    run('host-' + name, [*clang, *flags, '-c', host, '-o', output / (name + '.o')])
    run('jit-host-' + name, [*clang, *flags, '-DJIT_HOST', '-DNEOCLR_NATIVE_GC', '-I', console, '-dynamiclib', host, *support, '-o', output / (name + '.dylib')])

def aot(case):
    name, overflow = case['name'], case['overflow']
    image = images / (name + '.dll')
    obj = output / (name + '.o')
    run(name + '-aot', [codegen, '--closed-world', image, '@entry', obj])
    variant = 'overflow' if overflow else 'valid'
    executable = output / name
    run(name + '-link', [*clang, '-fsanitize=undefined,bounds', output / (variant + '.o'), obj, '-o', executable])
    _, log = run(name + '-execute', [executable])
    expected = 'status=2 result=-123' if overflow else 'status=0 result=42'
    assert expected in log, (name, log)
    return {**case, 'passed': True, 'imageSha256': hashlib.sha256(image.read_bytes()).hexdigest()}

with ThreadPoolExecutor(max_workers=8) as workers:
    for result in workers.map(aot, cases):
        report['cases'].append(result)
        if len(report['cases']) % 100 == 0:
            print(len(report['cases']), 'AOT PASS', flush=True)

# Every destination/signedness opcode, alternating success/fault and source
# categories, plus floating NaN, infinities, truncation and UInt64 boundaries.
opcodes = list(dict.fromkeys(c['name'].split('-')[0] for c in cases))
sources = ['Int32', 'Int64', 'IntPtr', 'UIntPtr', 'Single', 'Double']
selected = []
for index, opcode in enumerate(opcodes):
    candidates = [c for c in cases if c['name'].startswith(opcode + '-' + sources[index % 6] + '-')]
    selected.append(next((c for c in candidates if c['overflow'] == bool(index % 2)), candidates[0]))
for name in ['Conv_Ovf_I8-Double-3', 'Conv_Ovf_U8-Double-4', 'Conv_Ovf_I4-Single-5',
             'Conv_Ovf_U1-Double-1', 'Conv_Ovf_I8-Double-6', 'Conv_Ovf_U8-Double-7']:
    selected.append(next(c for c in cases if c['name'] == name))
report['jitCases'] = []
for case in selected:
    name = case['name']
    variant = 'overflow' if case['overflow'] else 'valid'
    result, log = run(name + '-jit', [codegen, '--jit-managed', '--closed-world', images / (name + '.dll'),
                                    '@entry', output / (variant + '.dylib'), *jit_options])
    jit = json.loads(result.stderr.split('JIT_RESULT ', 1)[1])
    expected = 'status=2 result=-123' if case['overflow'] else 'status=0 result=42'
    assert jit['guestExitCode'] == 0 and expected in log, (name, log)
    report['jitCases'].append({**case, 'passed': True})
    print(name, 'JIT PASS', flush=True)

report['passed'] = True
report['files'] = {str(p): hashlib.sha256(p.read_bytes()).hexdigest()
                   for p in (host, Path(__file__).resolve(), images / 'cases.json', runtime, codegen, bundle / 'bundle.json',
                             ROOT / 'tools/metadata/NeoCLR.Metadata.Experimental/bin/Debug/net10.0/NeoCLR.Metadata.Experimental.dll')}
(output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')

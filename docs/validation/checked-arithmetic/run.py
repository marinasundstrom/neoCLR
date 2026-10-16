#!/usr/bin/env python3
"""Validate SDK-authored checked integer arithmetic on interpreter, JIT and AOT (macOS ARM64)."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--images', type=Path, required=True, help='Fresh SDK --checked-arithmetic output')
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--bundle', type=Path, required=True, help='Matched runtime library bundle for managed JIT')
args = parser.parse_args()
images, output = args.images.resolve(), args.output.resolve()
output.mkdir(parents=True, exist_ok=False)
bundle = args.bundle.resolve()
cases = json.loads((images / 'cases.json').read_text())
runtime = ROOT / 'target/debug/neoclr'
codegen = ROOT / 'target/debug/neoclr-codegen'
host = Path(__file__).resolve().with_name('host.c')
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
jit_families = ('Int32-Add_Ovf-', 'UInt32-Add_Ovf_Un-', 'Int64-Sub_Ovf-',
                'UInt64-Sub_Ovf_Un-', 'IntPtr-Mul_Ovf-', 'UIntPtr-Mul_Ovf_Un-')
for overflow in (False, True):
    name = 'overflow' if overflow else 'valid'
    flags = ['-DEXPECT_OVERFLOW'] if overflow else []
    run('host-' + name, [*clang, *flags, '-c', host, '-o', output / (name + '.o')])
    run('jit-host-' + name, [*clang, *flags, '-DJIT_HOST', '-DNEOCLR_NATIVE_GC', '-I', console, '-dynamiclib', host, *support, '-o', output / (name + '.dylib')])

for case in cases:
    name, overflow = case['name'], case['overflow']
    image = images / (name + '.dll')
    result, log = run(name + '-interpreter', [runtime, 'run', image, '--execution', 'interpreter'], None)
    if overflow:
        assert result.returncode != 0 and 'ArithmeticOverflow:' in log, (name, log)
    else:
        assert result.returncode == 42, (name, log)
    obj = output / (name + '.o')
    run(name + '-aot', [codegen, '--closed-world', image, '@entry', obj])
    variant = 'overflow' if overflow else 'valid'
    executable = output / name
    run(name + '-link', [*clang, '-fsanitize=undefined,bounds', output / (variant + '.o'), obj, '-o', executable])
    _, aot_log = run(name + '-execute', [executable])
    expected = 'status=2 result=-123' if overflow else 'status=0 result=42'
    assert expected in aot_log, (name, aot_log)
    jit_checked = name.startswith(jit_families)
    if jit_checked:
        result, jit_log = run(name + '-jit', [codegen, '--jit-managed', '--closed-world', image, '@entry', output / (variant + '.dylib'), *jit_options])
        jit = json.loads(result.stderr.split('JIT_RESULT ', 1)[1])
        assert jit['guestExitCode'] == 0 and expected in jit_log, (name, jit_log)
    report['cases'].append({**case, 'passed': True, 'jitChecked': jit_checked,
                            'imageSha256': hashlib.sha256(image.read_bytes()).hexdigest()})
    print(name, 'PASS', flush=True)

report['passed'] = True
report['files'] = {str(p): hashlib.sha256(p.read_bytes()).hexdigest()
                   for p in (host, Path(__file__).resolve(), images / 'cases.json', runtime, codegen, bundle / 'bundle.json',
                             ROOT / 'tools/metadata/NeoCLR.Metadata.Experimental/bin/Debug/net10.0/NeoCLR.Metadata.Experimental.dll')}
(output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')

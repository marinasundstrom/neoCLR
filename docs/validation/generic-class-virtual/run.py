#!/usr/bin/env python3
"""Qualify generic class virtual dispatch against a native bundle on macOS ARM64."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[3]
SOURCE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bundle', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)

args = parser.parse_args()
bundle, output = (p.resolve() for p in (args.bundle, args.output))
output.mkdir(parents=True, exist_ok=True)
image = output / 'GenericClassVirtual.neox'
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

import os
os.environ['NEOCLR_GENERIC_VIRTUAL_ARTIFACT'] = str(image)
run('sdk-clr', ['dotnet', 'run', '--project', ROOT / 'tools/metadata/NeoCLR.Metadata.Experimental.Tests', '-p:WarningLevel=0', '--', '--generic-class-virtuals'])
references = ['--system', bundle / 'System.runtime.neox', '--object-root', bundle / 'System.Runtime.dll', '--module', bundle / 'System.Runtime.dll']
interpreter = [ROOT / 'target/debug/neoclr', 'run', image, '--execution', 'interpreter', *references]
run('interpreter', interpreter, 42)
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
run('execute-aot', [output / 'app'], 42)
run('link-jit', [*link, '-DJIT_HOST', '-dynamiclib', host, *support, '-o', output / 'host.dylib'])
result = run('jit', [codegen, '--jit-managed', '--closed-world', image, '@entry', output / 'host.dylib', *options])
report['jit'] = json.loads(result.stderr.split('JIT_RESULT ', 1)[1])
assert report['jit']['guestExitCode'] == 42
report['files'] = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in
    (ROOT / 'tools/metadata/NeoCLR.Metadata.Experimental.Tests/GenericClassVirtualChecks.cs', SOURCE / 'run.py', host, codegen, ROOT / 'target/debug/neoclr', image, bundle / 'bundle.json')}
report['passed'] = True
(output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')

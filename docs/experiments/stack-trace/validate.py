#!/usr/bin/env python3
"""Qualify owned guest StackTrace snapshots with a rebuilt native class library."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--recipe', type=Path, required=True)
p.add_argument('--benchmark', type=Path, required=True)
p.add_argument('--libraries', type=Path, required=True)
p.add_argument('--host', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
out = a.output.resolve(); out.mkdir(parents=True, exist_ok=False)
r = json.loads(a.recipe.read_text()); b = json.loads(a.benchmark.read_text())
if not r['passed'] or not b['passed']: raise ValueError('passing input recipes required')
base = r['commands'][0]['command']; old = str(Path(base[base.index('--core-reference') + 1]).parent)
base = [v.replace(old, str(a.libraries.resolve())) for v in base[:base.index('-o')]]
native = next(x['command'].copy() for x in b['commands'] if '--closed-world' in x['command'] and '--jit-managed' not in x['command'])
native = [v.replace(old, str(a.libraries.resolve())) for v in native]
native += ['--bind-stack-trace']
link = next(x['command'].copy() for x in b['commands'] if '-dynamiclib' in x['command'])
report = dict(passed=False, commands=[], inputsSha256={}, cases=[])
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
for value in [*base, *native, *link, str(a.host), __file__]:
    path = Path(value)
    if path.is_file(): report['inputsSha256'][str(path.resolve())] = sha(path)
for path in [*HERE.glob('*'), *(ROOT / 'docs/experiments/aot-console').glob('*.h'), ROOT / 'runtime/raven/src/System/Diagnostics/StackTrace.rvn', ROOT / 'runtime/raven/native/RuntimeDiagnosticsCalls.rvn']:
    if path.is_file(): report['inputsSha256'][str(path.resolve())] = sha(path)

def run(label, command):
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=180)
    # Preserve large admission output separately while retaining the hash.
    log = out / (label + '.stdout'); log.write_text(result.stdout)
    report['commands'].append(dict(label=label, command=command, exitCode=result.returncode,
                                   stdoutFile=str(log), stdoutSha256=sha(log), stderr=result.stderr))
    if result.returncode: raise RuntimeError(label + ': ' + result.stderr + result.stdout[-2000:])
    return result

def check(text, expected, repeats):
    blocks = text.rstrip('\n').split('\n\n')
    if len(blocks) != repeats or any(block != blocks[0] for block in blocks):
        raise RuntimeError('snapshot changed after return/collection or invocation output count differs: ' + text)
    if expected not in blocks[0] or 'System.Diagnostics.StackTrace..ctor' in blocks[0] or 'NeoMetadata_' in blocks[0]:
        raise RuntimeError('missing caller or leaked constructor/transport name: ' + text)
    return blocks[0]

try:
    cc = [link[0], '-isysroot', link[link.index('-isysroot') + 1], '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror',
          '-DNEOCLR_NATIVE_GC', '-fsanitize=undefined,bounds', '-I', str(ROOT / 'docs/experiments/aot-console')]
    test = out / 'native-test'
    run('native-test-build', cc + [str(HERE / 'native-test.c'), str(ROOT / 'docs/experiments/aot-console/root-probe.c'),
                                 str(ROOT / 'docs/experiments/aot-console/native-gc.c'), '-o', str(test)])
    run('native-test', [str(test)])
    host = out / 'host.dylib'; link[link.index('-o') + 1] = str(host); run('host', link)
    for asynchronous in [False, True]:
        case = 'AsyncStackTraceCases' if asynchronous else 'StackTraceCases'
        assembly = out / (case + '.dll'); command = base.copy()
        if not asynchronous:
            flag = command.index('--runtime-async-poc'); del command[flag:flag + 2]
        command += ['-o', str(assembly), str(HERE / (case + '.rvn'))]
        if asynchronous: command += [str(ROOT / 'docs/experiments/runtime-async/RuntimeAsyncBridge.rvn')]
        run(case + '-compile', command)
        interp = [str(a.host.resolve()), 'run', str(assembly), '--system', str(a.libraries.resolve() / 'System.runtime.neox'),
                  '--object-root', str(a.libraries.resolve() / 'System.Runtime.dll'), '--instructions', '100000000']
        for name in ('System.Runtime', 'System.Data', 'System.Networking', 'System.Web'):
            interp += ['--module', str(a.libraries.resolve() / (name + '.dll'))]
        result = run(case + '-interpreter', interp)
        trace = check(result.stdout, ('$closure$0.Invoke' if asynchronous else case + '.Capture'), 2)
        obj = assembly.with_suffix('.o'); compile_command = native.copy(); compile_command[2] = str(assembly); compile_command[4] = str(obj)
        run(case + '-native-compile', compile_command)
        aot = [x for x in link if x != '-dynamiclib'] + ['-DNEOCLR_BENCH_AOT', str(obj)]
        exe = assembly.with_suffix(''); aot[aot.index('-o') + 1] = str(exe); run(case + '-link', aot)
        jit = [compile_command[0], '--jit-managed', *compile_command[1:]]; jit[5] = str(host)
        for engine, command in [('jit', jit), ('aot', [str(exe)])]:
            result = run(case + '-' + engine, command)
            native_trace = check(result.stdout, ('$closure$0.Invoke' if asynchronous else case + '.Capture'), 8)
            samples = [json.loads(x.removeprefix('NATIVE_SAMPLE ')) for x in result.stderr.splitlines() if x.startswith('NATIVE_SAMPLE ')]
            if len(samples) != 4 or any(x['allocatedBlocks'] != x['reclaimedBlocks'] or x['reservedStackBytes'] for x in samples):
                raise RuntimeError('native resource check failed')
            report['cases'].append(dict(case=case, engine=engine, trace=native_trace, samples=samples))
        report['cases'].append(dict(case=case, engine='interpreter', trace=trace))
    for path, digest in report['inputsSha256'].items():
        if sha(path) != digest: raise RuntimeError('input changed: ' + path)
    report['passed'] = True
finally:
    (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
print('PASS owned StackTrace snapshots: interpreter, JIT, AOT, pending await and native reclamation')

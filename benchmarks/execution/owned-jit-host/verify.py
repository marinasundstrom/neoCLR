#!/usr/bin/env python3
"""Check opt-in host cleanup and preserve disabled-host machine code."""
import hashlib
import json
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[3]
out = root/'target/owned-jit-qualification'
out.mkdir(exist_ok=False)
prior = json.loads((root/'target/native-host-ownership/results.json').read_text())
base = next(x['command'].copy() for x in prior['commands'] if str(root/'target/native-host-ownership/owned-aot') in x['command'])
base = [x for x in base if not x.endswith(('/owned-host.c', '/main.c', '/candidate.o'))
        and not x.startswith('-DNEOCLR_GC_OWNED_HEAP')]
r = dict(passed=False, commands=[], inputs={})


def run(command):
    x = subprocess.run(command, capture_output=True, text=True, timeout=120)
    r['commands'].append(dict(command=command, exitCode=x.returncode, stdout=x.stdout, stderr=x.stderr))
    (out/'results.json').write_text(json.dumps(r, indent=2)+'\n')
    x.check_returncode()


try:
    for owned in [False, True]:
        for paced in [False, True]:
            for sanitized in [False, True]:
                command = base.copy()
                if not paced:
                    command.remove('-DNEOCLR_GC_PACED')
                if owned:
                    command.append('-DNEOCLR_GC_OWNED_HEAP')
                if sanitized:
                    command += ['-fsanitize=undefined,bounds', '-fno-sanitize-recover=all']
                exe = out/f'failures-{owned}-{paced}-{sanitized}'
                command[command.index('-o')+1] = str(exe)
                run(command + [str(root/'benchmarks/execution/owned-jit-host/failures.c')])
                run([str(exe)])
                if owned:
                    exe = out/f'lifecycle-{paced}-{sanitized}'
                    command[command.index('-o')+1] = str(exe)
                    run(command + [str(root/'docs/experiments/jit-managed/host.c'), str(root/'benchmarks/execution/native-host-ownership/lifecycle.c')])
                    run([str(exe)])
    old = out/'baseline-host.c'
    old.write_bytes(subprocess.check_output(['git','show','71d448f2e:docs/experiments/jit-managed/host.c']))
    first_source = next(i for i, arg in enumerate(base) if arg.endswith('.c'))
    flags = base[:first_source]
    hashes = {}
    for label, source in [('baseline', old), ('current', root/'docs/experiments/jit-managed/host.c')]:
        obj = out/(label+'.o')
        run(flags + ['-c', str(source), '-o', str(obj)])
        hashes[label] = hashlib.sha256(obj.read_bytes()).hexdigest()
    r['disabledHostObjects'] = hashes
    assert hashes['baseline'] == hashes['current']
    for source in [Path(__file__), root/'docs/experiments/jit-managed/host.c', root/'benchmarks/execution/owned-jit-host/failures.c', root/'benchmarks/execution/native-host-ownership/lifecycle.c']:
        r['inputs'][str(source)] = hashlib.sha256(source.read_bytes()).hexdigest()
    r['passed'] = True
finally:
    (out/'results.json').write_text(json.dumps(r, indent=2)+'\n')

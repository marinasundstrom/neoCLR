#!/usr/bin/env python3
"""Validate indexed owned heaps and replay the established shared-worker consumer."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
root = Path(__file__).resolve().parents[4]
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
report = dict(passed=False, commands=[], inputs={})


def run(command):
    r = subprocess.run(command, capture_output=True, text=True, timeout=90)
    report['commands'].append(dict(command=command, exitCode=r.returncode, stdout=r.stdout if len(r.stdout) < 4000 else None, stdoutSha256=hashlib.sha256(r.stdout.encode()).hexdigest(), stderr=r.stderr))
    r.check_returncode()
    assert 'runtime error:' not in r.stderr
    return r


try:
    base = root/'docs/experiments/aot-console'
    for sanitize in [False, True]:
        binary = out/('owned-' + str(sanitize))
        cmd = ['clang', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-pthread',
               '-DNEOCLR_NATIVE_GC', '-DNEOCLR_TEST_THREADS', '-I', str(base),
               str(base/'native-gc-free-index-test.c'), str(base/'native-gc.c'),
               str(base/'root-probe.c'), '-o', str(binary)]
        if sanitize: cmd += ['-fsanitize=undefined,bounds', '-fno-sanitize-recover=all']
        run(cmd)
        run([str(binary)])
    reference = root/'docs/experiments/native-worker-pool/poll-compiled-validation.json'
    prior = json.loads(reference.read_text())
    assert prior['passed']
    codegen = next(entry['command'][:] for entry in prior['commands'] if '--closed-world' in entry['command'])
    for item in codegen:
        path = Path(item)
        if path.is_absolute() and path.suffix in ['.dll', '.neox']:
            assert hashlib.sha256(path.read_bytes()).hexdigest() == prior['inputs'][str(path)]
    old_object = codegen[4]
    codegen[4] = str(out/'worker.o')
    run(codegen)
    for entry in prior['commands']:
        cmd = entry['command']
        if not any(s.endswith('/parallel-host.c') for s in cmd): continue
        for item in cmd:
            if item.endswith('.c'):
                path = Path(item)
                if root/'runtime/native' not in path.parents:
                    assert hashlib.sha256(path.read_bytes()).hexdigest() == prior['inputs'][str(path)], item
        cmd = [str(out/'worker.o') if s == old_object else s for s in cmd]
        binary = out/Path(cmd[cmd.index('-o')+1]).name
        cmd[cmd.index('-o')+1] = str(binary)
        run(cmd)
        run([str(binary)])
    paths = [Path(__file__).resolve(), reference, *(root/'runtime/native').rglob('*.[ch]')]
    for entry in report['commands']:
        paths += [Path(s) for s in entry['command'] if Path(s).is_absolute() and Path(s).is_file()]
    report['inputs'] = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in paths}
    report['passed'] = True
finally:
    (out/'validation.json').write_text(json.dumps(report, indent=2)+'\n')
print('Owned-index threaded differential and compiled shared-worker optimized/sanitized checks passed')

#!/usr/bin/env python3
"""Check diagnostic counts, bounds, TLS isolation and disabled-hook object identity."""
import argparse
import json
import os
from pathlib import Path
import resource
import signal
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=False)
root = Path(__file__).resolve().parents[4]
resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
checks = []
for mode, flags in [('optimized', ['-O2']), ('sanitized', ['-O1', '-fsanitize=undefined,bounds', '-fno-sanitize-recover=all'])]:
    binary = (a.output / mode).resolve()
    cmd = ['clang', '-std=c11', '-Wall', '-Wextra', '-Werror', '-pthread', *flags,
           '-DNEOCLR_GC_SITE_PROFILE', '-I', str(root/'runtime/native/gc'),
           str(Path(__file__).with_name('check.c')), '-o', str(binary)]
    subprocess.run(cmd, check=True)
    for case in ['valid', 'function', 'instruction', 'capacity', 'overflow', 'mismatch']:
        env = dict(os.environ)
        env.pop('NEOCLR_HTTP_CAPACITY_PROFILE', None)
        result = subprocess.run([binary, case], capture_output=True, env=env)
        assert result.returncode == (0 if case == 'valid' else -signal.SIGABRT), (case, result)
        assert not result.stderr, result.stderr
        checks.append(dict(mode=mode, case=case, exitCode=result.returncode))
# No macro: production GC and diagnostic-probe objects must remain byte-identical.
source = Path('runtime/native/gc/root-probe.c')
before = subprocess.run(['git', 'show', 'HEAD:'+str(source)], check=True, capture_output=True).stdout
old = a.output/'before.c'; old.write_bytes(before)
for mode, flags in [('gc', ['-DNEOCLR_NATIVE_GC']), ('probe', [])]:
    objects = []
    for name, path in [('before', old), ('after', root/source)]:
        obj = a.output/f'{mode}-{name}.o'
        subprocess.run(['clang', '-std=c11', '-O2', '-Werror', *flags, '-I', str(root/source.parent), '-c', str(path), '-o', str(obj)], check=True)
        objects.append(obj.read_bytes())
    assert objects[0] == objects[1], mode
    checks.append(dict(mode=mode, disabledObjectIdentical=True))
(a.output/'validation.json').write_text(json.dumps(dict(passed=True, checks=checks), indent=2)+'\n')
print('12 diagnostic cases passed; both disabled-hook objects byte-identical')

#!/usr/bin/env python3
"""Check generated search instrumentation against a known free-list workload."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--diagnostic', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
root = Path(__file__).resolve().parents[4]
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
report = dict(passed=False, commands=[], inputs={})
try:
    source = a.diagnostic.resolve().with_name('diagnostic-gc.c')
    diagnostic = json.loads(a.diagnostic.read_text())
    assert diagnostic['passed']
    assert hashlib.sha256(source.read_bytes()).hexdigest() == diagnostic['inputs'][str(source)]
    for sanitize in [False, True]:
        binary = out/str(sanitize)
        cmd = ['clang', '-O2', '-std=c11', '-Wall', '-Wextra', '-Werror', '-DNEOCLR_NATIVE_GC',
               '-I', str(source.parent), '-I', str(root/'runtime/native/gc'),
               '-I', str(root/'docs/experiments/aot-console'),
               str(Path(__file__).with_name('search-counter-test.c')),
               str(root/'docs/experiments/aot-console/root-probe.c'), '-o', str(binary)]
        if sanitize: cmd += ['-fsanitize=undefined,bounds', '-fno-sanitize-recover=all']
        for command in [cmd, [str(binary)]]:
            r = subprocess.run(command, capture_output=True, text=True, timeout=60)
            report['commands'].append(dict(command=command, exitCode=r.returncode, stdout=r.stdout, stderr=r.stderr))
            r.check_returncode()
            if len(command) == 1:
                assert r.stderr.splitlines() == ['ALLOCATION_SEARCH 5 2 1 4 240', 'ALLOCATION_SEARCH 0 0 0 0 0']
    paths = [source, a.diagnostic.resolve(), Path(__file__).resolve(),
             Path(__file__).with_name('search-counter-test.c'), *(root/'runtime/native').rglob('*.[ch]')]
    report['inputs'] = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in paths}
    report['passed'] = True
finally:
    (out/'validation.json').write_text(json.dumps(report, indent=2)+'\n')
print('Indexed search counters pass optimized and UBSan/bounds checks')

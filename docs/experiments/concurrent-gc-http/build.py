#!/usr/bin/env python3
"""Relink a qualified Serve guest with a tracing-only GC observer; never apply marks."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--native-build', type=Path, required=True)
p.add_argument('--walker-build', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
out = a.output.resolve(); out.mkdir(parents=True, exist_ok=False)
build = a.walker_build.resolve()
native = json.loads((a.native_build / 'build.json').read_text())
qualification = json.loads((build / 'results.json').read_text())
assert native['passed'] and qualification['passed']
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
inputs = {**native['inputs'], **qualification['inputs']}
for name, digest in inputs.items(): assert sha(Path(name)) == digest, name
for path in [Path(__file__).resolve(), HERE / 'shadow.c']: inputs[str(path)] = sha(path)
command = native['command'][:]
base = ROOT / 'docs/experiments/aot-console'
host = ROOT / 'benchmarks/native-web/http-host.c'
text = host.read_text()
marker = '#ifdef NEOCLR_RUNTIME_ASYNC\n    if (neoclr_runtime_async_shutdown_v1(context))'
assert text.count(marker) == 1
text = 'extern void neoclr_shadow_finish_v1(void);\n' + text.replace(marker, '    neoclr_shadow_finish_v1();\n' + marker)
(out / 'http-host.c').write_text(text)
for old, new in [(host, out / 'http-host.c'), (base / 'native-gc.c', HERE / 'shadow.c'), (base / 'root-probe.c', build / 'root-probe.c')]:
    assert command.count(str(old)) == 1
    command[command.index(str(old))] = str(new)
command[command.index('-o') + 1] = str(out / 'app-paced')
command[1:1] = ['-pthread', '-fsanitize=undefined,bounds', '-I', str(build), '-I', str(HERE.parent / 'concurrent-gc-walker'), '-I', str(HERE.parent / 'concurrent-gc-worker'), '-I', str(ROOT / 'runtime/native/gc')]
command.append(str(HERE.parent / 'concurrent-gc-worker/worker.c'))
report = dict(passed=False, contract='Diagnostic shadow tracing; synchronous collector authoritative; no live mark import.', qualification=qualification, inputs=inputs, command=command)
try:
    r = subprocess.run(command, capture_output=True, text=True, timeout=60)
    report.update(exitCode=r.returncode, stderr=r.stderr)
    r.check_returncode()
    shutil.copyfile(a.native_build / 'app.dll', out / 'app.dll')
    report['artifacts'] = {n: sha(out / n) for n in ('app-paced', 'app.dll', 'http-host.c')}
    for name, digest in inputs.items(): assert sha(Path(name)) == digest, name
    report['passed'] = True
finally:
    (out / 'build.json').write_text(json.dumps(report, indent=2) + '\n')

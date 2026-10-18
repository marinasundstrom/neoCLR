#!/usr/bin/env python3
"""Relink a qualified Serve guest with an isolated experimental GC adapter."""
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
p.add_argument('--mode', choices=['shadow', 'apply'], default='shadow')
p.add_argument('--heap-bytes', type=int, help='Override heap size for pressure qualification')
p.add_argument('--release', action='store_true', help='Disable sanitizers only after qualification')
a = p.parse_args()
adapter = HERE / (a.mode + '.c')
out = a.output.resolve(); out.mkdir(parents=True, exist_ok=False)
build = a.walker_build.resolve()
native = json.loads((a.native_build / 'build.json').read_text())
qualification = json.loads((build / 'results.json').read_text())
assert native['passed'] and qualification['passed']
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
inputs = {**native['inputs'], **qualification['inputs']}
for name, digest in inputs.items(): assert sha(Path(name)) == digest, name
for path in [Path(__file__).resolve(), adapter, HERE / 'safety.c']: inputs[str(path)] = sha(path)
command = native['command'][:]
if a.heap_bytes is not None:
    assert a.heap_bytes > 0 and a.heap_bytes % 8 == 0
    flags = [v for v in command if v.startswith('-DNEOCLR_HTTP_HEAP_BYTES=')]
    assert len(flags) == 1
    command[command.index(flags[0])] = '-DNEOCLR_HTTP_HEAP_BYTES=' + str(a.heap_bytes)
base = ROOT / 'docs/experiments/aot-console'
host = ROOT / 'benchmarks/native-web/http-host.c'
text = host.read_text()
marker = '#ifdef NEOCLR_RUNTIME_ASYNC\n    if (neoclr_runtime_async_shutdown_v1(context))'
assert text.count(marker) == 1
text = 'extern void neoclr_shadow_finish_v1(void);\n' + text.replace(marker, '    neoclr_shadow_finish_v1();\n' + marker)
(out / 'http-host.c').write_text(text)
for old, new in [(host, out / 'http-host.c'), (base / 'native-gc.c', adapter), (base / 'root-probe.c', build / 'root-probe.c')]:
    assert command.count(str(old)) == 1
    command[command.index(str(old))] = str(new)
command[command.index('-o') + 1] = str(out / 'app-paced')
command[1:1] = ['-pthread', *([] if a.release else ['-fsanitize=undefined,bounds']), '-I', str(build), '-I', str(HERE.parent / 'concurrent-gc-walker'), '-I', str(HERE.parent / 'concurrent-gc-worker'), '-I', str(ROOT / 'runtime/native/gc')]
command.append(str(HERE.parent / 'concurrent-gc-worker/worker.c'))
report = dict(passed=False, mode=a.mode, sanitized=not a.release, contract='Isolated HTTP snapshot experiment; owner-only application in apply mode; synchronous pressure fallback.', qualification=qualification, inputs=inputs, command=command)
try:
    if a.mode == 'apply' and not a.release:
        sdk = command[command.index('-isysroot') + 1]
        test_command = [command[0], '-isysroot', sdk, '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-pthread', '-fsanitize=undefined,bounds', '-I', str(build), '-I', str(HERE.parent / 'concurrent-gc-walker'), '-I', str(HERE.parent / 'concurrent-gc-worker'), '-I', str(ROOT / 'runtime/native/gc'), str(HERE / 'safety.c'), str(build / 'root-probe.c'), str(HERE.parent / 'concurrent-gc-worker/worker.c'), '-o', str(out / 'safety')]
        compiled = subprocess.run(test_command, capture_output=True, text=True, timeout=60)
        report['safety'] = dict(command=test_command, compilerExit=compiled.returncode, compilerStderr=compiled.stderr, passed=False)
        compiled.check_returncode()
        test = subprocess.run([str(out / 'safety')], capture_output=True, text=True, timeout=30)
        report['safety'].update(exitCode=test.returncode, stdout=test.stdout, stderr=test.stderr)
        test.check_returncode()
        assert not test.stderr
        report['safety']['passed'] = True
    r = subprocess.run(command, capture_output=True, text=True, timeout=60)
    report.update(exitCode=r.returncode, stderr=r.stderr)
    r.check_returncode()
    shutil.copyfile(a.native_build / 'app.dll', out / 'app.dll')
    report['artifacts'] = {n: sha(out / n) for n in ('app-paced', 'app.dll', 'http-host.c')}
    for name, digest in inputs.items(): assert sha(Path(name)) == digest, name
    report['passed'] = True
finally:
    (out / 'build.json').write_text(json.dumps(report, indent=2) + '\n')

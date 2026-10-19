#!/usr/bin/env python3
"""Relink identical JSON guest objects with explicit synchronous GC policies."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
build, out = a.build.resolve(), a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
source = json.loads((build/'build.json').read_text())
assert source['passed']
command = next(c['command'][:] for c in source['commands'] if c['phase'] == 'host-compile-link')
assert '-DNEOCLR_NATIVE_GC' in command
assert not any('NEOCLR_GC_PACED' in x or 'NEOCLR_GC_PROFILE' in x for x in command)
inputs = {Path(x) for x in command if Path(x).is_file()}
inputs.discard(build/'app')
inputs.update(p for p in (ROOT/'runtime/native').rglob('*') if p.suffix in ('.c', '.h'))
inputs.update([build/'build.json', HERE/'report.c', Path(__file__)])
report = dict(passed=False, inputs={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(inputs)}, commands=[])
try:
    for policy in ['every-boundary', 'paced']:
        for instrumented in [False, True]:
            name = policy + ('-profile' if instrumented else '')
            cmd = command[:]
            cmd[cmd.index('-o')+1] = str(out/name)
            if policy == 'paced':
                cmd += ['-DNEOCLR_GC_PACED']
            if instrumented:
                cmd += ['-DNEOCLR_GC_PROFILE', '-I', str(ROOT/'runtime/native/gc'), str(HERE/'report.c')]
            result = subprocess.run(cmd, capture_output=True, text=True)
            report['commands'].append(dict(command=cmd, exitCode=result.returncode, stderr=result.stderr))
            result.check_returncode()
    for path, digest in report['inputs'].items():
        assert hashlib.sha256(Path(path).read_bytes()).hexdigest() == digest, path
    report['executables'] = {p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(out.iterdir())}
    report['passed'] = True
finally:
    (out/'build.json').write_text(json.dumps(report, indent=2)+'\n')

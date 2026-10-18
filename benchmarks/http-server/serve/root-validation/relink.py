#!/usr/bin/env python3
"""Relink a qualified guest object against current native adapters, recording inputs."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
root = Path(__file__).resolve().parents[4]
build = a.build.resolve()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
source = json.loads((build/'build.json').read_text())
assert source['passed']
command = next(c['command'] for c in source['commands'] if c['phase'] == 'host-compile-link')[:]
report = dict(passed=False, sourceBuild=str(build/'build.json'), commands=[], inputs={})
inputs = {Path(item).resolve() for item in command if Path(item).is_file()}
inputs.discard(build/'app')
inputs.update(path for path in (root/'runtime/native').rglob('*') if path.suffix in ('.c', '.h'))
inputs.update([build/'build.json', build/'app.dll', Path(__file__).resolve()])
report['inputs'] = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in sorted(inputs)}
shutil.copy2(build/'app.dll', out/'app.dll')
try:
    for executable, flags in [('app', []), ('app-paced', ['-DNEOCLR_GC_PACED'])]:
        cmd = command[:]
        cmd[cmd.index('-o')+1] = str(out/executable)
        cmd += flags
        result = subprocess.run(cmd, capture_output=True, text=True)
        report['commands'].append(dict(command=cmd, exitCode=result.returncode, stderr=result.stderr))
        result.check_returncode()
    for path, digest in report['inputs'].items():
        assert hashlib.sha256(Path(path).read_bytes()).hexdigest() == digest, path
    report['executables'] = {name: hashlib.sha256((out/name).read_bytes()).hexdigest() for name in ['app','app-paced']}
    report['passed'] = True
finally:
    (out/'build.json').write_text(json.dumps(report, indent=2)+'\n')

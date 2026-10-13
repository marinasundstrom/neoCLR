#!/usr/bin/env python3
"""Build the retained direct net/http comparison with the local Go toolchain."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess

root = Path(__file__).resolve().parent
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
report = dict(prepared=False, inputs={}, outputs={}, platform=platform.platform())
env = dict(os.environ, GOTOOLCHAIN='local')
for name in ['main.go', 'go.mod', 'prepare.py']:
    report['inputs'][str(root/name)] = sha(root/name)
try:
    report['version'] = subprocess.check_output(['go', 'version'], cwd=root, env=env, text=True).strip()
    report['environment'] = json.loads(subprocess.check_output(['go', 'env', '-json'], cwd=root, env=env, text=True))
    command = ['go', 'build', '-trimpath', '-o', str(out/'http-comparison'), '.']
    result = subprocess.run(command, cwd=root, env=env, capture_output=True, text=True, timeout=600)
    report['build'] = dict(command=command, cwd=str(root), exitCode=result.returncode,
                           stdout=result.stdout, stderr=result.stderr)
    if result.returncode:
        raise RuntimeError('Go build failed')
    report['outputs'][str(out/'http-comparison')] = sha(out/'http-comparison')
    report['command'] = [str(out/'http-comparison')]
    for path, digest in report['inputs'].items():
        assert sha(Path(path)) == digest, path
    report['prepared'] = True
finally:
    (out/'build.json').write_text(json.dumps(report, indent=2)+'\n')
print(out/'build.json')

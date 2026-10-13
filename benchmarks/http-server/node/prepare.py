#!/usr/bin/env python3
"""Validate the dependency-free Node HTTP fixture and pin its runtime/source hashes."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

root = Path(__file__).resolve().parent
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
node = Path(shutil.which('node')).resolve()
report = dict(prepared=False, inputs={}, outputs={})
try:
    report['versions'] = json.loads(subprocess.check_output([str(node), '-p', 'JSON.stringify(process.versions)'], text=True))
    command = [str(node), '--check', str(root/'server.mjs')]
    subprocess.run(command, check=True, capture_output=True, text=True)
    report['check'] = command
    for path in [node, root/'server.mjs', root/'package.json', Path(__file__).resolve()]:
        report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
    report['command'] = [str(node), str(root/'server.mjs')]
    report['prepared'] = True
finally:
    (out/'build.json').write_text(json.dumps(report, indent=2)+'\n')
print(out/'build.json')

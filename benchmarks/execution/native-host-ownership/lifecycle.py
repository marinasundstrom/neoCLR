#!/usr/bin/env python3
"""Qualify the generated owned-host experiment's admission and cleanup paths."""
import json
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[3]
out = root/'target/native-host-lifecycle'
out.mkdir(exist_ok=False)
source = root/'benchmarks/execution/native-host-ownership/lifecycle.c'
previous = json.loads((root/'target/native-host-ownership/results.json').read_text())
base = next(x['command'] for x in previous['commands'] if str(root/'target/native-host-ownership/owned-aot') in x['command'])
base = [x for x in base if not x.endswith('/main.c') and not x.endswith('/candidate.o')]
r = dict(passed=False, runs=[])
try:
    for paced in [False, True]:
        for sanitized in [False, True]:
            command = base.copy()
            if not paced:
                command.remove('-DNEOCLR_GC_PACED')
            exe = out/f'check-{paced}-{sanitized}'
            command[command.index('-o')+1] = str(exe)
            command.append(str(source))
            if sanitized:
                command += ['-fsanitize=undefined,bounds', '-fno-sanitize-recover=all']
            for action in [command, [str(exe)]]:
                x = subprocess.run(action, capture_output=True, text=True, timeout=90)
                r['runs'].append(dict(command=action, exitCode=x.returncode, stdout=x.stdout, stderr=x.stderr))
                x.check_returncode()
    r['passed'] = True
finally:
    (out/'results.json').write_text(json.dumps(r, indent=2)+'\n')

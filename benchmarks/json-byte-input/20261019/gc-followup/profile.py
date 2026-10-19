#!/usr/bin/env python3
"""Run the four profiled arms separately from uninstrumented timing."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--baseline', type=Path, required=True)
p.add_argument('--candidate', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
report = dict(passed=False, runs=[])
try:
    for policy in ['every-boundary', 'paced']:
        for arm, directory in [('baseline', a.baseline), ('candidate', a.candidate)]:
            executable = (directory/(policy+'-profile')).resolve()
            path = out/(arm+'-'+policy+'.json')
            result = subprocess.run([str(executable)], capture_output=True, text=True,
                env=dict(os.environ, NEOCLR_JSON_GC_REPORT=str(path)), timeout=180)
            row = dict(arm=arm, policy=policy, executable=str(executable),
                sha256=hashlib.sha256(executable.read_bytes()).hexdigest(),
                exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr)
            report['runs'].append(row)
            assert result.returncode == 0 and result.stdout == 'PASS JSON byte input probe\n'
            row['gc'] = json.loads(path.read_text())
            assert row['gc']['failures'] == 0 and row['gc']['collections'] > 0
            print(arm, policy, row['gc'], flush=True)
    report['passed'] = True
finally:
    (out/'report.json').write_text(json.dumps(report, indent=2)+'\n')

#!/usr/bin/env python3
"""Measure copied-primitive CPU work; this does not measure managed HTTP execution."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--validation', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
out = a.output.resolve(); out.mkdir(parents=True, exist_ok=False)
here = Path(__file__).resolve().parent
baseline = json.loads(a.validation.read_text())
# Use the successful optimized build recipe, never sanitizer flags for timing.
build = baseline['commands'][0]
assert build['exitCode'] == 0 and not any('sanitize' in s for s in build['command'])
command = [str(here/'benchmark.c') if s.endswith('/test.c') else s for s in build['command']]
command[command.index('-o')+1] = str(out/'benchmark')
report = dict(passed=False, commands=[], samples=[],
              scope='Eight identical copied-primitive CPU jobs. Managed access is gated; no generated-code or HTTP throughput claim.')
report['inputs'] = {s:hashlib.sha256(Path(s).read_bytes()).hexdigest() for s in command if Path(s).is_file()}
try:
    for cmd in [command,[str(out/'benchmark')]]:
        r = subprocess.run(cmd,capture_output=True,text=True,timeout=60)
        report['commands'].append(dict(command=cmd,exitCode=r.returncode,stdout=r.stdout,stderr=r.stderr))
        r.check_returncode()
    report['samples'] = [json.loads(s) for s in r.stdout.splitlines()]
    assert [s['workers'] for s in report['samples']] == [1,2,4,4,2,1]
    report['passed'] = True
finally:
    (out/'measurement.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report['samples'],indent=2))

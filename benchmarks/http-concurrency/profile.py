#!/usr/bin/env python3
"""Run the concurrent fixture with whole-process diagnostic GC counters."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import validate

ROOT = Path(__file__).resolve().parents[2]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
build = json.loads(a.build.read_text())
command = next(c['command'][:] for c in build['commands'] if c.get('phase') == 'host-compile-link')
command[command.index('-o')+1] = str(out/'app')
command += ['-DNEOCLR_GC_PROFILE', str(ROOT/'benchmarks/http-server/profile-gc.c')]
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
inputs = {str(Path(v).resolve()): sha(Path(v)) for v in command if Path(v).is_file()}
for path in [Path(__file__), Path(validate.__file__), a.build]:
    inputs[str(path.resolve())] = sha(path)
report = dict(passed=False, command=command, inputs=inputs,
              scope='Whole-process diagnostic AOT GC, including admission, cancellation and shutdown; not a throughput or per-request allocation measurement.')
try:
    subprocess.run(command, check=True, capture_output=True, text=True)
    report['validation'] = validate.run_case(['/usr/bin/env', 'NEOCLR_HTTP_PROFILE_OUTPUT='+str(out/'gc.json'), str(out/'app')])
    # The shared exit reporter also emits HTTP hook counters. Those hooks are not
    # installed here; only its first (collector statistics) record is relevant.
    report['gc'] = json.loads((out/'gc.json').read_text().splitlines()[0])
    assert report['gc']['allocations'] > 0 and not report['gc']['failures']
    assert report['gc']['allocations'] == report['gc']['reclaimed'], report['gc']
    for path, digest in inputs.items():
        assert sha(Path(path)) == digest, path
    report['outputs'] = {name: sha(out/name) for name in ['app', 'gc.json']}
    report['passed'] = True
finally:
    (out/'profile.json').write_text(json.dumps(report, indent=2)+'\n')
print(json.dumps(report['gc'], indent=2))

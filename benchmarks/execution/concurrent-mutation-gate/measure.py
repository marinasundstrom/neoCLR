import argparse
import hashlib
import json
from pathlib import Path
import subprocess

p = argparse.ArgumentParser()
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
base = Path(__file__).resolve().parent
root = base.parents[2]
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
r = dict(passed=False, commands=[], scope='Single owner, no marker: 0 noinline direct store; 1 inactive gate; 2 active gate plus drain per store. Diagnostic costs, not application performance.')
try:
    recipe = json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
    cmd = recipe[:recipe.index('-std=c11')] + ['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread',str(base/'gate.c'),str(base/'benchmark.c'),'-o',str(out/'benchmark')]
    for command in [cmd, [str(out/'benchmark')]]:
        x = subprocess.run(command, capture_output=True, text=True, timeout=60)
        r['commands'].append(dict(command=command, exitCode=x.returncode, stdout=x.stdout, stderr=x.stderr))
        x.check_returncode()
    r['trials'] = [json.loads(line) for line in x.stdout.splitlines()]
    assert len(r['trials']) == 6
    r['inputs'] = {str(x):hashlib.sha256(x.read_bytes()).hexdigest() for x in [base/'gate.c',base/'gate.h',base/'benchmark.c',Path(__file__)]}
    r['passed'] = True
finally:
    (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

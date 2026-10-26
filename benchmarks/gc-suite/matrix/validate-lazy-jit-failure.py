"""Inject a cold compilation failure after guest entry; require host cleanup first."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--build',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args()
source=a.build.resolve()/'build.json'
b=json.loads(source.read_text());assert b['passed']
for path,digest in b['inputs'].items():
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest,('stale input',path)
r=dict(passed=False,scope=__doc__,cases=[],inputs={str(source):hashlib.sha256(source.read_bytes()).hexdigest()})
try:
    for collector,failure in [(c,f) for c in ['sync','live'] for f in ['error','panic']]:
        recipe=b['recipes']['lazy-'+collector+'-jit']
        env={k:v for k,v in os.environ.items() if not k.startswith('NEOCLR_')}
        env.update(recipe['env']);env['NEOCLR_TEST_LAZY_PANIC_AFTER' if failure=='panic' else 'NEOCLR_TEST_LAZY_FAIL_AFTER']='20'
        x=subprocess.run(recipe['command'],env=env,capture_output=True,text=True,timeout=90)
        row=dict(collector=collector,failure=failure,command=recipe['command'],environment={k:v for k,v in env.items() if k.startswith('NEOCLR_')},exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr)
        r['cases'].append(row)
        # The retained test driver unwraps the expected backend error (101).
        # A signal/native abort, silent success, or a stuck guest is a failure.
        assert x.returncode==101,row
        expected='lazy compiler panicked before publication' if failure=='panic' else 'injected lazy publication failure'
        assert expected in x.stderr,x.stderr
        assert 'PROFILE ' in x.stderr,'native invocation did not reach cleanup/reporting'
        assert '\nREADY\n' not in x.stdout,'fault must occur before server readiness'
        if collector=='live':
            counts=next(list(map(int,l.split()[1:])) for l in x.stderr.splitlines() if l.startswith('CANDIDATE '))
            assert counts[0]==counts[1]+counts[2] and counts[5]==0,counts
    r['passed']=True
finally:a.output.write_text(json.dumps(r,indent=2)+'\n')

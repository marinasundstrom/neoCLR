"""Check mutual recursion, once-only compilation, unused methods and stack-fault unwind."""
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
source=a.build.resolve()/'build.json';b=json.loads(source.read_text());assert b['passed']
for path,digest in b['inputs'].items():
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest,('stale input',path)
root=Path(__file__).resolve().parents[3]
fixture=root/'benchmarks/execution/lazy-jit/recursive.neoil'
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,scope=__doc__,cases=[],inputs={str(source):hashlib.sha256(source.read_bytes()).hexdigest(),str(fixture):hashlib.sha256(fixture.read_bytes()).hexdigest()})
try:
    for overflow in [False,True]:
        selected=out/('overflow.neoil' if overflow else 'bounded.neoil')
        selected.write_text(fixture.read_text().replace('ldc.i4 32','ldc.i4 100000') if overflow else fixture.read_text())
        for mode in ['eager-sync-jit','lazy-sync-jit','eager-live-jit','lazy-live-jit']:
            recipe=b['recipes'][mode]
            http=json.loads(Path(recipe['env']['NEOCLR_LIVE_SELECTION_COMMAND']).read_text())
            host=next(v for v in http if v.endswith('.dylib'))
            system=http[http.index('--system')+1];library=http[http.index('--object-root')+1]
            command=['unused','--jit-managed','--closed-world',str(selected),'Main',host,'--reference-arena','--native-gc','--native-stack-budget','--compile-system','--system',system,'--module',library,'--object-root',library]
            path=out/(('overflow-' if overflow else 'bounded-')+mode+'.json');path.write_text(json.dumps(command))
            providers=['default','system'] if mode.startswith('lazy-') else ['fallback']
            for provider in providers:
                env={k:v for k,v in os.environ.items() if not k.startswith('NEOCLR_')};env.update(recipe['env'])
                env['NEOCLR_LIVE_SELECTION_COMMAND']=str(path)
                if provider=='default':env.pop('NEOCLR_TEST_JIT_ARENA',None)
                else:env['NEOCLR_TEST_JIT_ARENA']=provider
                x=subprocess.run(recipe['command'],env=env,capture_output=True,text=True,timeout=45)
                row=dict(mode=mode,overflow=overflow,provider=provider,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr,selection=command)
                r['cases'].append(row);assert x.returncode==0,row
                jit=json.loads(next(l[len('JIT_RESULT '):] for l in x.stderr.splitlines() if l.startswith('JIT_RESULT ')))
                assert jit['guestExitCode']==int(overflow),jit
                assert 'PROFILE ' in x.stderr,'native cleanup/report missing'
                if mode.startswith('lazy-'):
                    lazy=jit['lazyJit'];assert lazy['selectedMethods']==4 and lazy['compiledMethods']==lazy['loweredMethods']==3 and lazy['uncompiledMethods']==1,lazy
                if recipe['collector']=='live':
                    counts=next(list(map(int,l.split()[1:])) for l in x.stderr.splitlines() if l.startswith('CANDIDATE '))
                    assert counts[0]==counts[1]+counts[2] and counts[5]==0,counts
    r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

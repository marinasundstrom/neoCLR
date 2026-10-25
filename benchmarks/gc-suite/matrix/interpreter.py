"""Measure full standard samples in the interpreter; record unsupported concurrent GC.

Whole-process CPU/wall includes metadata loading and startup, unlike native host
invocation metrics. Timeouts remain failed measurements, never throughput rows.
"""
import argparse
import hashlib
import json
from pathlib import Path
import resource
import subprocess
import time

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--samples',nargs='+')
p.add_argument('--rounds',type=int,default=3)
p.add_argument('--timeout',type=int,default=300)
a = p.parse_args()
root=Path(__file__).resolve().parents[3]
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
build=json.loads((a.build/'results.json').read_text())
assert build['passed'] and build['iterations']==1 and build['suite']=='allocation-patterns'
spec=json.loads((root/'benchmarks/gc-suite/samples.json').read_text())
cases=[c for c in build['cases'] if not a.samples or c['sample'] in a.samples]
assert cases and (not a.samples or {c['sample'] for c in cases}==set(a.samples))
bundle=root/'target/uri-efficiency-bundle/bundle'
r=dict(passed=False,scope=__doc__,execution='interpreter',collector='mark-sweep',rounds=a.rounds,trials=[],inputs={})
try:
    original=cases[0]['native'][0]['selectionCommand']
    context=original[original.index('--system'):original.index('--compile-system')]
    assembly=Path(original[2])
    rejected=subprocess.run([str(bundle/'bin/neoclr'),'run',str(assembly),'--execution','interpreter','--gc','concurrent',*context],capture_output=True,text=True,timeout=30)
    assert rejected.returncode != 0 and 'available: mark-sweep' in rejected.stderr
    r['unsupportedConcurrent']=dict(exitCode=rejected.returncode,stderr=rejected.stderr,
                                    evidence='src/gc.rs GarbageCollector::available and CLI selector rejection')
    for repetition in range(a.rounds):
        for case in cases:
            source=next(c for c in spec['cases'] if c['name']==case['sample'])
            source_path=root/'benchmarks/gc-suite'/source['source']
            assert build['inputs'][str(source_path)]==hashlib.sha256(source_path.read_bytes()).hexdigest()
            original=case['native'][0]['selectionCommand']
            command=[str(bundle/'bin/neoclr'),'run',original[2],'--execution','interpreter',
                     '--gc','mark-sweep','--gc-stats','--instructions','4000000000',*context]
            print(repetition,case['sample'],flush=True)
            before=resource.getrusage(resource.RUSAGE_CHILDREN)
            start=time.perf_counter()
            try:
                x=subprocess.run(command,capture_output=True,text=True,timeout=a.timeout)
                row=dict(sample=case['sample'],round=repetition,command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr,timedOut=False)
            except subprocess.TimeoutExpired as error:
                row=dict(sample=case['sample'],round=repetition,command=command,exitCode=None,timedOut=True,
                         stdout=(error.stdout or b'').decode(),stderr=(error.stderr or b'').decode())
            after=resource.getrusage(resource.RUSAGE_CHILDREN)
            row.update(wallSeconds=time.perf_counter()-start,
                       processCpuSeconds=after.ru_utime+after.ru_stime-before.ru_utime-before.ru_stime,
                       items=source['itemsPerInvocation'])
            row['passed']=not row['timedOut'] and row['exitCode']==0 and row['stdout']==case['expectedPerIteration']
            r['trials'].append(row)
            (out/'progress.json').write_text(json.dumps(r,indent=2)+'\n')
            assert row['passed'], (case['sample'],'timeout' if row['timedOut'] else row['stderr'])
    for path in [Path(__file__), a.build/'results.json',root/'src/gc.rs',bundle/'bin/neoclr']:
        r['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
    for case in cases:
        path=Path(case['native'][0]['selectionCommand'][2]);r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
    r['passed']=True
finally:
    (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

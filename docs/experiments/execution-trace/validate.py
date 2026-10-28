#!/usr/bin/env python3
"""Qualify private worker traces and measure traced/untraced tiny-job controls."""
import argparse
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import platform
import statistics
import subprocess

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--baseline-revision',required=True,help='Commit containing the worker pool before tracing')
a=p.parse_args(); out=a.output.resolve(); out.mkdir(parents=True,exist_ok=False)
here=Path(__file__).resolve().parent; root=here.parents[2]
spec=importlib.util.spec_from_file_location('trace_export',here/'export.py')
export=importlib.util.module_from_spec(spec); spec.loader.exec_module(export)
sources=[root/'runtime/native/gc/native-gc.c',root/'runtime/native/gc/root-probe.c',
         root/'runtime/native/workers/native-workers.c',here/'probe.c']
inputs=[*sources, *sorted((root/'runtime/native/gc').glob('*.h')),
        *sorted((root/'runtime/native/workers').glob('*.h')),here/'export.py',here/'validate.py']
report=dict(passed=False,platform=platform.platform(),commands=[],measurements=[],
            inputs={str(f.relative_to(root)):hashlib.sha256(f.read_bytes()).hexdigest() for f in inputs})
def run(cmd):
    result=subprocess.run(list(map(str,cmd)),capture_output=True,text=True,timeout=60)
    report['commands'].append(dict(command=list(map(str,cmd)),exitCode=result.returncode,
                                   stdout=result.stdout,stderr=result.stderr))
    result.check_returncode(); return result.stdout
try:
    cc=['clang']
    if platform.system()=='Darwin':
        cc=[subprocess.check_output(['xcrun','--find','clang'],text=True).strip(),'-isysroot',
            subprocess.check_output(['xcrun','--show-sdk-path'],text=True).strip()]
    for variant,flags in [('optimized',[]),('ubsan',['-fsanitize=undefined','-fno-sanitize-recover=all'])]:
        binary=out/variant
        run([*cc,'-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC',
             '-DNEOCLR_WORKER_TRACE','-I',root/'runtime/native/workers',*sources,*flags,'-o',binary])
        summaries=[]
        for workers,capacity in [(1,1024),(4,1024),(1,1),(4,1),(1,0)]:
            text=run([binary,workers,capacity])
            if not capacity: continue
            raw=json.loads(text); trace,summary=export.convert(raw); summaries.append(summary)
            assert summary['completeCapture'] == (capacity==1024)
            if capacity==1024:
                assert summary['jobsObserved']==(51 if workers==1 else 49)
                assert summary['queueWaitSamples']==(50 if workers==1 else 49)
            name=f'{variant}-{workers}-{capacity}'
            (out/(name+'.json')).write_text(text)
            (out/(name+'.trace.json')).write_text(json.dumps(trace)+'\n')
        raw=json.loads((out/f'{variant}-1-1024.json').read_text())
        truncated=copy.deepcopy(raw)
        for lane in truncated['lanes']:
            lane['dropped']+=len(lane['events'])-2; lane['events']=lane['events'][:2]
        assert not export.convert(truncated)[1]['completeCapture']
        bad=copy.deepcopy(raw); bad['lanes'][0]['events'][0][1]=0
        try: export.convert(bad)
        except ValueError: pass
        else: raise AssertionError('invalid sequence accepted')
        report[variant]=summaries
    # Fixed work, alternating order, three pairs per worker count. Setup/export excluded.
    for workers in (1,2,4):
        for pair in range(3):
            for enabled in ([False,True] if pair%2==0 else [True,False]):
                sample=json.loads(run([out/'optimized',workers,65536 if enabled else 0,'benchmark']))
                assert sample['dropped']==0
                sample['pair']=pair; report['measurements'].append(sample)
    report['medians']=[]
    for workers in (1,2,4):
        med={}
        for enabled in (False,True):
            rows=[r for r in report['measurements'] if r['workers']==workers and r['enabled']==enabled]
            med[str(enabled)]=dict(wallSeconds=statistics.median(r['wallSeconds'] for r in rows),
                                  cpuSeconds=statistics.median(r['cpuSeconds'] for r in rows),
                                  bufferBytes=rows[0]['bufferBytes'])
        report['medians'].append(dict(workers=workers,modes=med,
            wallChangePercent=(med['True']['wallSeconds']/med['False']['wallSeconds']-1)*100))
    baseline=out/'baseline-workers.c'
    baseline.write_bytes(subprocess.check_output(['git','show',a.baseline_revision+':runtime/native/workers/native-workers.c'],cwd=root))
    report['baselineRevision']=a.baseline_revision
    report['baselineSourceSha256']=hashlib.sha256(baseline.read_bytes()).hexdigest()
    adapter=out/'baseline-adapter.c'
    adapter.write_text('#include "native-worker-trace.h"\nneoclr_workers *neoclr_workers_create_traced(unsigned w,unsigned c,size_t b,size_t e) { return e ? 0 : neoclr_workers_create(w,c,b); }\nint neoclr_workers_trace_view(neoclr_workers *p,unsigned l,neoclr_worker_trace_view *v) { (void)p;(void)l;(void)v;return 0; }\n')
    run([*cc,'-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC',
         '-I',root/'runtime/native/workers',sources[0],sources[1],baseline,adapter,sources[3],'-o',out/'baseline'])
    run([*cc,'-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC',
         '-I',root/'runtime/native/workers',*sources,adapter,'-o',out/'default'])
    report['disabledControls']=[]
    for workers in (1,2,4):
        for pair in range(3):
            for name in (['baseline','default'] if pair%2==0 else ['default','baseline']):
                sample=json.loads(run([out/name,workers,0,'benchmark',32768]))
                sample.update(binary=name,pair=pair); report['disabledControls'].append(sample)
    report['passed']=True
finally:
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=report['passed'],medians=report.get('medians')),indent=2))

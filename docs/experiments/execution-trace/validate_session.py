#!/usr/bin/env python3
"""Qualify shared producers and correlated async/GC timelines."""
import argparse
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
here=Path(__file__).resolve().parent;root=here.parents[2];native=root/'runtime/native';legacy=root/'docs/experiments/aot-console';live=native/'gc/concurrent'
spec=importlib.util.spec_from_file_location('session_export',here/'session-export.py');export=importlib.util.module_from_spec(spec);spec.loader.exec_module(export)
report=dict(passed=False,commands=[],measurements=[])
def run(cmd):
    cmd=list(map(str,cmd));r=subprocess.run(cmd,capture_output=True,text=True,timeout=60)
    report['commands'].append(dict(command=cmd,exitCode=r.returncode,stdout=r.stdout,stderr=r.stderr));r.check_returncode();return r.stdout
try:
    cc=[subprocess.check_output(['xcrun','--find','clang'],text=True).strip(),'-isysroot',subprocess.check_output(['xcrun','--show-sdk-path'],text=True).strip()]
    flags=['-std=c11','-O2','-Wall','-Wextra','-Werror','-Wno-deprecated-declarations','-pthread','-Wl,-dead_strip',
           '-I',native/'trace','-I',native/'async','-I',native/'gc','-I',legacy]
    run(['python3',live/'generate.py','--output',out/'live-gc.c'])
    policy=json.loads((live/'policy.json').read_text())
    for name,sanitize in [('optimized',[]),('ubsan',['-fsanitize=undefined','-fno-sanitize-recover=all'])]:
        run([*cc,*flags,*sanitize,here/'session-test.c',native/'trace/session.c','-o',out/f'producer-{name}'])
        run([out/f'producer-{name}'])
        for mode in ('sync','concurrent'):
            defs=['-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PROFILE'] if mode=='sync' else ['-D'+v for v in policy['defines']]+['-DTRACE_LIVE','-I',live]
            gc=[native/'gc/native-gc.c'] if mode=='sync' else [out/'live-gc.c',live/'gate.c']
            binary=out/f'{mode}-{name}'
            run([*cc,*flags,*defs,*sanitize,'-DNEOCLR_ASYNC_TRACE','-DNEOCLR_EXECUTION_TRACE',here/'session-probe.c',native/'async/native-runtime.c',native/'trace/session.c',*gc,native/'gc/root-probe.c',legacy/'native-stack.c','-o',binary])
            for capacity in (65536,1):
                text=run([binary,capacity,8]);raw=json.loads(text.splitlines()[-1]);trace,summary=export.convert(raw)
                assert summary['completeCapture']==(capacity==65536),summary
                if capacity==65536:
                    assert summary['asyncSessions']==1 and summary['gcRequests']>0
                    assert (summary['gcEpochs']>0)==(mode=='concurrent')
                    if mode=='concurrent':assert any(e.get('args',{}).get('cancelled') for e in trace['traceEvents'])
                    damaged=copy.deepcopy(raw);del damaged['lanes'][0]['events'][5];damaged['lanes'][0]['dropped']+=1
                    partial,info=export.convert(damaged)
                    assert not info['completeCapture'] and not any(e['ph']=='X' for e in partial['traceEvents'])
                    damaged=copy.deepcopy(raw);damaged['lanes']=[]
                    try:export.convert(damaged)
                    except ValueError:pass
                    else:raise AssertionError('missing lane accepted')
                label=f'{mode}-{name}-{capacity}'
                (out/(label+'.json')).write_text(json.dumps(raw)+'\n');(out/(label+'.trace.json')).write_text(json.dumps(trace)+'\n')
    for mode in ('sync','concurrent'):
        for pair in range(3):
            for capacity in ([0,65536] if pair%2==0 else [65536,0]):
                text=run([out/f'{mode}-optimized',capacity,512]);lines=text.splitlines();base=json.loads(lines[0])
                shared=json.loads(lines[-1]) if capacity else None
                if shared:assert shared['lanes'][0]['dropped']==0
                report['measurements'].append(dict(mode=mode,pair=pair,shared=bool(capacity),wallSeconds=base['wallSeconds'],cpuSeconds=base['cpuSeconds'],rounds=512,events=len(shared['lanes'][0]['events']) if shared else 0))
    paths=[*here.glob('*session*'),here/'async-probe.c',here/'async-export.py',native/'async/native-runtime.c',native/'async/native-async-trace-impl.h',native/'async/native-async-trace.h',*list((native/'trace').glob('*')),out/'live-gc.c',legacy/'native-stack.c',native/'gc/root-probe.c']
    paths += [p for p in (native/'gc').rglob('*') if p.suffix in ('.c','.h','.py','.json')]
    report['inputs']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths if p.is_file()}
    report['passed']=True
finally:(out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print('PASS shared sessions and async/GC correlation')

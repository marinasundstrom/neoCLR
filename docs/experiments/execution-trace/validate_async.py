#!/usr/bin/env python3
"""Qualify async trace transitions under synchronous and owned concurrent GC."""
import argparse
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import platform
import subprocess

p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
here=Path(__file__).resolve().parent;root=here.parents[2];native=root/'runtime/native'
legacy=root/'docs/experiments/aot-console';carrier=root/'docs/experiments/runtime-async'
spec=importlib.util.spec_from_file_location('async_export',here/'async-export.py');export=importlib.util.module_from_spec(spec);spec.loader.exec_module(export)
report=dict(passed=False,platform=platform.platform(),commands=[],measurements=[])
def run(cmd):
    cmd=list(map(str,cmd));r=subprocess.run(cmd,capture_output=True,text=True,timeout=60)
    report['commands'].append(dict(command=cmd,exitCode=r.returncode,stdout=r.stdout,stderr=r.stderr))
    r.check_returncode();return r.stdout
try:
    cc=[subprocess.check_output(['xcrun','--find','clang'],text=True).strip(),'-isysroot',subprocess.check_output(['xcrun','--show-sdk-path'],text=True).strip()]
    flags=['-std=c11','-O2','-Wall','-Wextra','-Werror','-Wno-deprecated-declarations','-pthread','-Wl,-dead_strip',
           '-I',native/'async','-I',native/'gc','-I',legacy]
    live=native/'gc/concurrent';run(['python3',live/'generate.py','--output',out/'live-gc.c'])
    policy=json.loads((live/'policy.json').read_text())
    for mode in ('sync','concurrent'):
        defs=['-DNEOCLR_NATIVE_GC'] if mode=='sync' else ['-D'+v for v in policy['defines']]+['-DTRACE_LIVE','-I',live]
        gc=[native/'gc/native-gc.c'] if mode=='sync' else [out/'live-gc.c',live/'gate.c']
        for variant,extra in [('optimized',[]),('ubsan',['-fsanitize=undefined','-fno-sanitize-recover=all'])]:
            binary=out/f'{mode}-{variant}'
            run([*cc,*flags,*defs,'-DNEOCLR_ASYNC_TRACE',*extra,here/'async-probe.c',native/'async/native-runtime.c',*gc,native/'gc/root-probe.c',legacy/'native-stack.c','-o',binary])
            for capacity in (65536,1,0):
                text=run([binary,capacity,8]);raw=json.loads(text)
                if capacity:
                    trace,summary=export.convert(raw)
                    assert summary['completeCapture']==(capacity==65536),summary
                    if capacity==65536:
                        assert sum(e[4]==5 for e in raw['events'])==9
                        assert sum(e[4]==7 and e[5]==2 for e in raw['events'])==1
                        intervals=sorted((e['ts'],e['ts']+e['dur']) for e in trace['traceEvents'] if e.get('name')=='executing (wall time)' and e['ph']=='X')
                        assert all(a[1]<=b[0]+1e-8 for a,b in zip(intervals,intervals[1:])), 'nested execution counted twice'
                        partial=copy.deepcopy(raw);partial['events']=partial['events'][:-2];partial['dropped']+=2
                        assert not export.convert(partial)[1]['completeCapture']
                        bad=copy.deepcopy(raw);bad['events'][0][1]=0
                        try: export.convert(bad)
                        except ValueError: pass
                        else: raise AssertionError('invalid sequence accepted')
                    name=f'{mode}-{variant}-{capacity}'
                    (out/(name+'.json')).write_text(text);(out/(name+'.trace.json')).write_text(json.dumps(trace)+'\n')
    # Existing dependent-root expiry fixture; the source defines its own main.
    wrapper=out/'owned-wrapper.c'
    wrapper.write_text('#include "native-async-trace.h"\n#define main original_main\n#include "'+str(carrier/'native-owned-body-test.c')+'"\n#undef main\nint main(void) { if(!neoclr_async_trace_begin(65536)) return 2; int status=original_main(1,0); neoclr_async_trace_view v; if(!neoclr_async_trace_finish(&v) || v.dropped) return 3; unsigned abandoned=0; for(size_t i=0;i<v.count;i++) abandoned+=v.events[i].kind==NEOCLR_ASYNC_TRACE_ABANDON; if(abandoned!=161) return 4; if(!neoclr_async_trace_dispose()) return 5; return status; }\n')
    run([*cc,*flags,'-DNEOCLR_NATIVE_GC','-DNEOCLR_ASYNC_TRACE','-fsanitize=undefined','-fno-sanitize-recover=all',wrapper,native/'async/native-runtime.c',native/'gc/native-gc.c',native/'gc/root-probe.c',legacy/'native-stack.c','-o',out/'owned'])
    run([out/'owned'])
    # Three alternating pairs per collector; 1024 rounds include genuine parks.
    for mode in ('sync','concurrent'):
        for pair in range(3):
            for capacity in ([0,65536] if pair%2==0 else [65536,0]):
                raw=json.loads(run([out/f'{mode}-optimized',capacity,1024]));assert not raw['dropped']
                report['measurements'].append(dict(mode=mode,pair=pair,enabled=bool(capacity),
                    wallSeconds=raw['wallSeconds'],cpuSeconds=raw['cpuSeconds'],events=len(raw['events']),rounds=raw['rounds']))
    inputs=[here/'async-probe.c',here/'async-export.py',Path(__file__),native/'async/native-runtime.c',native/'async/native-runtime.h',native/'async/native-async-trace.h',native/'async/native-async-trace-impl.h',legacy/'native-stack.c',carrier/'native-owned-body-test.c',out/'live-gc.c']
    inputs += [f for f in (native/'gc').rglob('*') if f.suffix in ('.c','.h','.py','.json')]
    report['inputs']={str(f):hashlib.sha256(f.read_bytes()).hexdigest() for f in inputs}
    report['passed']=True
finally:
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print('PASS async trace, both collectors, dependent-root expiry and paired overhead screen')

#!/usr/bin/env python3
"""Matched HTTP comparison: identical guest inputs, only native-gc.c differs."""
import argparse
import json
from pathlib import Path
import statistics
import subprocess
import benchmark

ROOT=Path(__file__).resolve().parents[2]
sha=benchmark.sha

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--plain',type=Path,required=True,help='Passing plain HTTP benchmark report')
    p.add_argument('--json',type=Path,required=True,help='Passing JSON HTTP benchmark report')
    p.add_argument('--host',type=Path,required=True,help='Passing JIT bundle preparation report')
    p.add_argument('--baseline',required=True)
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--rounds',type=int,default=5)
    a=p.parse_args()
    if a.rounds<5: p.error('At least five rounds required')
    out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
    old=out/'baseline-gc.c'
    old.write_bytes(subprocess.check_output(['git','show',a.baseline+':docs/experiments/aot-console/native-gc.c'],cwd=ROOT))
    gc=ROOT/'docs/experiments/aot-console/native-gc.c'
    report=dict(passed=False,baselineRevision=subprocess.check_output(['git','rev-parse',a.baseline],text=True).strip(),
      commands=[],inputs={},warmups=[],samples=[],summary={},scope='Same guest assemblies/AOT objects and host sources; only collector differs. 32 requests per process, eight discarded, five alternating rounds. No instrumentation or sampling during timings.')
    for path in [Path(__file__).resolve(),Path(benchmark.__file__),a.plain,a.json,a.host,old,gc]:
        report['inputs'][str(path.resolve())]=sha(path)
    for directory in ['aot-console','aot-fault-details','runtime-async','jit-managed']:
        for path in (ROOT/'docs/experiments'/directory).glob('*.h'): report['inputs'][str(path)]=sha(path)
    def build(cmd):
        for value in cmd:
            if Path(value).is_file(): report['inputs'][str(Path(value).resolve())]=sha(value)
        r=subprocess.run(cmd,capture_output=True,text=True,timeout=180)
        report['commands'].append(dict(command=cmd,exitCode=r.returncode,stderr=r.stderr))
        if r.returncode: raise RuntimeError(r.stderr)
    try:
        host=json.loads(a.host.read_text());assert host['passed']
        hostcmd=next(c['command'][:] for c in host['commands'] if '-dynamiclib' in c['command'])
        invocations={}
        for variant in ['baseline','candidate']:
            cmd=[str(old) if v==str(gc) and variant=='baseline' else v for v in hostcmd]
            lib=out/(variant+'.dylib');cmd[cmd.index('-o')+1]=str(lib);build(cmd)
            for case,path in [('plain',a.plain),('json',a.json)]:
                source=json.loads(path.read_text());assert source['passed']
                cmd=next(c['command'][:] for c in source['build']['commands'] if c.get('phase')=='host-compile-link')
                cmd=[str(old) if v==str(gc) and variant=='baseline' else v for v in cmd]
                app=out/(case+'-'+variant);cmd[cmd.index('-o')+1]=str(app);build(cmd)
                jit=source['commands']['jit']+['--jit-host',str(lib)]
                invocations[(case,'jit',variant)]=jit
                invocations[(case,'aot',variant)]=[str(app)]
                for command in [jit,[str(app)]]:
                    for value in command:
                        if Path(value).is_file(): report['inputs'][str(Path(value).resolve())]=sha(value)
        for key,command in invocations.items():
            case,mode,variant=key
            report['warmups'].append(dict(case=case,mode=mode,variant=variant,**benchmark.server_sample(command,case)))
        for n in range(a.rounds):
            for case in ['plain','json']:
                for mode in ['aot','jit']:
                    for variant in (['baseline','candidate'] if n%2==0 else ['candidate','baseline']):
                        report['samples'].append(dict(round=n,case=case,mode=mode,variant=variant,
                          **benchmark.server_sample(invocations[(case,mode,variant)],case)))
            print('Round',n+1,flush=True)
        for case in ['plain','json']:
            for mode in ['jit','aot']:
                row={}
                for variant in ['baseline','candidate']:
                    values=[s['requestsPerSecond'] for s in report['samples'] if (s['case'],s['mode'],s['variant'])==(case,mode,variant)]
                    row[variant]=dict(median=statistics.median(values),min=min(values),max=max(values))
                row['throughputRatio']=row['candidate']['median']/row['baseline']['median']
                report['summary'][case+'-'+mode]=row
        for path,digest in report['inputs'].items(): assert sha(path)==digest,path
        report['passed']=True
    finally: (out/'results.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report['summary'],indent=2))
if __name__=='__main__':main()

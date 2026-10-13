#!/usr/bin/env python3
"""Matched HTTP comparison: identical guest inputs, only the selected native GC source differs."""
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
    p.add_argument('--host',type=Path,help='Passing JIT bundle preparation report')
    p.add_argument('--aot-only',action='store_true',help='Use passing native build reports without requiring a JIT fixture')
    p.add_argument('--baseline',required=True)
    p.add_argument('--source', choices=['native-gc.c', 'root-probe.c'], default='native-gc.c')
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--rounds',type=int,default=5)
    a=p.parse_args()
    if not a.aot_only and not a.host: p.error('--host is required unless --aot-only')
    modes = ['aot'] if a.aot_only else ['aot', 'jit']
    if a.rounds<5: p.error('At least five rounds required')
    out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
    old=out/('baseline-'+a.source)
    baseline_path = 'docs/experiments/aot-console/' + a.source
    if a.source == 'native-gc.c':
        canonical_path = 'runtime/native/gc/native-gc.c'
        if subprocess.run(['git', 'cat-file', '-e', a.baseline + ':' + canonical_path], cwd=ROOT,
                          stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode == 0:
            baseline_path = canonical_path
    old.write_bytes(subprocess.check_output(['git','show',a.baseline+':' + baseline_path],cwd=ROOT))
    gc=ROOT/'docs/experiments/aot-console'/a.source
    report=dict(passed=False,baselineRevision=subprocess.check_output(['git','rev-parse',a.baseline],text=True).strip(),
      changedSource=a.source,modes=modes,commands=[],inputs={},warmups=[],samples=[],summary={},scope=f'Same guest assemblies/AOT objects and host sources; only selected GC source differs. 32 requests per process, eight discarded, {a.rounds} alternating rounds. No instrumentation or sampling during timings.')
    for path in [*sorted((ROOT/'runtime/native/gc').glob('*.[ch]')),Path(__file__).resolve(),Path(benchmark.__file__),a.plain,a.json,old,gc,*([a.host] if a.host else [])]:
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
        if not a.aot_only:
            host=json.loads(a.host.read_text());assert host['passed']
            hostcmd=next(c['command'][:] for c in host['commands'] if '-dynamiclib' in c['command'])
            assert str(gc) in hostcmd
        invocations={}
        for variant in ['baseline','candidate']:
            if not a.aot_only:
                cmd=[str(old) if v==str(gc) and variant=='baseline' else v for v in hostcmd]
                lib=out/(variant+'.dylib');cmd[cmd.index('-o')+1]=str(lib);build(cmd)
            for case,path in [('plain',a.plain),('json',a.json)]:
                source=json.loads(path.read_text());assert source['passed']
                recipe=source.get('build',source)
                cmd=next(c['command'][:] for c in recipe['commands'] if c.get('phase')=='host-compile-link')
                assert str(gc) in cmd
                cmd=[str(old) if v==str(gc) and variant=='baseline' else v for v in cmd]
                app=out/(case+'-'+variant);cmd[cmd.index('-o')+1]=str(app);build(cmd)
                commands=[[str(app)]]
                invocations[(case,'aot',variant)]=commands[0]
                if not a.aot_only:
                    jit=source['commands']['jit']+['--jit-host',str(lib)]
                    invocations[(case,'jit',variant)]=jit
                    commands.append(jit)
                for command in commands:
                    for value in command:
                        if Path(value).is_file(): report['inputs'][str(Path(value).resolve())]=sha(value)
        for key,command in invocations.items():
            case,mode,variant=key
            report['warmups'].append(dict(case=case,mode=mode,variant=variant,**benchmark.server_sample(command,case)))
        for n in range(a.rounds):
            for case in ['plain','json']:
                for mode in modes:
                    for variant in (['baseline','candidate'] if n%2==0 else ['candidate','baseline']):
                        report['samples'].append(dict(round=n,case=case,mode=mode,variant=variant,
                          **benchmark.server_sample(invocations[(case,mode,variant)],case)))
            print('Round',n+1,flush=True)
        for case in ['plain','json']:
            for mode in modes:
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

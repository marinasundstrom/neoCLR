#!/usr/bin/env python3
"""Compare prepared baseline/candidate HTTP libraries with fixed runtime hosts."""
import argparse
import json
from pathlib import Path
import statistics
import benchmark


def main():
    p=argparse.ArgumentParser(description=__doc__)
    for variant in ['baseline','candidate']:
        for case in ['plain','json']:
            p.add_argument('--'+variant+'-'+case,type=Path,required=True)
    p.add_argument('--jit-host',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True)
    a=p.parse_args();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
    report=dict(passed=False,inputs={},commands={},warmups=[],samples=[],summary={},
      scope='Five alternating rounds, 32 requests/process, eight discarded; same host/compiler and application sources, baseline/candidate System.Web libraries. No profiling during timing.')
    def retain(path):
        path=Path(path).resolve(); report['inputs'][str(path)]=benchmark.sha(path)
    retain(__file__);retain(benchmark.__file__);retain(a.jit_host)
    for case in ['plain','json']:
        for variant in ['baseline','candidate']:
            path=getattr(a,variant+'_'+case);retain(path)
            source=json.loads(path.read_text())
            assert source.get('prepared') or source.get('passed')
            bundle=Path(source['commands']['jit'][0]).parent.parent
            for name,digest in source['bundleManifest']['files'].items():
                f=bundle/name;assert benchmark.sha(f)==digest,str(f);retain(f)
            for name,digest in source['build']['inputs'].items():
                assert benchmark.sha(Path(name))==digest,name;retain(name)
            for mode in ['jit','aot']:
                command=source['commands'][mode][:]
                if mode=='jit':command+=['--jit-host',str(a.jit_host.resolve())]
                for arg in command:
                    if Path(arg).is_file():retain(arg)
                report['commands'][case+'-'+mode+'-'+variant]=command
    try:
        for key,command in report['commands'].items():
            case,mode,variant=key.split('-')
            report['warmups'].append(dict(case=case,mode=mode,variant=variant,**benchmark.server_sample(command,case)))
        for n in range(5):
            for case in ['plain','json']:
                for mode in ['jit','aot']:
                    for variant in (['baseline','candidate'] if n%2==0 else ['candidate','baseline']):
                        row=benchmark.server_sample(report['commands'][case+'-'+mode+'-'+variant],case)
                        report['samples'].append(dict(case=case,mode=mode,variant=variant,round=n,**row))
            print('Round',n+1,flush=True)
        for case in ['plain','json']:
            for mode in ['jit','aot']:
                row={}
                for variant in ['baseline','candidate']:
                    values=[s['requestsPerSecond'] for s in report['samples'] if (s['case'],s['mode'],s['variant'])==(case,mode,variant)]
                    row[variant]=dict(median=statistics.median(values),min=min(values),max=max(values))
                row['throughputRatio']=row['candidate']['median']/row['baseline']['median']
                report['summary'][case+'-'+mode]=row
        for path,digest in report['inputs'].items():assert benchmark.sha(Path(path))==digest,path
        report['passed']=True
    finally:(out/'results.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report['summary'],indent=2))
if __name__=='__main__':main()

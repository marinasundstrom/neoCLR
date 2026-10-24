"""Qualify retained ordinary task/collection samples on an explicit live-GC JIT host.

These tiny invocations establish admission and output correctness, not steady-state
performance. The supplied host must be the qualified owned live host, not a sync host.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--runner',type=Path,required=True)
parser.add_argument('--host',type=Path,required=True)
parser.add_argument('--output',type=Path,required=True)
args=parser.parse_args()
root=Path(__file__).resolve().parents[3]
out=args.output.resolve();out.mkdir(parents=True,exist_ok=False)
prior_path=root/'target/gc-mode-comparison-final/results.json'
prior=json.loads(prior_path.read_text())
report={'passed':False,'scope':'Retained standalone output/admission only; tiny invocation timings are not GC performance evidence.','cases':[],'inputs':{}}
try:
    for case in prior['cases']:
        command=next(x['command'].copy() for x in case['runs'] if x.get('mode')=='jit' and not x.get('diagnostic'))
        assert command[1]=='--jit-managed'
        command[5]=str(args.host.resolve())
        path=out/(case['sample']+'-command.json');path.write_text(json.dumps(command,indent=2)+'\n')
        env=dict(os.environ,NEOCLR_LIVE_SELECTION_COMMAND=str(path),NEOCLR_LIVE_SELECTION_MODE='live')
        invocation=[str(args.runner.resolve()),'mutation_store::tests::full_http_selection_compiles_with_live_gc','--exact','--ignored','--nocapture']
        result=subprocess.run(invocation,env=env,cwd=root,capture_output=True,text=True,timeout=240)
        row={'sample':case['sample'],'command':invocation,'selectionCommand':command,'expected':case['expected'],'exitCode':result.returncode,'stdout':result.stdout,'stderr':result.stderr,'passed':False}
        report['cases'].append(row)
        if result.returncode==0:
            lines=[line for line in result.stdout.splitlines(keepends=True) if line.strip() and not line.startswith(('running 1 test','test mutation_store::tests::','test result:'))]
            row['observed']=''.join(lines)
            jit=json.loads(next(line[11:] for line in result.stderr.splitlines() if line.startswith('JIT_RESULT ')))
            row['jitResult']=jit
            counts=list(map(int,next(line for line in result.stderr.splitlines() if line.startswith('CANDIDATE ')).split()[1:]))
            row['candidateCounts']=counts
            row['passed']=row['observed']==case['expected'] and jit['guestExitCode']==0 and len(counts)==6 and counts[0]==counts[1]+counts[2] and counts[5]==0
        for item in command:
            path=Path(item)
            if path.is_file():report['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
    for path in [prior_path,args.runner.resolve(),Path(__file__)]:
        report['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
    report['passed']=bool(report['cases']) and all(c['passed'] for c in report['cases'])
finally:
    (out/'results.json').write_text(json.dumps(report,indent=2)+'\n')
raise SystemExit(0 if report['passed'] else 1)

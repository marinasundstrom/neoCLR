#!/usr/bin/env python3
"""Qualify pacing only for a closed, rooted-admission native workload."""
import argparse
import hashlib
import json
import platform
import statistics
import subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent
BASE=ROOT/'docs/experiments/aot-console'
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',required=True,type=Path);p.add_argument("--integrated",action="store_true");a=p.parse_args()
    out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
    report=dict(passed=False,promotionEligible=False,platform=platform.platform(),commands=[],samples=[],diagnostics=[],summary={},
        scope='Closed native workload only: every allocating path uses rooted admission. General managed/HTTP pacing remains disabled pending helper coverage and scheduler idle integration.')
    inputs=[*sorted((ROOT/'runtime/native').rglob('*.[ch]')),Path(__file__),HERE/'rooted-policy.h',HERE/'rooted-policy-test.c',HERE/'rooted-workload.c',BASE/'native-gc.c',BASE/'root-probe.c',BASE/'text-arena.c',*BASE.glob('*.h')]
    report['inputs']={str(x):hashlib.sha256(x.read_bytes()).hexdigest() for x in inputs}
    def run(cmd):
        cmd=list(map(str,cmd));r=subprocess.run(cmd,capture_output=True,text=True,timeout=120)
        report['commands'].append(dict(command=cmd,exitCode=r.returncode,stderr=r.stderr))
        if r.returncode:raise RuntimeError(r.stdout+r.stderr)
        return r.stdout
    try:
        windows=platform.system()=='Windows'
        if windows: cc=['cl','/nologo','/W4','/WX','/std:c11','/experimental:c11atomics','/O2','/MT','/DNEOCLR_NATIVE_GC','/I'+str(BASE)]
        else: cc=[run(['xcrun','--sdk','macosx','--find','clang']).strip(),'-isysroot',run(['xcrun','--sdk','macosx','--show-sdk-path']).strip(),'-std=c11','-O2','-Wall','-Wextra','-Werror','-DNEOCLR_NATIVE_GC','-I',BASE]
        if a.integrated:
            cc += [('/D' if windows else '-D')+'NEOCLR_INTEGRATED_PACING', ('/D' if windows else '-D')+'NEOCLR_GC_PACED']
            report['scope']='Integrated opt-in native boundary policy; closed workload timings, not managed HTTP/default qualification.'
        report['integrated']=a.integrated
        executables={}
        for name,source in [('contracts','rooted-policy-test.c'),('timed','rooted-workload.c'),('diagnostic','rooted-workload.c')]:
            exe=out/(name+('.exe' if windows else ''));cmd=cc[:]
            if name=='contracts' and not windows:cmd+=['-fsanitize=undefined,bounds']
            if name=='diagnostic':cmd+=[('/D' if windows else '-D')+'NEOCLR_GC_PROFILE']
            cmd += [HERE/source,BASE/'native-gc.c',BASE/'root-probe.c',BASE/'text-arena.c']
            cmd += ['/Fe:'+str(exe)] if windows else ['-o',exe]
            run(cmd);executables[name]=exe
        run([executables['contracts']]);report['contractsPassed']=True
        for workload in ['objects','strings']:
            for round_index in range(-1,5):
                for mode in ['stress','paced'][::1 if round_index%2==0 else -1]:
                    result=json.loads(run([executables['timed'],int(mode=='stress'),workload]));report['samples'].append(dict(workload=workload,mode=mode,round=round_index,**result))
            for mode in ['stress','paced']:
                report['diagnostics'].append(dict(workload=workload,mode=mode,**json.loads(run([executables['diagnostic'],int(mode=='stress'),workload]))))
            medians={m:statistics.median(s['seconds'] for s in report['samples'] if s['workload']==workload and s['mode']==m and s['round']>=0) for m in ['stress','paced']}
            report['summary'][workload]=dict(medians=medians,speedup=medians['stress']/medians['paced']);print(workload,report['summary'][workload],flush=True)
        for path,digest in report['inputs'].items():assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest,path
        report['passed']=True
    finally:(out/'evaluation.json').write_text(json.dumps(report,indent=2)+'\n')
if __name__=='__main__':main()

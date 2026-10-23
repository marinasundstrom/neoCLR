#!/usr/bin/env python3
import argparse,hashlib,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);a=p.parse_args()
a.output.mkdir(parents=True,exist_ok=False)
root=Path(__file__).resolve().parents[4];base=root/'docs/experiments/aot-console';carrier=root/'docs/experiments/runtime-async'
report=dict(passed=False,commands=[],inputs={})
try:
 for reuse in [False,True]:
  for sanitize in [False,True]:
   for fixture in ['native-owned-body-test','native-idle-drain-test','environment']:
    source=Path(__file__).with_name('environment.c') if fixture=='environment' else carrier/(fixture+'.c')
    binary=(a.output/f'{fixture}-{reuse}-{sanitize}').resolve()
    cmd=['clang','-std=c11','-O2','-Wall','-Wextra','-Werror','-Wno-deprecated-declarations','-DNEOCLR_NATIVE_GC','-I',str(base),'-I',str(carrier),str(source),str(carrier/'native-runtime.c'),str(base/'native-gc.c'),str(base/'root-probe.c'),str(base/'native-stack.c'),'-o',str(binary)]
    if reuse:cmd+=['-DNEOCLR_ASYNC_CONTEXT_REUSE']
    if sanitize:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
    for command in [cmd,[str(binary)]]:
     r=subprocess.run(command,capture_output=True,text=True,timeout=60)
     report['commands'].append(dict(command=command,exitCode=r.returncode,stdout=r.stdout,stderr=r.stderr));r.check_returncode()
 for path in [Path(__file__),Path(__file__).with_name('environment.c'),root/'runtime/native/async/native-runtime.c',carrier/'native-owned-body-test.c',carrier/'native-idle-drain-test.c']:
  report['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 report['passed']=True
finally:(a.output/'validation.json').write_text(json.dumps(report,indent=2)+'\n')
print('12 baseline/reuse optimized/sanitized executions passed')

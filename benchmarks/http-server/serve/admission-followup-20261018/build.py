#!/usr/bin/env python3
"""Relink a retained, source-built guest with current native sources; no guest recompilation."""
import argparse, hashlib, json, shutil, subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--build',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
a=p.parse_args();b=a.build.resolve();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
root=Path(__file__).resolve().parents[4];old=json.loads((b/'build.json').read_text());assert old['passed']
for n in ['app.dll','app.o']:shutil.copyfile(b/n,out/n)
native=next(c.copy() for c in old['commands'] if c['phase']=='native-compilation');native['reusedObject']=True
report=dict(passed=False,commands=[native],inputs={},retainedBuild=str(b/'build.json'))
command=next(c['command'][:] for c in old['commands'] if c['phase']=='host-compile-link')
command=[str(out/'app.o') if x.endswith('/app.o') else x for x in command]
try:
 for name,flags in [('app',[]),('app-paced',['-DNEOCLR_GC_PACED'])]:
  command[command.index('-o')+1]=str(out/name)
  r=subprocess.run(command+flags,capture_output=True,text=True)
  report['commands'].append(dict(phase='host-compile-link',command=r.args,exitCode=r.returncode,stdout=r.stdout,stderr=r.stderr));r.check_returncode()
 paths=[Path(__file__),b/'build.json',out/'app.o',out/'app.dll',out/'app',out/'app-paced']
 paths += [Path(x) for x in command if Path(x).is_file()]
 for folder in [root/'runtime/native',root/'docs/experiments/aot-console']:
  paths += [x for x in folder.rglob('*') if x.suffix in ('.c','.h')]
 report['inputs']={str(x):hashlib.sha256(x.read_bytes()).hexdigest() for x in paths};report['passed']=True
finally:(out/'build.json').write_text(json.dumps(report,indent=2)+'\n')

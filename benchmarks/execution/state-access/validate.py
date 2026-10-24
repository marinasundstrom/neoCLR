#!/usr/bin/env python3
"""Pacing lifetime/thread isolation and existing generated consumer qualification."""
import argparse,json,subprocess,hashlib,signal
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);p.add_argument('--pacing-only',action='store_true');a=p.parse_args()
root=Path(__file__).resolve().parents[3];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,commands=[],inputs={})
def run(cmd,expected=0):
 cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=120)
 r['commands'].append(dict(command=cmd,exitCode=x.returncode,expected=expected,stdout=x.stdout,stderr=x.stderr))
 (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
 assert x.returncode==expected,r['commands'][-1]
 return x
try:
 recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command'];cc=recipe[:recipe.index('-std=c11')]
 base=root/'docs/experiments/aot-console'
 for sanitized in [False,True]:
  exe=out/f'pacing-{sanitized}'
  cmd=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PACED','-DNEOCLR_TEST_THREADS',base/'native-gc-pacing-test.c',base/'native-gc.c',base/'root-probe.c',base/'text-arena.c','-o',exe]
  if sanitized:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
  run(cmd);run([exe]);run([exe,"cleanup-publication"],-signal.SIGABRT)
 for group,path in ([] if a.pacing_only else [('workers','target/slot-maps/consumers/results.json'),('consumers','target/slot-maps/async-consumers/results.json')]):
  old=json.loads((root/path).read_text())
  for row in old['commands']:
   cmd=row['command'].copy()
   if '-o' not in cmd or not any(x.endswith('.o') for x in cmd):continue
   for item in cmd:
    f=Path(item)
    if f.is_file():r['inputs'][str(f)]=hashlib.sha256(f.read_bytes()).hexdigest()
   original=cmd[cmd.index('-o')+1];exe=out/(group+'-'+Path(original).name);cmd[cmd.index('-o')+1]=str(exe)
   run(cmd)
   expected=next(x for x in old['commands'] if x['command']==[original])
   x=run([exe],expected['exitCode'] if group=='consumers' else 0)
   if group=='consumers':assert x.stdout==expected['stdout'] and x.stderr==expected['stderr']
 for path in [Path(__file__),base/'native-gc-pacing-test.c',*(root/'runtime/native').rglob('*.[ch]')]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

#!/usr/bin/env python3
"""Relink qualified generated async/shared-worker objects against current native source."""
import argparse,gzip,hashlib,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);p.add_argument('--faults-only',action='store_true');a=p.parse_args()
root=Path(__file__).resolve().parents[3];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,commands=[],inputs={})
try:
 for group in ['workers','consumers']:
  path=root/f'benchmarks/http-server/primitive-comparison/root-map/{group}.json.gz'
  old=json.loads(gzip.decompress(path.read_bytes()));assert old['passed']
  r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
  for row in old['commands']:
   cmd=row['command'].copy()
   if '-o' not in cmd or not any(x.endswith('.o') for x in cmd):continue
   if a.faults_only and 'RuntimeAwaitFault-' not in ' '.join(cmd):continue
   for item in cmd:
    path=Path(item)
    if path.is_file():
     digest=hashlib.sha256(path.read_bytes()).hexdigest();r['inputs'][str(path)]=digest
     if item.endswith('.o'):assert digest==old['inputs'][item]
   original=cmd[cmd.index('-o')+1];exe=out/(group+'-'+Path(original).name);cmd[cmd.index('-o')+1]=str(exe)
   for command in [cmd,[str(exe)]]:
    x=subprocess.run(command,capture_output=True,text=True,timeout=120)
    r['commands'].append(dict(command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr))
    (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
    expected=next((z for z in old['commands'] if z['command']==[original]),None) if command==[str(exe)] else None
    assert x.returncode==(expected['exitCode'] if expected else 0),r['commands'][-1]
    if group=='consumers' and command==[str(exe)]:
     expected=next(z for z in old['commands'] if z['command']==[original])
     assert hashlib.sha256(x.stdout.encode()).hexdigest()==expected['stdoutSha256']
     assert x.stderr==expected['stderr']
 for path in (root/'runtime/native').rglob('*.[ch]'):r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
print('Shared workers and async consumer relinks passed')

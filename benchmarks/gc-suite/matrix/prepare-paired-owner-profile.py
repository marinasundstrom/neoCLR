"""Instrument every AOT recipe in a matched native HTTP build; diagnostic only."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import runpy
import subprocess
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--build',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();root=Path(__file__).resolve().parents[3]
source=a.build.resolve()/'build.json';built=json.loads(source.read_text());assert built['passed']
for path,digest in built['inputs'].items():
 assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest,('stale build',path)
profiler=root/'benchmarks/execution/concurrent-workloads/owner-pauses.py'
instrument=runpy.run_path(str(profiler))['instrument']
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,diagnosticOnly=__doc__,recipes={},commands=[],inputs=dict(built['inputs']))
try:
 for name,recipe in built['recipes'].items():
  if recipe['execution']!='aot':continue
  link=next(c['command'].copy() for c in built['commands'] if '-o' in c['command'] and c['command'][c['command'].index('-o')+1]==recipe['command'][0])
  old=next(v for v in link if v.endswith('/native-gc.c'))
  original=root/'runtime/native/gc/native-gc.c' if recipe['collector']=='sync' else Path(old)
  text=original.read_text()
  text=re.sub(r'#include "(.*?)"',lambda m:'#include "'+str(original.parent/m[1])+'"' if (original.parent/m[1]).exists() else m[0],text)
  copied=out/(name+'-gc.c');copied.write_text(instrument(text,recipe['collector']=='live'))
  host=next(Path(v) for v in link if v.endswith('/owned-host.c'))
  assert 'experiment_owner_pauses_report();' in host.read_text()
  link=[str(copied) if v==old else v for v in link]+['-DOWNER_PAUSE_PROFILE','-I',str(root/'runtime/native/gc')]
  link[link.index('-o')+1]=str(out/name)
  x=subprocess.run(link,capture_output=True,text=True,timeout=180)
  r['commands'].append(dict(command=link,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
  r['recipes'][name]=dict(recipe,command=[str(out/name)])
  for path in [original,copied,out/name]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 for path in [source,profiler,Path(__file__).resolve()]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'build.json').write_text(json.dumps(r,indent=2)+'\n')

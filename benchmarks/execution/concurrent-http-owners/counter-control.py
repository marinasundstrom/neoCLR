"""Relink identical live/guarded hosts without store diagnostic increments."""
import argparse,hashlib,json,re,shutil,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--build',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
base=a.build.resolve();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,sample='plain',commands=[],inputs={},control=str(base),removed={})
try:
 data=json.loads((base/'results.json').read_text());assert data['passed'] and data['sample']=='plain'
 for mode in ['sync','live']:
  for workers in [1,4]:
   old=base/f'{mode}-{workers}'
   if mode=='sync':shutil.copy2(old,out/old.name);continue
   cmd=next(c['command'].copy() for c in data['commands'] if '-o' in c['command'] and Path(c['command'][c['command'].index('-o')+1]).name==old.name)
   source=next(Path(x) for x in cmd if x.endswith('/native-gc.c'))
   text=source.read_text()
   text,count=re.subn(r'live_store_(?:lookup_)?counts\[[0-4]\]\+\+;', '(void)0; /* Diagnostic increment disabled. */',text)
   assert count==13,count
   modified=out/'native-gc.c';modified.write_text(text);r['removed'][str(source)]=count
   cmd=[str(modified) if x==str(source) else x for x in cmd];cmd[cmd.index('-o')+1]=str(out/old.name)
   x=subprocess.run(cmd,capture_output=True,text=True,timeout=60);r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
   for item in [str(source),*cmd]:
    path=Path(item)
    if path.is_file():r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 for path in [Path(__file__),base/'results.json',*out.glob('*-1'),*out.glob('*-4')]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

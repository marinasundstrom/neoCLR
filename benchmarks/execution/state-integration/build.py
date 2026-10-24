import argparse,json,subprocess,hashlib
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);a=p.parse_args();out=a.output.resolve();out.mkdir(parents=True,exist_ok=True)
r={'passed':False,'commands':[],'inputs':{}}
try:
 for name,path in [('plain','target/root-map/build.json'),('json','target/root-map/json-build.json')]:
  x=json.load(open(path));cmd=(x['command'] if 'command' in x else x['commands'][0]['command']).copy();cmd[0]=str(Path('target/release/neoclr-codegen').resolve());cmd[4]=str(out/(name+'.o'));y=subprocess.run(cmd,capture_output=True,text=True,timeout=120);r['commands'].append({'command':cmd,'exitCode':y.returncode,'stdout':y.stdout,'stderr':y.stderr});y.check_returncode()
  for item in cmd:
   p=Path(item)
   if p.is_file():r['inputs'][str(p)]=hashlib.sha256(p.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'objects.json').write_text(json.dumps(r,indent=2)+'\n')

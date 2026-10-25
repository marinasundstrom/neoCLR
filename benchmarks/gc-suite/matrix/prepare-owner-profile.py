"""Relink qualified scalar HTTP variants with complete owner-operation histograms.
Diagnostic wall times include scheduling/clock overhead, not total collector CPU.
"""
import argparse,hashlib,json,re,runpy,subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--control',type=Path,required=True)
p.add_argument('--candidate',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();root=Path(__file__).resolve().parents[3];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
profiler=root/'benchmarks/execution/concurrent-workloads/owner-pauses.py'
instrument=runpy.run_path(str(profiler))['instrument']
r=dict(passed=False,diagnosticOnly=__doc__,recipes={},commands=[],inputs={})
def run(cmd):
 cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=180)
 r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
try:
 for name,directory,mode in [('sync',a.control,'sync'),('live',a.control,'live'),('scalar',a.candidate,'live')]:
  directory=directory.resolve();report=json.loads((directory/'build.json').read_text());assert report['passed']
  for path,digest in report['inputs'].items():
   assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest,('stale build',path)
   r['inputs'][path]=digest
  link=next(c['command'].copy() for c in report['commands'] if '-o' in c['command'] and c['command'][c['command'].index('-o')+1]==str(directory/mode))
  old=next(v for v in link if v.endswith('/native-gc.c'))
  original=root/'runtime/native/gc/native-gc.c' if mode=='sync' else Path(old)
  source=original.read_text()
  source=re.sub(r'#include "(.*?)"',lambda m:'#include "'+str(original.parent/m[1])+'"' if (original.parent/m[1]).exists() else m[0],source)
  copied=out/(name+'-gc.c');copied.write_text(instrument(source,mode=='live'))
  host_source=next(Path(v) for v in link if v.endswith('/owned-host.c'))
  assert 'experiment_owner_pauses_report();' in host_source.read_text()
  link=[str(copied) if v==old else v for v in link]+['-DOWNER_PAUSE_PROFILE','-I',str(root/'runtime/native/gc')]
  link[link.index('-o')+1]=str(out/name);run(link)
  r['recipes'][name+'-aot']=dict(execution='aot',collector=mode,command=[str(out/name)],env={})
  shared=[v for v in link if not v.endswith('/main.c') and v!=str(directory/(mode+'.o'))]+['-dynamiclib']
  host=out/(name+'.dylib');shared[shared.index('-o')+1]=str(host);run(shared)
  old_recipe=report['recipes'][mode+'-jit'];old_selection=Path(old_recipe['env']['NEOCLR_LIVE_SELECTION_COMMAND'])
  selection=json.loads(old_selection.read_text());previous=str(directory/(mode+'.dylib'));assert previous in selection
  selection=[str(host) if v==previous else v for v in selection]
  path=out/(name+'-selection.json');path.write_text(json.dumps(selection))
  r['recipes'][name+'-jit']=dict(old_recipe,env=dict(old_recipe['env'],NEOCLR_LIVE_SELECTION_COMMAND=str(path)))
  for path in [original,copied,host,directory/'build.json',out/name,out/(name+'-selection.json')]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 for path in [Path(__file__),profiler]:r['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'build.json').write_text(json.dumps(r,indent=2)+'\n')

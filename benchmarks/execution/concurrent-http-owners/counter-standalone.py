"""Screen diagnostic counter removal against retained standalone live objects."""
import argparse,hashlib,json,re,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--report',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();out=a.output.resolve();out.mkdir(exist_ok=False)
r=dict(passed=False,commands=[],trials=[],inputs={})
def record(path):r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
try:
 data=json.loads(a.report.read_text());assert data['passed'];record(a.report)
 expected={}
 for sample in ['tasks','collections','arrays']:
  name=sample+'-live-hybrid-inline'
  command=next(c['command'].copy() for c in data['commands'] if '-o' in c['command'] and Path(c['command'][c['command'].index('-o')+1]).name==name)
  expected[sample]=next(t['stdout'] for t in data['trials'] if t['sample']==sample and t['mode']=='live-hybrid-inline')
  source=next(Path(x) for x in command if x.endswith('/native-gc.c'));record(source)
  for mode in ['on','off']:
   cmd=command.copy()
   if mode=='off':
    text,count=re.subn(r'live_store_(?:lookup_)?counts\[[0-4]\]\+\+;', '(void)0; /* Diagnostic increment disabled. */',source.read_text());assert count==13
    modified=out/(sample+'-gc.c');modified.write_text(text);cmd=[str(modified) if x==str(source) else x for x in cmd]
   cmd[cmd.index('-o')+1]=str(out/(sample+'-'+mode))
   x=subprocess.run(cmd,capture_output=True,text=True,timeout=60);r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
   for item in cmd:
    path=Path(item)
    if path.is_file():record(path)
 for pair in range(5):
  for sample in expected:
   for mode in (['on','off'] if pair%2==0 else ['off','on']):
    x=subprocess.run([str(out/(sample+'-'+mode))],capture_output=True,text=True,timeout=30)
    row=dict(sample=sample,mode=mode,pair=pair,exitCode=x.returncode,stderr=x.stderr,stdoutSha256=hashlib.sha256(x.stdout.encode()).hexdigest());r['trials'].append(row)
    assert x.returncode==0 and x.stdout==expected[sample]
    def values(label):return next(l.split()[1:] for l in x.stderr.splitlines() if l.startswith(label+' '))
    counts=list(map(int,values('CANDIDATE')));index=list(map(int,values('INDEX_METADATA')))
    assert counts[0]==counts[1]+counts[2] and counts[1]>0 and counts[5]==0 and index[1]==0
    stores=list(map(int,values('STORES')));lookups=list(map(int,values('STORE_LOOKUPS')))
    assert all(v==0 for v in stores+lookups) if mode=='off' else sum(stores)>0
    profile=values('PROFILE');row.update(cpuSeconds=float(profile[0]),wallSeconds=float(profile[1]),allocations=int(profile[2]),allocatedBytes=int(profile[3]))
 for sample in expected:assert len({(t['allocations'],t['allocatedBytes']) for t in r['trials'] if t['sample']==sample})==1
 record(Path(__file__));r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

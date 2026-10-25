"""Reject inactive, incomplete, errored and count-mismatched close evidence."""
import argparse,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.mkdir(exist_ok=False)
r={'passed':False,'cases':[]}
try:
 for name,trace,count,success in [('valid','1 S 8 100 200 112 0\n2 B 8 100 200 0 0\n3 C 8 100 200 0 0\n',1,True),('inactive','',1,False),('missing-close','1 S 8 100 200 112 0\n2 B 8 100 200 0 0\n',1,False),('close-error','1 S 8 100 200 112 0\n2 B 8 100 200 0 0\n3 C 8 100 200 -1 9\n',1,False),('count-mismatch','1 S 8 100 200 112 0\n2 B 8 100 200 0 0\n3 C 8 100 200 0 0\n',2,False)]:
  directory=a.output/name;directory.mkdir()
  if trace:(directory/'events-1-1.log').write_text(trace)
  (directory/'check.json').write_text(json.dumps({'passed':True,'trials':[{'pid':1,'totalResponses':count,'workers':1,'mode':'sync','repeat':0}]}))
  cmd=['python3',str(Path(__file__).with_name('summarize-close.py')),'--trace',str(directory),'--output',str(directory/'summary.json')]
  x=subprocess.run(cmd,capture_output=True,text=True,timeout=15);r['cases'].append(dict(name=name,command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));assert (x.returncode==0)==success
 r['passed']=True
finally:(a.output/'results.json').write_text(json.dumps(r,indent=2)+'\n')

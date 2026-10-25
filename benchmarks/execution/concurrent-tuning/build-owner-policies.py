"""Relink qualified live HTTP objects with independent per-owner GC policies."""
import argparse,hashlib,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--build',type=Path,required=True,help='Qualified immutable-byref owner build manifest')
p.add_argument('--output',type=Path,required=True)
p.add_argument('--modes',nargs='+',choices=['live','live-moderate','live-watermark'],default=['live','live-moderate','live-watermark'])
p.add_argument('--owners',nargs='+',type=int,choices=[1,4],default=[1,4])
p.add_argument('--sanitize',action='store_true')
a=p.parse_args();assert len(set(a.modes))==len(a.modes) and len(set(a.owners))==len(a.owners)
root=Path(__file__).resolve().parents[3]
manifest=json.loads((a.build/'results.json').read_text());assert manifest['passed'] and manifest.get('stableByrefs') and manifest.get('sample')=='plain'
recipe=next(c['command'] for c in manifest['commands'] if '-o' in c['command'] and Path(c['command'][c['command'].index('-o')+1]).name=='live-1')
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,sample='plain',sanitized=a.sanitize,modes=a.modes,owners=a.owners,scope='Isolated owner heaps; retained generated object; no shared-heap admission',policies={},commands=[],inputs={})
def run(cmd):
 cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=120)
 r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
try:
 source=out/'native-gc.c';provider=out/'text-arena.c'
 run(['python3',root/'benchmarks/execution/concurrent-live/native-scopes.py','--output',source])
 run(['python3',root/'benchmarks/execution/concurrent-store/native-providers.py','--output',provider])
 for mode,allocation,publication,watermark in [('live',256,4096,0),('live-moderate',1024,16384,0),('live-watermark',1024,16384,50)]:
  if mode not in a.modes:continue
  r['policies'][mode]=dict(allocations=allocation,publications=publication,watermark=watermark)
  for owners in a.owners:
   cmd=[]
   for item in recipe:
    if item.startswith(('-DOWNERS=','-DNEOCLR_HTTP_WORKERS=','-DNEOCLR_GC_ALLOCATION_BUDGET=','-DNEOCLR_GC_PUBLICATION_BUDGET=','-DNEOCLR_LIVE_FINISH_WATERMARK=')):continue
    if item.endswith('/native-gc.c'):item=str(source)
    elif item.endswith('/text-arena.c'):item=str(provider)
    cmd.append(item)
   cmd[cmd.index('-o')+1]=str(out/f'{mode}-{owners}')
   cmd += [f'-DOWNERS={owners}',f'-DNEOCLR_HTTP_WORKERS={owners}',f'-DNEOCLR_GC_ALLOCATION_BUDGET={allocation}',f'-DNEOCLR_GC_PUBLICATION_BUDGET={publication}']
   if watermark:cmd += [f'-DNEOCLR_LIVE_FINISH_WATERMARK={watermark}']
   if a.sanitize:cmd += ['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
   run(cmd)
 for path in [Path(__file__),a.build/'results.json',*[Path(x) for c in r['commands'] for x in c['command'] if Path(x).is_file()]]:
  r['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

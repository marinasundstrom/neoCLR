"""Attribute out-of-arena store calls on the exact isolated-owner plaintext image."""
import argparse,hashlib,json,re,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--build',type=Path,required=True);p.add_argument('--client',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
root=Path(__file__).resolve().parents[3];base=a.build.resolve();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,sample='plain',diagnosticOnly=True,commands=[],inputs={})
def run(cmd):
 x=subprocess.run(list(map(str,cmd)),capture_output=True,text=True,timeout=90)
 r['commands'].append(dict(command=x.args,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode();return x
try:
 original=json.loads((base/'results.json').read_text());assert original['passed'] and original['sample']=='plain'
 command=next(c['command'].copy() for c in original['commands'] if '-o' in c['command'] and Path(c['command'][c['command'].index('-o')+1]).name=='live-1')
 source=out/'native-gc.c';source.write_text((base/'native-gc.c').read_text())
 run(['python3',root/'benchmarks/execution/concurrent-workloads/store-sites.py','--source',source])
 host=out/'owned-host.c';text=(base/'owned-host.c').read_text();anchor='static void report_measurement(void) {\n flockfile(stderr);';assert text.count(anchor)==1
 host.write_text('extern void experiment_store_sites_report(void);\n'+text.replace(anchor,anchor+'\n experiment_store_sites_report();'))
 command=[str(source) if x==str(base/'native-gc.c') else str(host) if x==str(base/'owned-host.c') else x for x in command];command[command.index('-o')+1]=str(out/'live-1')
 run(command)
 for item in [str(Path(__file__)),str(base/'results.json'),*command]:
  path=Path(item)
  if path.is_file():r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
run(['python3',root/'benchmarks/execution/concurrent-http-owners/measure.py','--build',out,'--go-client',a.client.resolve(),'--owners','1','--modes','live','--seconds','1','--rounds','1','--output',out/'measurement.json'])
# Normalize only the schema consumed by the existing symbol mapper. Keep the
# original checked measurement, and do not present instrumented timing as capacity.
measurement=json.loads((out/'measurement.json').read_text());assert measurement['passed']
measurement['diagnosticOnly']=True
for trial in measurement['trials']:
 trial['storeCounts']=list(map(int,re.search(r'^STORES ([\d ]+)$',''.join(trial['stderr']),re.M)[1].split()))
(out/'mapped-input.json').write_text(json.dumps(measurement,indent=2)+'\n')
(out/'live').symlink_to('live-1')
# The build records sync then live. Require identical source-index mappings
# before using the latter to map the linked live object.
selection=[]
for c in original['commands']:
 if 'full_http_selection_compiles_with_live_gc' not in ' '.join(c['command']):continue
 selection.append(dict(c,mode='live',sample='plain'))
assert len(selection)==2,len(selection)
def decoded(c):return json.JSONDecoder().raw_decode(c['stdout'][c['stdout'].index('{'):])[0]
assert decoded(selection[0])['functions']==decoded(selection[1])['functions']
selection=selection[-1:]
(out/'selection.json').write_text(json.dumps(dict(commands=selection),indent=2)+'\n')
run(['python3',root/'benchmarks/execution/concurrent-workloads/summarize-sites.py','--measurement',out/'mapped-input.json','--selection',out/'selection.json','--output',out/'summary.json'])
(out/'driver.json').write_text(json.dumps(r,indent=2)+'\n')

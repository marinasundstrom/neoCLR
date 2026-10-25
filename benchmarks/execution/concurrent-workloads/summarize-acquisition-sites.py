"""Attribute diagnostic acquisition sites to retained compiler selections, validating input hashes."""
import argparse, hashlib, json, re
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--profile',type=Path,required=True)
p.add_argument('--root',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();d=json.loads(a.profile.read_text());assert d['passed'] and d['diagnosticOnly'] and d['sites']
r=dict(passed=False,diagnosticOnly=True,samples={},inputs={})
def read(path):
 raw=path.read_bytes();digest=hashlib.sha256(raw).hexdigest()
 assert next(value for key,value in d['inputs'].items() if Path(key).resolve()==path.resolve())==digest
 r['inputs'][str(path)]=digest
 return json.loads(raw)
for trial in d['trials']:
 sample=trial['sample'];ordinary=sample not in ['plain','json']
 retained=read(a.root/('borrowed-managed-ordinary-measure' if ordinary else 'borrowed-managed-http-'+sample)/'results.json')
 # The build's selected object must be one of the hashed native input objects.
 build=a.root/('borrowed-managed-ordinary-build' if ordinary else 'borrowed-managed-http-build')/'results.json'
 data=json.loads(build.read_text())
 entry=next(n for c in data['cases'] if c['sample']==sample for n in c['native'] if n['mode']=='live') if ordinary else next(c for c in data['commands'] if c['sample']==sample)
 obj=Path(entry['selectionCommand'][4]);assert hashlib.sha256(obj.read_bytes()).hexdigest()==d['inputs'][str(obj)]
 stdout=entry['stdout'];selection,_=json.JSONDecoder().raw_decode(stdout[stdout.index('{'):])
 functions={f['compiledIndex']:f for f in selection['functions']}
 rows=[]
 assert sum(site['count'] for site in trial['sites'])==trial['acquisitions']
 assert len({site['address'] for site in trial['sites']})==len(trial['sites'])
 for site in trial['sites']:
  match=re.fullmatch('_neoclr_value_(\d+)',site['symbol'])
  function=functions.get(int(match[1])) if match else None
  rows.append(dict(site,selectedFunction=function,share=site['count']/trial['acquisitions']))
 r['samples'][sample]=dict(acquisitions=trial['acquisitions'],stores=trial['stores'],sites=rows)
 r['inputs'][str(build)]=hashlib.sha256(build.read_bytes()).hexdigest()
r['inputs'][str(a.profile)]=hashlib.sha256(a.profile.read_bytes()).hexdigest();r['passed']=True
a.output.write_text(json.dumps(r,indent=2)+'\n')

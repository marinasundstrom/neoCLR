"""Compare each collector with itself; never mix diagnostic and benchmark timings."""
import argparse,json
from pathlib import Path

def compare(old,new):
 assert old['passed'] and new['passed']
 for key in ['schemaVersion','kind','suite','sampleDefinitionSha256','system','hardware','requests']:
  assert key in old and key in new and old[key]==new[key],('incompatible runs',key)
 assert new['kind']=='benchmark','Profile timings are not benchmark regressions'
 a={(r['sample'],r['collector']):r for r in old['rows']}
 b={(r['sample'],r['collector']):r for r in new['rows']}
 assert len(a)==len(old['rows']) and len(b)==len(new['rows'])
 assert a.keys()==b.keys()
 rows=[]
 for key,x in b.items():
  prior=a[key];assert x['unit']==prior['unit']
  rows.append(dict(sample=key[0],collector=key[1],policyChanged=old['policies'][key[1]]!=new['policies'][key[1]],cpuRatio=x['cpuSeconds']/prior['cpuSeconds'],throughputRatio=x['unitsPerSecond']/prior['unitsPerSecond'],allocationChange=x['allocations']-prior['allocations'],arenaChange=x['peakArenaBytes']-prior['peakArenaBytes']))
 return dict(passed=True,scope='Same-collector ratios; sequential historical runs remain sensitive to host load',rows=rows,previousPolicies=old['policies'],currentPolicies=new['policies'])
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('previous',type=Path);p.add_argument('current',type=Path);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
 a.output.write_text(json.dumps(compare(json.loads(a.previous.read_text()),json.loads(a.current.read_text())),indent=2)+'\n')

"""Summarize static forwarding candidates alongside prior acquisition caller observations."""
import argparse,hashlib,json
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--audit',type=Path,required=True)
p.add_argument('--profile',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();audit=json.loads(a.audit.read_text());profile=json.loads(a.profile.read_text())
assert audit['passed'] and profile['passed'] and profile['diagnosticOnly'] and profile['sites']
assert {r['sample'] for r in audit['commands']}=={'tasks','collections','arrays','plain','json'}
r=dict(passed=False,scope='Static sites and affected-function acquisition observations, not executed-store counts or performance.',samples={},inputs={})
for row in audit['commands']:
 assert row['exitCode']==0 and row['byteIdentical'] and row['forwardedCandidates']>=row['directCandidates']
 changes={x['name']:x for x in row['changedFunctions']};assert len(changes)==len(row['changedFunctions'])
 assert all(0<=c['before']['candidateStores']<c['after']['candidateStores']<=c['after']['guardedStores'] for c in changes.values())
 assert sum(c['after']['candidateStores']-c['before']['candidateStores'] for c in changes.values())==row['forwardedCandidates']-row['directCandidates']
 observed=next(t for t in profile['trials'] if t['sample']==row['sample'])
 assert sum(s['count'] for s in observed['sites'])==observed['acquisitions']
 affected=[s for s in observed['sites'] if s['symbol'].removeprefix('_') in changes]
 count=sum(s['count'] for s in affected)
 r['samples'][row['sample']]=dict(directCandidates=row['directCandidates'],forwardedCandidates=row['forwardedCandidates'],additionalSites=row['forwardedCandidates']-row['directCandidates'],affectedFunctions=len(changes),observedAcquisitionsInAffectedFunctions=count,observedAcquisitionShare=count/observed['acquisitions'],observedSites=affected)
for path in [a.audit,a.profile]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
r['passed']=True;a.output.write_text(json.dumps(r,indent=2)+'\n')

"""Compare measured control CPU gaps; sampled stacks are separate diagnostic evidence."""
import argparse
import hashlib
import json
from pathlib import Path
import statistics

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--root',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args()
r=dict(passed=False,scope=__doc__,samples={},inputs={})
for sample in ['plain','json']:
    path=a.root/('direct-seed-http-'+sample)/'results.json'
    d=json.loads(path.read_text());assert d['passed'] and not d.get('diagnosticOnly')
    modes=['sync','guarded-sync-direct-seed-inline-index','live-direct-seed-inline-index']
    groups={m:[t for t in d['trials'] if t['mode']==m] for m in modes}
    assert all(len(v)==4 for v in groups.values())
    for m,rows in groups.items():
        assert {t['pair'] for t in rows}==set(range(4))
        for t in rows:
            assert t['exitCode']==0 and t['gcProfile']['failures']==0
            done=next(line.split() for line in t['stdout'] if line.startswith('DONE '))
            assert int(done[2])==t['requests']+t['warmup'] and int(done[3])==0
            if m!='sync':
                c=t['candidateCounts'];assert c[0]==c[1]+c[2] and c[5]==0
                assert t['indexMetadata'][1]==0 and t['storeCounts'][4]==0
                assert c==[0]*6 if m.startswith('guarded') else c[1]>0
    assert len({(t['requests'],t['warmup'],t['clientConcurrency']) for rows in groups.values() for t in rows})==1
    med=[statistics.median(t['processCpuSeconds'] for t in groups[m]) for m in modes]
    sync,guarded,live=med;assert 0<sync<guarded<live
    summary_path=a.root/('whole-profile-'+sample+'-summary.json')
    summary=json.loads(summary_path.read_text());assert len(summary['trials'])==3
    assert {t['mode'] for t in summary['trials']}==set(modes)
    stacks=[]
    for t in summary['trials']:
        tls=sum(x['observations'] for x in t['topFrames'] if x['symbol']=='_tlv_get_addr')
        assert tls==sum(x['observations'] for x in t['tlsCallers'])+t['unattributedTlsObservations']
        stacks.append(dict(mode=t['mode'],topFrames=t['topFrames'][:12],tlsCallers=t['tlsCallers'],unattributedTls=t['unattributedTlsObservations']))
    r['samples'][sample]=dict(medianCpuSeconds=dict(zip(modes,med)),guardedShareOfExcess=(guarded-sync)/(live-sync),stacks=stacks,
        limitation='Difference of medians is descriptive, not isolated barrier cost. Modes differ in generated stores, scheduling, live sets and native paths. Wall-stack observations are not CPU percentages.')
    for f in [path,summary_path]:r['inputs'][str(f)]=hashlib.sha256(f.read_bytes()).hexdigest()
r['passed']=True
a.output.write_text(json.dumps(r,indent=2)+'\n')

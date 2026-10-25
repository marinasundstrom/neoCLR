"""Compare measured control CPU gaps; sampled stacks are separate diagnostic evidence."""
import argparse
import hashlib
import json
from pathlib import Path
import statistics

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--root',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--measurement-prefix',default='direct-seed-http')
p.add_argument('--profile-prefix',default='whole-profile')
p.add_argument('--mode-suffix',default='direct-seed-inline-index')
p.add_argument('--verify-profile-builds',action='store_true',help='Require identical compiler options and hashed inputs in diagnostic and uninstrumented cohorts.')
a=p.parse_args()
r=dict(passed=False,scope=__doc__,samples={},inputs={})
for sample in ['plain','json']:
    path=a.root/(a.measurement_prefix+'-'+sample)/'results.json'
    d=json.loads(path.read_text());assert d['passed'] and not d.get('diagnosticOnly')
    modes=['sync','guarded-sync-'+a.mode_suffix,'live-'+a.mode_suffix]
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
    summary_path=a.root/(a.profile_prefix+'-'+sample+'-summary.json')
    summary=json.loads(summary_path.read_text());assert len(summary['trials'])==3
    if a.verify_profile_builds:
        profile_path=a.root/(a.profile_prefix+'-'+sample)/'results.json'
        profile=json.loads(profile_path.read_text())
        assert profile['passed'] and profile.get('diagnosticOnly')
        assert len(profile['trials'])==3 and {t['mode'] for t in profile['trials']}==set(modes)
        for t in profile['trials']:
            assert t['exitCode']==0 and t['gcProfile']['failures']==0
            done=next(line.split() for line in t['stdout'] if line.startswith('DONE '))
            assert int(done[2])==t['requests']+t['warmup'] and int(done[3])==0
            if t['mode']!='sync':
                c=t['candidateCounts'];assert c[0]==c[1]+c[2] and c[5]==0
                assert t['indexMetadata'][1]==0 and t['storeCounts'][4]==0
                assert c==[0]*6 if t['mode'].startswith('guarded') else c[1]>0
        profile_digest=hashlib.sha256(profile_path.read_bytes()).hexdigest()
        assert any(Path(k).resolve()==profile_path.resolve() and v==profile_digest for k,v in summary['inputs'].items())
        def build_signature(report, mode):
            commands=[x['command'] for x in report['commands'] if '-o' in x['command'] and Path(x['command'][x['command'].index('-o')+1]).name==mode]
            assert len(commands)==1
            command=commands[0]; signature=[]
            for i,value in enumerate(command):
                if i==command.index('-o')+1:
                    signature.append('<output>')
                elif str(Path(value).resolve()) in report['inputs'] or Path(value).is_file():
                    digest=hashlib.sha256(Path(value).read_bytes()).hexdigest()
                    assert report['inputs'][str(Path(value).resolve())]==digest, ('changed build input',value)
                    signature.append(digest)
                else:
                    signature.append(value)
            return signature
        for mode in modes:
            assert build_signature(d,mode)==build_signature(profile,mode), ('profile build mismatch',sample,mode)
        r['inputs'][str(profile_path)]=hashlib.sha256(profile_path.read_bytes()).hexdigest()

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

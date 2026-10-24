import argparse, gzip, json, math
from pathlib import Path


def summarize(paths):
    rows=[];seen=set()
    for path in paths:
        report=json.loads(gzip.decompress(path.read_bytes()))
        assert report['passed'] and len(report['trials'])==2
        for trial in report['trials']:
            key=(report['sample'],report['collector'],trial['mode'])
            assert key not in seen;seen.add(key)
            p=trial['pauses'];n=trial['count']
            assert n==1024 and trial['exitCode']==0 and len(p)==74
            assert p[64]>0 and sum(p[:64])==p[64]==sum(p[68:]) and p[73]==0
            assert not p[67] and not trial['gc'][1] and not sum(trial['failures'])
            assert p[65]>=trial['gc'][4] and p[65]>=p[66]
            target=math.ceil(p[64]*.99);cumulative=0
            for bucket,count in enumerate(p[:64]):
                cumulative+=count
                if cumulative>=target:break
            rows.append(dict(sample=key[0],collector=key[1],mode=key[2],requests=n,
                operations=p[64],outerMicrosecondsPerRequest=p[65]/n/1000,
                innerMicrosecondsPerRequest=trial['gc'][4]/n/1000,
                additionalMicrosecondsPerRequest=(p[65]-trial['gc'][4])/n/1000,
                p99BucketUpperMicroseconds=((1<<(bucket+1))-1)/1000,
                maximumMicroseconds=p[66]/1000,operationCounts=p[68:73]))
    assert seen=={(s,c,m) for s in ['plain','json'] for c in ['synchronous','candidate'] for m in ['aot','jit']}
    return rows


if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('inputs',type=Path,nargs=4);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    a.output.write_text(json.dumps(summarize(a.inputs),indent=2)+'\n')

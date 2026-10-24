#!/usr/bin/env python3
"""Separate request-phase GC trigger counts from startup/teardown."""
import argparse,json,gzip
from pathlib import Path
def summarize(report):
    if not report['passed'] or len(report['trials'])!=6:raise ValueError('failed or incomplete diagnostic run')
    result={}
    for mode in ['aot','jit']:
        selected=[r for r in report['trials'] if r['mode']==mode]
        if len(selected)!=3:raise ValueError('duplicate or missing mode')
        rows={r['count']:r for r in selected}
        if set(rows)!={0,32,64}:raise ValueError('expected one 0/32/64 cohort per mode')
        for row in rows.values():
            if any(len(row[k])!=8 for k in ['requests','failures','gc']):raise ValueError('counter shape')
            if sum(row['requests'])!=row['gc'][0] or any(row['failures']) or row['gc'][1]:raise ValueError('unclassified or failed collection entry')
        a,b=rows[32],rows[64]
        result[mode]=dict(requestDelta=32,triggersPerRequest={k:(y-x)/32 for k,x,y in zip(report['fields'],a['requests'],b['requests'])},
            gcPerRequest={k:(y-x)/32 for k,x,y in zip(report['gcFields'],a['gc'],b['gc']) if not k.startswith('peak_')},
            peakLiveBytes=b['gc'][5],peakArenaBytes=b['gc'][6],zeroRequest=dict(triggers=rows[0]['requests'],gc=rows[0]['gc']))
    return result
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('plain',type=Path);p.add_argument('json',type=Path);p.add_argument('output',type=Path);a=p.parse_args()
    def read(path):return json.loads(gzip.decompress(path.read_bytes()) if path.suffix=='.gz' else path.read_bytes())
    a.output.write_text(json.dumps({k:summarize(read(v)) for k,v in [('plain',a.plain),('json',a.json)]},indent=2)+'\n')

"""Correlate a recorded EOF timeout with that process's endpoint-specific close trace."""
import argparse,json,re,hashlib
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--trace',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();assert not a.output.exists()
r={'passed':False,'inputs':{}}
try:
 report=json.loads((a.trace/'check.json').read_text());assert not report['passed'];trial=report['trials'][-1]
 client=next(c for c in reversed(report['clientRuns']) if c['exitCode'])
 match=re.search(r'eofWaitSeconds=([\d.]+).*read tcp 127\.0\.0\.1:(\d+)->127\.0\.0\.1:(\d+): i/o timeout',client['stderr'])
 assert match
 wait,peer,local=match.groups();events=[]
 for path in a.trace.glob('events-'+str(trial['pid'])+'-*.log'):
  r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
  for line in path.read_text().splitlines():
   ts,event,fd,lp,pp,value,error=line.split()
   if (lp,pp)==(local,peer):events.append(dict(ns=int(ts),event=event,fd=int(fd),value=int(value),error=int(error),source=str(path)))
 events.sort(key=lambda x:x['ns']);assert events and all(e['error']==0 for e in events)
 sends=[e for e in events if e['event']=='S'];begins=[e for e in events if e['event']=='B'];closes=[e for e in events if e['event']=='C']
 assert sends and len(begins)==len(closes)==1 and closes[0]['value']==0 and sends[-1]['ns']<=begins[0]['ns']<=closes[0]['ns']
 assert len({e['fd'] for e in events})==1
 r.update(pid=trial['pid'],mode=trial['mode'],workers=trial['workers'],localPort=int(local),peerPort=int(peer),clientEofWaitSeconds=float(wait),events=events,lastSendToCloseCallMicroseconds=(begins[0]['ns']-sends[-1]['ns'])/1000,closeCallMicroseconds=(closes[0]['ns']-begins[0]['ns'])/1000,failureState=trial['failureState'],passed=True)
finally:a.output.write_text(json.dumps(r,indent=2)+'\n')
print(r['lastSendToCloseCallMicroseconds'],r['closeCallMicroseconds'],r['clientEofWaitSeconds'])

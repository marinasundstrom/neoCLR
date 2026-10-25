"""Validate complete per-owner traces against sequential checked request trials."""
import argparse,json,hashlib,statistics
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--trace',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();assert not a.output.exists()
r={'passed':False,'inputs':{},'processes':[]}
try:
 check=json.loads((a.trace/'check.json').read_text());assert check['passed']
 groups={}
 for path in a.trace.glob('events-*.log'):
  pid=int(path.name.split('-')[1]);group=groups.setdefault(pid,dict(pid=pid,first=2**64,delays=[],closeTimes=[],sendCount=0,connections=0,files=[]))
  group['files'].append(str(path));r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest();sent={};closing={}
  for line in path.read_text().splitlines():
   ts,event,fd,local,peer,value,error=line.split();ts=int(ts);value=int(value);error=int(error);key=(fd,local,peer);group['first']=min(group['first'],ts)
   assert event in ['S','B','C']
   if event=='S':
    assert value>0 and error==0 and key not in closing
    sent[key]=ts;group['sendCount']+=1
   elif event=='B':
    assert key not in closing;closing[key]=ts
   else:
    assert key in closing and value==0 and error==0
    group['closeTimes'].append(ts-closing.pop(key))
    if key in sent:group['delays'].append(ts-sent.pop(key));group['connections']+=1
  assert not sent and not closing
 assert len(groups)==len(check['trials'])
 for group,trial in zip(sorted(groups.values(),key=lambda x:x['first']),check['trials']):
  if 'pid' in trial:assert group['pid']==trial['pid']
  # Original cohort did not record PID: serial execution and increasing first
  # trace timestamp establish the process/trial ordering, checked by exact counts.
  assert group['connections']==trial['totalResponses']
  assert len(group['files'])==trial['workers']
  delays=group.pop('delays');close=group.pop('closeTimes')
  group.update(mode=trial['mode'],workers=trial['workers'],repeat=trial['repeat'],maxSendToCloseMs=max(delays)/1e6,medianSendToCloseMs=statistics.median(delays)/1e6,maxCloseMs=max(close)/1e6)
  r['processes'].append(group)
 r['connections']=sum(g['connections'] for g in r['processes']);r['maxSendToCloseMs']=max(g['maxSendToCloseMs'] for g in r['processes']);r['passed']=True
finally:a.output.write_text(json.dumps(r,indent=2)+'\n')
print(r['connections'],r['maxSendToCloseMs'])

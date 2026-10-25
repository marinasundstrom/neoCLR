"""Count same-thread acquisitions/stores in retained workloads; diagnostic timings are not evidence."""
import argparse, hashlib, json, queue, socket, subprocess, threading
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--root',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,diagnosticOnly=True,commands=[],trials=[],inputs={})
def record(path):
 r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
def run(command):
 x=subprocess.run(command,capture_output=True,text=True,timeout=120)
 r['commands'].append(dict(command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode();return x
try:
 for sample in ['tasks','collections','arrays','plain','json']:
  ordinary=sample not in ['plain','json']
  report=a.root/('borrowed-managed-ordinary-measure' if ordinary else 'borrowed-managed-http-'+sample)/'results.json'
  record(report);data=json.loads(report.read_text())
  name=sample+'-live-hybrid-inline' if ordinary else 'live-inline-index'
  command=next(c['command'].copy() for c in data['commands'] if '-o' in c['command'] and Path(c['command'][c['command'].index('-o')+1]).name==name)
  source=next(Path(x) for x in command if x.endswith('/native-gc.c'));record(source)
  text=source.read_text();needle='void *experiment_gc_store_state_borrow(void) { return &live_owner; }';assert text.count(needle)==1
  text=text.replace(needle,'''static _Thread_local uint64_t acquisition_calls, borrowed_calls;
void *experiment_gc_store_state_borrow(void) { acquisition_calls++; return &live_owner; }
__attribute__((destructor)) static void report_acquisitions(void) {
    fprintf(stderr,"ACQUISITIONS %llu %llu\\n",(unsigned long long)acquisition_calls,(unsigned long long)borrowed_calls);
}''')
  needle='    live_owner_state *store_state=opaque;';assert text.count(needle)==1
  text=text.replace(needle,'    borrowed_calls++;\n'+needle)
  modified=out/(sample+'-gc.c');modified.write_text(text);record(modified)
  exe=out/sample;command=[str(modified) if x==str(source) else x for x in command];command[command.index('-o')+1]=str(exe)
  for x in command:
   path=Path(x)
   if path.is_file():record(path)
  run(command)
  if ordinary:
   x=run([str(exe)]);stdout=x.stdout;stderr=x.stderr
   expected=next(t['stdout'] for t in data['trials'] if t['sample']==sample and t['mode']=='live-hybrid-inline')
   assert stdout==expected
  else:
   proc=subprocess.Popen([str(exe)],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
   lines=[];errors=[];events=queue.Queue()
   def read(stream,target):
    for line in stream:target.append(line);events.put(line)
   readers=[threading.Thread(target=read,args=(proc.stdout,lines)),threading.Thread(target=read,args=(proc.stderr,errors))]
   for reader in readers:reader.start()
   try:
    while True:
     line=events.get(timeout=30)
     if line.startswith('READY '):break
    _,port,control=line.split()
    for _ in range(512):
     with socket.create_connection(('127.0.0.1',int(port)),timeout=10) as connection:
      connection.sendall(b'GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n');response=b''
      while True:
       part=connection.recv(4096)
       if not part:break
       response+=part
      header,body=response.split(b'\r\n\r\n',1);assert header.startswith(b'HTTP/1.1 200 ')
      assert body==b'xxxxxxxxxx' if sample=='plain' else json.loads(body)=={'id':42,'name':'Café team','active':True}
    with socket.create_connection(('127.0.0.1',int(control)),timeout=10):pass
    assert proc.wait(timeout=30)==0
   finally:
    if proc.poll() is None:proc.kill();proc.wait(timeout=10)
    for reader in readers:reader.join(timeout=2)
   stdout=''.join(lines);stderr=''.join(errors)
   done=next(x.split() for x in lines if x.startswith('DONE '));assert int(done[2])==512 and int(done[3])==0
  def counts(label):return list(map(int,next(x.split()[1:] for x in stderr.splitlines() if x.startswith(label+' '))))
  acquired,stores=counts('ACQUISITIONS');epochs=counts('CANDIDATE');metadata=counts('INDEX_METADATA')
  assert acquired>0 and stores>0 and epochs[1]>0 and epochs[0]==epochs[1]+epochs[2] and epochs[5]==0 and metadata[1]==0
  r['trials'].append(dict(sample=sample,acquisitions=acquired,stores=stores,storesPerAcquisition=stores/acquired,candidateCounts=epochs,stdout=stdout,stderr=stderr))
 record(Path(__file__));r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

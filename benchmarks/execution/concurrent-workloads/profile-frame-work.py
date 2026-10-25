"""Count activation descriptor work in retained sync/guarded/concurrent workloads; no timing claims."""
import argparse, hashlib, json, queue, socket, subprocess, threading
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--root',type=Path,required=True)
p.add_argument('--prefix',default='forward-elision',help='Retained workload/measurement prefix')
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,diagnosticOnly=True,commands=[],trials=[],inputs={})
def record(path):
 r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
def run(command):
 x=subprocess.run(command,capture_output=True,text=True,timeout=120)
 r['commands'].append(dict(command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode();return x
try:
 for sample,mode in [(s,m) for s in ['tasks','collections','arrays','plain','json'] for m in ['sync','guarded','live']]:
  ordinary=sample not in ['plain','json']
  report=a.root/(a.prefix+'-ordinary-measure' if ordinary else a.prefix+'-http-'+sample)/'results.json'
  record(report);data=json.loads(report.read_text());assert data['passed'] and not data.get('diagnosticOnly')
  mode_name='sync' if mode=='sync' else ('guarded-sync' if mode=='guarded' else 'live')+('-hybrid-inline' if ordinary else '-inline-index')
  name=sample+'-'+mode_name if ordinary else mode_name
  command=next(c['command'].copy() for c in data['commands'] if '-o' in c['command'] and Path(c['command'][c['command'].index('-o')+1]).name==name)
  source=next(Path(x) for x in command if x.endswith('/root-probe.c'));record(source)
  implementation=Path("runtime/native/gc/root-probe.c").resolve();record(implementation)
  assert "runtime/native/gc/root-probe.c" in source.read_text()
  text=implementation.read_text()
  instrumentation=r'''#include <stdio.h>
static _Thread_local uint64_t frame_counts[8], frame_sizes[18];
__attribute__((destructor)) static void report_frame_work(void) {
    fprintf(stderr,"FRAME_WORK");
    for(unsigned i=0;i<8;i++) fprintf(stderr," %llu",(unsigned long long)frame_counts[i]);
    fprintf(stderr,"\nFRAME_SIZES");
    for(unsigned i=0;i<18;i++) fprintf(stderr," %llu",(unsigned long long)frame_sizes[i]);
    fprintf(stderr,"\n");
}
'''
  # Insert after includes so integer definitions are available. No GC/layout change.
  text += '\n'+instrumentation[instrumentation.index('__attribute__'):]
  text = '#include <stdio.h>\n#include <stdint.h>\nstatic _Thread_local uint64_t frame_counts[8], frame_sizes[18];\n'+text
  def replace(old,new):
   global text
   assert text.count(old)==1,(old,text.count(old));text=text.replace(old,new)
  replace('    if (!storage) abort();\n    validate_storage(storage, count, 1);','    frame_counts[0]++; frame_counts[1]+=count; frame_sizes[count<17?count:17]++;\n    if (!storage) abort();\n    validate_storage(storage, count, 1);')
  replace('    uint32_t invalid = 0;','    frame_counts[6]++; frame_counts[7]+=count;\n    uint32_t invalid = 0;')
  replace('    if (!frame || state->roots.head != frame || instruction >= 8192', '    frame_counts[2]++; frame_counts[3]+=lane_count;\n    if (!frame || state->roots.head != frame || instruction >= 8192')
  replace('    if (!frame || state->roots.head != frame || (phase != 1', '    frame_counts[4]++; frame_counts[5]+=count;\n    if (!frame || state->roots.head != frame || (phase != 1')
  modified=out/(sample+'-'+mode+'-roots.c');modified.write_text(text);record(modified)
  exe=out/(sample+'-'+mode);command=[str(modified) if x==str(source) else x for x in command];command[command.index('-o')+1]=str(exe)
  command+=['-I',str(implementation.parent)]
  for x in command:
   path=Path(x)
   if path.is_file():
    if x!=str(modified):assert data['inputs'][str(path.resolve())]==hashlib.sha256(path.read_bytes()).hexdigest(),x
    record(path)
  run(command);record(exe)
  if ordinary:
   x=run([str(exe)]);stdout=x.stdout;stderr=x.stderr
   expected=next(t['stdout'] for t in data['trials'] if t['sample']==sample and t['mode']==mode_name)
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
  values=counts('FRAME_WORK');sizes=counts('FRAME_SIZES')
  assert len(values)==8 and len(sizes)==18 and values[0]>0
  assert sum(sizes)==values[0] and sum(i*c for i,c in enumerate(sizes))<=values[1]
  assert values[6]>=values[0] and values[7]>=values[1]
  row=dict(sample=sample,mode=mode,frameWork=values,storageHistogram=sizes,stdout=stdout,stderr=stderr)
  if mode!='sync':
   epochs=counts('CANDIDATE');metadata=counts('INDEX_METADATA')
   assert epochs[0]==epochs[1]+epochs[2] and epochs[5]==0 and metadata[1]==0
   assert epochs==[0]*6 if mode=='guarded' else epochs[1]>0
   row.update(candidateCounts=epochs,indexMetadata=metadata)
  r['trials'].append(row)
 record(Path(__file__));r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

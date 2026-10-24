"""Run a freshly compiled live plaintext or JSON HTTP object with the owned native host."""
import argparse
import hashlib
import json
import os
import queue
from pathlib import Path
import socket
import subprocess
import threading
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--object',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--requests',type=int,default=256)
p.add_argument('--sample',choices=['plain','json'],default='plain')
p.add_argument('--execution',choices=['aot','jit','both'],default='aot')
p.add_argument('--runner',type=Path)
p.add_argument('--pacing',choices=['forced','paced','both'],default='both')
a=p.parse_args();assert a.requests>0
if a.execution!='aot':assert a.runner is not None
root=Path(__file__).resolve().parents[3];base=Path(__file__).resolve().parent
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r={'passed':False,'scope':f'{a.execution} {a.sample} correctness/lifecycle only, not a capacity benchmark.','commands':[],'trials':[],'inputs':{}}
def run(cmd):
 x=subprocess.run(list(map(str,cmd)),cwd=root,capture_output=True,text=True,timeout=120)
 r['commands'].append({'command':list(map(str,cmd)),'exitCode':x.returncode,'stdout':x.stdout,'stderr':x.stderr})
 x.check_returncode()
try:
 recipe_path=root/'target/native-host-ownership/results.json'
 previous=json.loads(recipe_path.read_text())
 source=out/'native-gc.c';run(['python3',base.parent/'concurrent-live/native-scopes.py','--output',source])
 provider=out/'text-arena.c';run(['python3',base/'native-providers.py','--output',provider])
 host=out/'owned-host.c'
 host_source=(root/'target/native-host-ownership/owned-host.c').read_text()
 # Collector counters are thread-local: report after cleanup on the invoking
 # owner thread, not from a process/library destructor (JIT can run elsewhere).
 boundary='    int32_t status = run_entry(entry, with_args, argc, argv);'
 assert host_source.count(boundary)==1
 host_source=host_source.replace(boundary,boundary+'\n    report_candidate();')
 host.write_text('static void report_candidate(void);\n'+host_source+'''
static void report_candidate(void) {
 extern void experiment_gc_candidate_counts(uint64_t *);
 uint64_t counts[6]; experiment_gc_candidate_counts(counts);
 fprintf(stderr,"CANDIDATE");
 for(unsigned i=0;i<6;i++) fprintf(stderr," %llu",(unsigned long long)counts[i]);
 fprintf(stderr,"\\n");
}
''')
 modes=['aot','jit'] if a.execution=='both' else [a.execution]
 pacing=[False,True] if a.pacing=='both' else [a.pacing=='paced']
 for execution,paced in [(mode,paced) for mode in modes for paced in pacing]:
  old=root/('target/native-host-ownership/owned-aot' if execution=='aot' else 'target/native-host-ownership/owned.dylib')
  recipe=next(c['command'].copy() for c in previous['commands'] if str(old) in c['command'])
  exe=out/(execution+('-paced' if paced else '-forced')+('.dylib' if execution=='jit' else ''))
  command=[]
  for item in recipe:
   if item=='-DNEOCLR_GC_PACED':continue
   if item.endswith('/owned-host.c'):item=str(host)
   elif item.endswith('/native-gc.c'):item=str(source)
   elif item.endswith('/text-arena.c'):item=str(provider)
   elif item==str(root/'target/root-map/candidate.o'):item=str(a.object.resolve())
   command.append(item)
  command[command.index('-o')+1]=str(exe)
  command+=['-I',str(root/'runtime/native/memory'),'-DNEOCLR_GC_PROFILE','-DNEOCLR_SHADOW_SPARSE_INDEX','-I',str(base.parent/'concurrent-mutation-gate'),str(base.parent/'concurrent-mutation-gate/gate.c')]
  if paced:command+=['-DNEOCLR_GC_PACED']
  run(command)
  launch=[str(exe)];env=None
  if execution=='jit':
   selection_path=root/'target/root-map'/('build.json' if a.sample=='plain' else 'json-build.json')
   selection_data=json.loads(selection_path.read_text())
   original=selection_data.get('command') or selection_data['commands'][0]['command']
   selection=[original[0],'--jit-managed',*original[1:4],str(exe),*original[5:]]
   selection_file=out/f'jit-{paced}-command.json';selection_file.write_text(json.dumps(selection,indent=2)+'\n')
   env=dict(os.environ,NEOCLR_LIVE_SELECTION_COMMAND=str(selection_file),NEOCLR_LIVE_SELECTION_MODE='live')
   launch=[str(a.runner.resolve()),'mutation_store::tests::full_http_selection_compiles_with_live_gc','--exact','--ignored','--nocapture']
   for path in [a.runner.resolve(),selection_path,*[Path(x) for x in original if Path(x).is_file()]]:
    r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
  proc=subprocess.Popen(launch,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
  lines=[];errors=[];events=queue.Queue()
  def read(stream,target):
   for line in stream:target.append(line);events.put(line)
  readers=[threading.Thread(target=read,args=(proc.stdout,lines)),threading.Thread(target=read,args=(proc.stderr,errors))]
  for reader in readers:reader.start()
  trial={'execution':execution,'paced':paced,'requested':a.requests,'command':launch};r['trials'].append(trial)
  try:
   while True:
    line=events.get(timeout=240 if execution=='jit' else 30)
    if line.startswith('READY '):break
   _,port,control=line.split()
   for _ in range(a.requests):
    with socket.create_connection(('127.0.0.1',int(port)),timeout=10) as connection:
     connection.sendall(b'GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n')
     response=b''
     while True:
      part=connection.recv(4096)
      if not part:break
      response+=part
     header,body=response.split(b'\r\n\r\n',1)
     assert header.startswith(b'HTTP/1.1 200 ')
     if a.sample=='plain':assert body==b'xxxxxxxxxx'
     else:assert json.loads(body)=={'id':42,'name':'Café team','active':True}
   with socket.create_connection(('127.0.0.1',int(control)),timeout=10):pass
   proc.wait(timeout=30)
   for reader in readers:reader.join(timeout=2)
   assert proc.returncode==0
   if execution=='jit':
    jit_result=json.loads(next(s for s in errors if s.startswith('JIT_RESULT '))[len('JIT_RESULT '):])
    assert jit_result['guestExitCode']==0
    trial['jitResult']=jit_result
   done=next(s for s in lines if s.startswith('DONE ')).split()
   assert int(done[2])==a.requests and int(done[3])==0
   counts=list(map(int,next(s for s in errors if s.startswith('CANDIDATE ')).split()[1:]))
   assert len(counts)==6 and counts[0]==counts[1]+counts[2] and counts[5]==0
   if paced:assert counts[1]>0
   trial['candidateCounts']=counts
  finally:
   if proc.poll() is None:proc.kill()
   proc.wait()
   for reader in readers:reader.join(timeout=2)
   trial.update(exitCode=proc.returncode,stdout=lines,stderr=errors)
 for path in [a.object.resolve(),Path(__file__),recipe_path,host,source,provider,base/'native-providers.py',base.parent/'concurrent-live/native-scopes.py',base.parent/'concurrent-mutation-gate/gate.c',*[Path(x) for x in recipe if Path(x).is_file()],*[p for p in (root/'runtime/native').rglob('*') if p.suffix in ['.c','.h']]]:
  r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

"""Matched sequential HTTP work: sync versus live GC, not saturated capacity."""
import argparse
import hashlib
import json
import queue
from pathlib import Path
import socket
import statistics
import subprocess
import threading
import time
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--sync-object',type=Path,required=True)
p.add_argument('--live-object',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--requests',type=int,default=256)
p.add_argument('--sample',choices=['plain','json'],default='plain')
p.add_argument('--warmup',type=int,default=16)
p.add_argument('--pairs',type=int,default=3)
p.add_argument('--include-guarded-sync',action='store_true')
p.add_argument('--gate-words',type=int,default=4096)
p.add_argument('--disable-store-cache',action='store_true')
p.add_argument('--compare-store-cache',action='store_true')
a=p.parse_args();assert a.requests>0 and a.warmup>=0 and a.pairs>0
assert not (a.disable_store_cache and a.compare_store_cache)
root=Path(__file__).resolve().parents[3];base=Path(__file__).resolve().parent
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r={'passed':False,'gateWords':a.gate_words,'storeCache':not a.disable_store_cache,'compareStoreCache':a.compare_store_cache,'sample':a.sample,'scope':'Sequential loopback AOT fixed-work comparison; no saturation/cross-framework or GC-pause-distribution claim.', 'commands':[],'trials':[],'inputs':{}}
def run(cmd):
 x=subprocess.run(list(map(str,cmd)),cwd=root,capture_output=True,text=True,timeout=120)
 r['commands'].append({'command':list(map(str,cmd)),'exitCode':x.returncode,'stdout':x.stdout,'stderr':x.stderr});x.check_returncode()
def quantile(values,fraction):return sorted(values)[min(len(values)-1,int((len(values)-1)*fraction))]
try:
 recipe_path=root/'target/native-host-ownership/results.json'
 previous=json.loads(recipe_path.read_text());old=root/'target/native-host-ownership/owned-aot'
 recipe=next(c['command'].copy() for c in previous['commands'] if str(old) in c['command'])
 source=out/'native-gc.c';run(['python3',base.parent/'concurrent-live/native-scopes.py','--output',source])
 guarded_source=out/'guarded-sync-gc.c'
 if a.include_guarded_sync:
  code=source.read_text()
  start='candidate_autostart = reason == NEOCLR_GC_TRIGGER_PUBLICATION || reason == NEOCLR_GC_TRIGGER_ALLOCATION;'
  idle='if (!status && !native_scope_depth && reason == NEOCLR_GC_TRIGGER_IDLE && owned_heap.arena == &context->text)'
  assert code.count(start)==1 and code.count(idle)==1
  guarded_source.write_text(code.replace(start,'candidate_autostart = 0; /* Measurement control: synchronous scheduling. */').replace(idle,'if (0 && !status && !native_scope_depth && reason == NEOCLR_GC_TRIGGER_IDLE && owned_heap.arena == &context->text)'))
 provider=out/'text-arena.c';run(['python3',base/'native-providers.py','--output',provider])
 host=out/'owned-host.c'
 host_source=(root/'target/native-host-ownership/owned-host.c').read_text()
 boundary='    int32_t status = run_entry(entry, with_args, argc, argv);'
 assert host_source.count(boundary)==1
 host_source=host_source.replace(boundary,boundary+'\n    report_measurement();')
 host.write_text('static void report_measurement(void);\n'+host_source+'''
#include <sys/resource.h>
static void report_measurement(void) {
 struct rusage usage; if(getrusage(RUSAGE_SELF,&usage)) abort();
 fprintf(stderr,"PROCESS %.9f %.9f %ld\\n",
   usage.ru_utime.tv_sec+usage.ru_utime.tv_usec/1e6,
   usage.ru_stime.tv_sec+usage.ru_stime.tv_usec/1e6,usage.ru_maxrss);
 neoclr_gc_statistics stats=neoclr_gc_statistics_v1();
 fprintf(stderr,"ALLOCATIONS %llu\\n",(unsigned long long)stats.allocations);
 neoclr_gc_profile p=neoclr_gc_profile_v1();
 fprintf(stderr,"PROFILE %llu %llu %llu %llu %llu %llu %llu\\n",
   (unsigned long long)p.attempts,(unsigned long long)p.nanoseconds,
   (unsigned long long)p.maximum_nanoseconds,(unsigned long long)p.failures,
   (unsigned long long)p.allocated_bytes,(unsigned long long)p.peak_live_bytes,
   (unsigned long long)p.peak_arena_used);
#ifdef LIVE_REPORT
 extern void experiment_gc_candidate_counts(uint64_t *);
 extern void experiment_gc_native_scope_counts(uint64_t *);
 extern void experiment_gc_store_counts(uint64_t *);
 extern void experiment_gc_store_lookup_counts(uint64_t *);
 uint64_t counts[6]; experiment_gc_candidate_counts(counts);
 fprintf(stderr,"CANDIDATE");
 for(unsigned i=0;i<6;i++) fprintf(stderr," %llu",(unsigned long long)counts[i]);
 fprintf(stderr,"\\n");
 experiment_gc_store_counts(counts);fprintf(stderr,"STORES");
 for(unsigned i=0;i<5;i++) fprintf(stderr," %llu",(unsigned long long)counts[i]);
 fprintf(stderr,"\\n");
 experiment_gc_store_lookup_counts(counts);fprintf(stderr,"STORE_LOOKUPS %llu %llu\\n",(unsigned long long)counts[0],(unsigned long long)counts[1]);
 experiment_gc_native_scope_counts(counts);fprintf(stderr,"SCOPES");
 for(unsigned i=0;i<5;i++) fprintf(stderr," %llu",(unsigned long long)counts[i]);
 fprintf(stderr,"\\n");
#endif
}
''')
 executables={}
 configurations=[('sync',a.sync_object),('live',a.live_object)]
 if a.include_guarded_sync:configurations.insert(1,('guarded-sync',a.live_object))
 if a.compare_store_cache:
  configurations.append(('live-no-cache',a.live_object))
  if a.include_guarded_sync:configurations.append(('guarded-sync-no-cache',a.live_object))
 for mode,obj in configurations:
  exe=out/mode;command=[]
  for item in recipe:
   if item.endswith('/owned-host.c'):item=str(host)
   elif item.endswith('/native-gc.c') and mode!='sync':item=str(guarded_source if mode.startswith('guarded-sync') else source)
   elif item.endswith('/text-arena.c') and mode!='sync':item=str(provider)
   elif item==str(root/'target/root-map/candidate.o'):item=str(obj.resolve())
   command.append(item)
  assert '-DNEOCLR_GC_PACED' in command
  command[command.index('-o')+1]=str(exe)
  command+=['-I',str(root/'runtime/native/memory'),'-DNEOCLR_GC_PROFILE']
  if mode!='sync':command+=[f'-DGATE_WORDS={a.gate_words}','-DLIVE_REPORT','-DNEOCLR_SHADOW_SPARSE_INDEX','-I',str(base.parent/'concurrent-mutation-gate'),str(base.parent/'concurrent-mutation-gate/gate.c')]
  if mode!='sync' and (a.disable_store_cache or mode.endswith('-no-cache')):command+=['-DNEOCLR_LIVE_NO_STORE_CACHE']
  run(command);executables[mode]=exe
 # Finish every build before timing. Alternate order to expose systematic drift.
 for pair in range(a.pairs):
  for mode in (list(executables) if pair%2==0 else list(reversed(executables))):
   started=time.perf_counter();proc=subprocess.Popen([str(executables[mode])],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
   lines=[];errors=[];events=queue.Queue()
   def read(stream,target):
    for line in stream:target.append(line);events.put(line)
   readers=[threading.Thread(target=read,args=(proc.stdout,lines)),threading.Thread(target=read,args=(proc.stderr,errors))]
   for reader in readers:reader.start()
   trial={'mode':mode,'pair':pair,'requests':a.requests,'warmup':a.warmup};r['trials'].append(trial)
   try:
    while True:
     line=events.get(timeout=30)
     if line.startswith('READY '):break
    _,port,control=line.split();trial['startupSeconds']=time.perf_counter()-started
    latencies=[]
    for index in range(a.warmup+a.requests):
     if index==a.warmup:measured=time.perf_counter()
     before=time.perf_counter()
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
     if index>=a.warmup:latencies.append(time.perf_counter()-before)
    trial['requestSeconds']=time.perf_counter()-measured
    trial['requestsPerSecond']=a.requests/trial['requestSeconds']
    trial['clientLatencyMs']={str(q):quantile(latencies,q)*1000 for q in [.5,.95,.99,1]}
    with socket.create_connection(('127.0.0.1',int(control)),timeout=10):pass
    proc.wait(timeout=30)
    for reader in readers:reader.join(timeout=2)
    assert proc.returncode==0
    done=next(s for s in lines if s.startswith('DONE ')).split()
    assert int(done[2])==a.requests+a.warmup and int(done[3])==0
    def numbers(prefix):return next(s for s in errors if s.startswith(prefix+' ')).split()[1:]
    process=numbers('PROCESS');trial['processCpuSeconds']=float(process[0])+float(process[1]);trial['peakRssBytes']=int(process[2])
    trial['allocations']=int(numbers('ALLOCATIONS')[0])
    profile=list(map(int,numbers('PROFILE')));assert profile[3]==0
    trial['gcProfile']=dict(zip(['attempts','nanoseconds','maximumNanoseconds','failures','allocatedBytes','peakLiveBytes','peakArenaUsed'],profile))
    if mode!='sync':
     counts=list(map(int,numbers('CANDIDATE')));scopes=list(map(int,numbers('SCOPES')))
     assert len(counts)==6 and counts[0]==counts[1]+counts[2] and counts[5]==0 and scopes[4]==0
     trial['candidateCounts']=counts;trial['nativeScopeCounts']=scopes
     trial['storeCounts']=list(map(int,numbers('STORES')))
     trial['storeLookupCounts']=list(map(int,numbers('STORE_LOOKUPS')))
     assert trial['storeCounts'][4]==0
     if mode.startswith('guarded-sync'):assert counts==[0]*6
     else:assert counts[1]>0
     if a.disable_store_cache or mode.endswith('-no-cache'):assert trial['storeLookupCounts'][0]==0
     else:assert trial['storeLookupCounts'][0]>0
   finally:
    if proc.poll() is None:proc.kill()
    proc.wait()
    for reader in readers:reader.join(timeout=2)
    trial.update(exitCode=proc.returncode,stdout=lines,stderr=errors,processWallSeconds=time.perf_counter()-started)
    (out/'progress.json').write_text(json.dumps(r,indent=2)+'\n')
 assert len({t['allocations'] for t in r['trials']})==1
 assert len({t['gcProfile']['allocatedBytes'] for t in r['trials']})==1
 r['summary']={mode:{field:statistics.median(t[field] for t in r['trials'] if t['mode']==mode) for field in ['requestsPerSecond','requestSeconds','processCpuSeconds','peakRssBytes']} for mode in executables}
 for path in [a.sync_object.resolve(),a.live_object.resolve(),Path(__file__),recipe_path,host,source,provider,base/'native-providers.py',base.parent/'concurrent-live/native-scopes.py',base.parent/'concurrent-mutation-gate/gate.c',base.parent/'concurrent-mutation-gate/gate.h',base.parent/'concurrent-live/store.h',*[Path(x) for x in recipe if Path(x).is_file()],*[p for p in (root/'runtime/native').rglob('*') if p.suffix in ['.c','.h']]]:
  r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 if a.include_guarded_sync:r['inputs'][str(guarded_source)]=hashlib.sha256(guarded_source.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

"""Matched fixed-work HTTP clients: sync versus live GC; not cross-framework capacity."""
import argparse
import concurrent.futures
import hashlib
import json
import queue
import shutil
from pathlib import Path
import socket
import statistics
import subprocess
import threading
import time
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--compare-scalar-validation',action='store_true')
p.add_argument('--compare-linear-seed',action='store_true')
p.add_argument('--compare-direct-seed',action='store_true')
p.add_argument('--borrow-stores',action='store_true',help='Use borrowed native stores for current live objects; retain checked previous-object controls.')
p.add_argument('--compare-worker-tls',action='store_true')
p.add_argument('--modes',nargs='+',help='Select unique modes from the configurations enabled by the comparison flags.')
p.add_argument('--compare-inline-base',action='store_true')
p.add_argument('--direct-exact-allocation-index',action='store_true')
p.add_argument('--sync-object',type=Path,required=True)
p.add_argument('--live-object',type=Path,required=True)
p.add_argument('--previous-live-object',type=Path)
p.add_argument('--previous-inline-object',type=Path,help='Compare previous generated code using the same inlined exact-index native host.')
p.add_argument('--output',type=Path,required=True)
p.add_argument('--requests',type=int,default=256)
p.add_argument('--allow-allocation-variation',action='store_true',help='Report scheduling-dependent allocation ranges instead of requiring equality.')
p.add_argument('--concurrency',type=int,default=1, help='Independent connection-close client loops (1..128).')
p.add_argument('--sample',choices=['plain','json'],default='plain')
p.add_argument('--warmup',type=int,default=16)
p.add_argument('--pairs',type=int,default=3)
p.add_argument('--include-guarded-sync',action='store_true')
p.add_argument('--gate-words',type=int,default=4096)
p.add_argument('--disable-store-cache',action='store_true')
p.add_argument('--compare-store-cache',action='store_true')
p.add_argument('--compare-allocation-index',action='store_true')
p.add_argument('--compare-exact-allocation-index',action='store_true')
p.add_argument('--compare-store-tls',action='store_true')
p.add_argument('--compare-store-walk',action='store_true')
p.add_argument('--compare-snapshot-lookup',action='store_true')
p.add_argument('--compare-header-proof',action='store_true')
p.add_argument('--compare-inline-validation',action='store_true')
p.add_argument('--compare-split-tls',action='store_true')
p.add_argument('--compare-serialized-store',action='store_true')
p.add_argument('--compare-exact-store',action='store_true')
p.add_argument('--store-ranges',type=int,choices=range(1,17),default=4)
p.add_argument('--compare-store-ranges',action='store_true')
p.add_argument('--profile-cpu', action='store_true', help='macOS sample capture; diagnostic timings only.')
p.add_argument('--profile-store-walks', action='store_true')
p.add_argument('--profile-store-sites', action='store_true')
p.add_argument('--profile-owner-pauses', action='store_true')
p.add_argument('--profile-finish', action='store_true')
p.add_argument('--profile-collection', action='store_true')
a=p.parse_args();assert not a.profile_collection or a.profile_finish
assert not a.profile_finish or a.profile_owner_pauses
assert a.requests>0 and a.warmup>=0 and a.pairs>0
assert not a.previous_inline_object or (a.compare_inline_base and not a.previous_live_object)
assert not a.compare_linear_seed or a.compare_inline_base
assert not a.compare_direct_seed or a.compare_inline_base
assert not a.compare_worker_tls or a.compare_inline_base
assert not a.compare_inline_base or (a.compare_exact_allocation_index and not a.direct_exact_allocation_index)
assert not a.direct_exact_allocation_index or a.compare_exact_allocation_index
assert not a.compare_exact_allocation_index or a.compare_allocation_index
assert 1<=a.concurrency<=min(128,a.requests)
if a.profile_cpu:
 assert shutil.which('sample') and a.requests>=8192 and not a.profile_store_sites and not a.profile_store_walks
assert not (a.compare_allocation_index and (a.compare_store_tls or a.compare_store_walk or a.compare_snapshot_lookup or a.compare_header_proof or a.compare_inline_validation or a.compare_split_tls or a.compare_serialized_store or a.previous_live_object or a.compare_store_ranges or a.compare_store_cache or a.compare_exact_store or a.disable_store_cache))
assert not (a.compare_store_tls and (a.compare_store_walk or a.compare_snapshot_lookup or a.compare_header_proof or a.compare_inline_validation or a.compare_split_tls or a.compare_serialized_store or a.previous_live_object or a.compare_store_ranges or a.compare_store_cache or a.compare_exact_store or a.disable_store_cache))
assert not (a.compare_store_walk and (a.compare_snapshot_lookup or a.compare_header_proof or a.compare_inline_validation or a.compare_split_tls or a.compare_serialized_store or a.previous_live_object or a.compare_store_ranges or a.compare_store_cache or a.compare_exact_store or a.disable_store_cache))
assert not (a.compare_snapshot_lookup and (a.compare_header_proof or a.compare_inline_validation or a.compare_split_tls or a.compare_serialized_store or a.previous_live_object or a.compare_store_ranges or a.compare_store_cache or a.compare_exact_store or a.disable_store_cache))
assert not (a.compare_header_proof and (a.compare_inline_validation or a.compare_split_tls or a.compare_serialized_store or a.previous_live_object or a.compare_store_ranges or a.compare_store_cache or a.compare_exact_store or a.disable_store_cache))
assert not (a.compare_inline_validation and (a.compare_split_tls or a.compare_serialized_store or a.previous_live_object or a.compare_store_ranges or a.compare_store_cache or a.compare_exact_store or a.disable_store_cache))
assert not (a.compare_split_tls and (a.compare_serialized_store or a.previous_live_object or a.compare_store_ranges or a.compare_store_cache or a.compare_exact_store or a.disable_store_cache))
assert not (a.compare_serialized_store and (a.previous_live_object or a.compare_store_ranges or a.compare_store_cache or a.compare_exact_store or a.disable_store_cache))
assert not (a.disable_store_cache and a.compare_store_cache)
assert not (a.compare_exact_store and (a.disable_store_cache or a.compare_store_cache))
assert not (a.previous_live_object and (a.compare_store_ranges or a.compare_store_cache or a.compare_exact_store))
assert not (a.compare_store_ranges and (a.disable_store_cache or a.compare_store_cache or a.compare_exact_store or a.store_ranges==1))
root=Path(__file__).resolve().parents[3];base=Path(__file__).resolve().parent
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r={'passed':False,'compareWorkerTls':a.compare_worker_tls,'selectedModes':a.modes,'compareInlineBase':a.compare_inline_base,'directExactAllocationIndex':a.direct_exact_allocation_index,'compareExactAllocationIndex':a.compare_exact_allocation_index,'compareAllocationIndex':a.compare_allocation_index,'compareStoreTls':a.compare_store_tls,'compareStoreWalk':a.compare_store_walk,'allowAllocationVariation':a.allow_allocation_variation,'concurrency':a.concurrency,'compareSnapshotLookup':a.compare_snapshot_lookup,'compareHeaderProof':a.compare_header_proof,'compareInlineValidation':a.compare_inline_validation,'compareSplitTls':a.compare_split_tls,'compareSerializedStore':a.compare_serialized_store,'gateWords':a.gate_words,'storeCache':not a.disable_store_cache,'compareStoreCache':a.compare_store_cache,'compareExactStore':a.compare_exact_store,'storeRanges':a.store_ranges,'compareStoreRanges':a.compare_store_ranges,'sample':a.sample,'scope':'Loopback AOT fixed-work comparison; Python client scheduling and connection-close costs included. No saturation/cross-framework or GC-pause-distribution claim.', 'commands':[],'trials':[],'inputs':{}}
if a.profile_store_walks:r['diagnosticOnly']='Heap-walk instrumentation; timings are not benchmark evidence.'
if a.profile_owner_pauses:r['diagnosticOnly']='Owner-operation wall histograms; timings are not throughput evidence.'
if a.profile_cpu:r['diagnosticOnly']='External CPU sampling affects timings; do not use this cohort for throughput claims.'
def run(cmd):
 x=subprocess.run(list(map(str,cmd)),cwd=root,capture_output=True,text=True,timeout=120)
 r['commands'].append({'command':list(map(str,cmd)),'exitCode':x.returncode,'stdout':x.stdout,'stderr':x.stderr});x.check_returncode()
def quantile(values,fraction):return sorted(values)[min(len(values)-1,int((len(values)-1)*fraction))]
try:
 recipe_path=root/'target/native-host-ownership/results.json'
 previous=json.loads(recipe_path.read_text());old=root/'target/native-host-ownership/owned-aot'
 recipe=next(c['command'].copy() for c in previous['commands'] if str(old) in c['command'])
 source=out/'native-gc.c';run(['python3',base.parent/'concurrent-live/native-scopes.py','--output',source])
 if a.profile_store_sites:
  r['diagnosticOnly']='Call-site instrumentation; timings are not benchmark evidence.'
  run(['python3',base.parent/'concurrent-workloads/store-sites.py','--source',source])
 guarded_source=out/'guarded-sync-gc.c'
 if a.include_guarded_sync:
  code=source.read_text()
  start='candidate_autostart = reason == NEOCLR_GC_TRIGGER_PUBLICATION || reason == NEOCLR_GC_TRIGGER_ALLOCATION;'
  idle='if (!status && !native_scope_depth && reason == NEOCLR_GC_TRIGGER_IDLE && owned_heap.arena == &context->text)'
  assert code.count(start)==1 and code.count(idle)==1
  guarded_source.write_text(code.replace(start,'candidate_autostart = 0; /* Measurement control: synchronous scheduling. */').replace(idle,'if (0 && !status && !native_scope_depth && reason == NEOCLR_GC_TRIGGER_IDLE && owned_heap.arena == &context->text)'))
 provider=out/'text-arena.c';run(['python3',base/'native-providers.py','--output',provider])
 sync_profile=out/'sync-profile-gc.c'
 if a.profile_owner_pauses:
  profiler=base.parent/'concurrent-workloads/owner-pauses.py'
  run(['python3',profiler,'--source',root/'runtime/native/gc/native-gc.c','--output',sync_profile])
  for candidate in [source]+([guarded_source] if a.include_guarded_sync else []):
   run(['python3',profiler,'--source',candidate,'--output',candidate,'--live']+(['--finish-details'] if a.profile_finish else []) + (['--collection-details'] if a.profile_collection else []))
 host=out/'owned-host.c'
 host_source=(root/'target/native-host-ownership/owned-host.c').read_text()
 boundary='    int32_t status = run_entry(entry, with_args, argc, argv);'
 assert host_source.count(boundary)==1
 host_source=host_source.replace(boundary,boundary+'\n    report_measurement();')
 host.write_text('static void report_measurement(void);\n'+host_source+'''
#include <sys/resource.h>
static void report_measurement(void) {
#ifdef OWNER_PAUSE_PROFILE
 extern void experiment_owner_pauses_report(void);
 experiment_owner_pauses_report();
#endif
#ifdef FINISH_PHASE_PROFILE
 extern void experiment_finish_phases_report(void);
 experiment_finish_phases_report();
#endif
#ifdef COLLECTION_PART_PROFILE
 extern void experiment_collection_parts_report(void);
 experiment_collection_parts_report();
#endif
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
#ifdef NEOCLR_STORE_SITES
 extern void experiment_store_sites_report(void);
 experiment_store_sites_report();
#endif
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
#ifdef NEOCLR_LIVE_ALLOCATION_INDEX
 extern void experiment_gc_index_counts(uint64_t *);
 experiment_gc_index_counts(counts);
 fprintf(stderr,"INDEX_METADATA %llu %llu %llu\\n",(unsigned long long)counts[0],(unsigned long long)counts[1],(unsigned long long)counts[2]);
#endif
#ifdef NEOCLR_LIVE_WALK_PROFILE
 extern void experiment_gc_store_walk_counts(uint64_t *);
 experiment_gc_store_walk_counts(counts);
 fprintf(stderr,"STORE_WALKS %llu %llu %llu\\n",(unsigned long long)counts[0],(unsigned long long)counts[1],(unsigned long long)counts[2]);
#endif
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
 if a.compare_allocation_index:
  configurations.append(('live-index',a.live_object))
  if a.include_guarded_sync:configurations.append(('guarded-sync-index',a.live_object))
 if a.compare_exact_allocation_index:
  configurations.append(('live-exact-index',a.live_object))
  if a.include_guarded_sync:configurations.append(('guarded-sync-exact-index',a.live_object))
 if a.compare_inline_base:
  configurations.append(('live-inline-index',a.live_object))
  if a.include_guarded_sync: configurations.append(('guarded-sync-inline-index',a.live_object))
 if a.compare_worker_tls:
  configurations.append(('live-worker-inline-index',a.live_object))
  if a.include_guarded_sync: configurations.append(('guarded-sync-worker-inline-index',a.live_object))
 if a.compare_store_tls:
  configurations.append(('live-split-store-tls',a.live_object))
  if a.include_guarded_sync:configurations.append(('guarded-sync-split-store-tls',a.live_object))
 if a.compare_store_walk:
  configurations.append(('live-full-walk',a.live_object))
  if a.include_guarded_sync:configurations.append(('guarded-sync-full-walk',a.live_object))
 if a.compare_snapshot_lookup:
  configurations.append(('live-repeat-lookup',a.live_object))
  if a.include_guarded_sync:configurations.append(('guarded-sync-repeat-lookup',a.live_object))
 if a.compare_header_proof:
  configurations.append(('live-header-proof',a.live_object))
  if a.include_guarded_sync:configurations.append(('guarded-sync-header-proof',a.live_object))
 if a.compare_inline_validation:
  configurations.append(('live-inline',a.live_object))
  if a.include_guarded_sync:configurations.append(('guarded-sync-inline',a.live_object))
 if a.compare_split_tls:
  configurations.append(('live-split-tls',a.live_object))
  if a.include_guarded_sync:configurations.append(('guarded-sync-split-tls',a.live_object))
 if a.compare_serialized_store:
  configurations.append(('live-serialized',a.live_object))
  if a.include_guarded_sync:configurations.append(('guarded-sync-serialized',a.live_object))
 if a.compare_exact_store:
  configurations.append(('live-no-exact',a.live_object))
  if a.include_guarded_sync:configurations.append(('guarded-sync-no-exact',a.live_object))
 if a.compare_store_ranges:
  configurations.append(('live-one-range',a.live_object))
  if a.include_guarded_sync:configurations.append(('guarded-sync-one-range',a.live_object))
 if a.previous_live_object:
  configurations.append(('live-previous-code',a.previous_live_object))
  if a.include_guarded_sync:configurations.append(('guarded-sync-previous-code',a.previous_live_object))
 if a.previous_inline_object:
  configurations.append(('live-previous-inline-index',a.previous_inline_object))
  if a.include_guarded_sync: configurations.append(('guarded-sync-previous-inline-index',a.previous_inline_object))
 if a.compare_linear_seed:
  configurations.append(('live-linear-seed-inline-index',a.live_object))
  if a.include_guarded_sync:configurations.append(('guarded-sync-linear-seed-inline-index',a.live_object))
 if a.compare_direct_seed:
  configurations.append(('live-direct-seed-inline-index',a.live_object))
  if a.include_guarded_sync:configurations.append(('guarded-sync-direct-seed-inline-index',a.live_object))
 if a.modes:
  assert len(set(a.modes))==len(a.modes) and set(a.modes)<=set(mode for mode,_ in configurations)
  configurations=[(mode,obj) for mode,obj in configurations if mode in a.modes]
 for mode,obj in configurations:
  exe=out/mode;command=[]
  for item in recipe:
   if item.endswith('/owned-host.c'):item=str(host)
   elif item.endswith('/native-gc.c') and mode=='sync' and a.profile_owner_pauses:item=str(sync_profile)
   elif item.endswith('/native-gc.c') and mode!='sync':item=str(guarded_source if mode.startswith('guarded-sync') else source)
   elif item.endswith('/text-arena.c') and mode!='sync':item=str(provider)
   elif item==str(root/'target/root-map/candidate.o'):item=str(obj.resolve())
   command.append(item)
  assert '-DNEOCLR_GC_PACED' in command
  command[command.index('-o')+1]=str(exe)
  command+=['-I',str(root/'runtime/native/memory'),'-DNEOCLR_GC_PROFILE']
  if mode!='sync':command+=[f'-DGATE_WORDS={a.gate_words}','-DLIVE_REPORT','-DNEOCLR_SHADOW_SPARSE_INDEX','-I',str(base.parent/'concurrent-mutation-gate'),str(base.parent/'concurrent-mutation-gate/gate.c')]
  if mode!='sync' and (a.disable_store_cache or mode.endswith('-no-cache')):command+=['-DNEOCLR_LIVE_NO_STORE_CACHE']
  if mode.endswith('-index'):command+=['-DNEOCLR_LIVE_ALLOCATION_INDEX']
  if a.direct_exact_allocation_index and mode.endswith('-exact-index'):command+=['-DNEOCLR_LIVE_DIRECT_EXACT_INDEX']
  if mode.endswith('-worker-inline-index'):command+=['-DNEOCLR_LIVE_GROUP_WORKER_TLS']
  if mode.endswith('-inline-index'):command+=['-DNEOCLR_LIVE_INLINE_BASE','-DNEOCLR_LIVE_EXACT_INDEX']
  if mode.endswith('-exact-index'):command+=['-DNEOCLR_LIVE_EXACT_INDEX']
  if mode.endswith('-split-store-tls'):command+=['-DNEOCLR_LIVE_SPLIT_STORE_TLS']
  if mode.endswith('-full-walk'):command+=['-DNEOCLR_LIVE_FULL_STORE_WALK']
  if mode.endswith('-repeat-lookup'):command+=['-DNEOCLR_LIVE_REPEAT_SNAPSHOT_LOOKUP']
  if mode.endswith('-header-proof'):command+=['-DNEOCLR_LIVE_HEADER_PROOF']
  if mode.endswith('-inline'):command+=['-DNEOCLR_LIVE_INLINE_VALIDATION']
  if mode.endswith('-split-tls'):command+=['-DNEOCLR_LIVE_SPLIT_TLS']
  if mode.endswith('-serialized'):command+=['-DNEOCLR_LIVE_SERIALIZED_STORE']
  if mode.endswith('-no-exact'):command+=['-DNEOCLR_LIVE_NO_EXACT_STORE']
  command += [f'-DNEOCLR_LIVE_STORE_RANGES={1 if mode.endswith("-one-range") else a.store_ranges}']
  if a.profile_store_walks and mode!='sync':command+=['-DNEOCLR_LIVE_WALK_PROFILE']
  if a.profile_store_sites and mode!='sync':command+=['-DNEOCLR_STORE_SITES']
  if a.profile_owner_pauses:command+=['-DOWNER_PAUSE_PROFILE','-I',str(root/'runtime/native/gc')]
  if a.profile_finish and mode!='sync':command+=['-DFINISH_PHASE_PROFILE']
  if a.profile_collection and mode!='sync':command+=['-DCOLLECTION_PART_PROFILE']
  if '-linear-seed-' in mode:command+=['-DNEOCLR_LIVE_LINEAR_SEED']
  if '-direct-seed-' in mode:command+=['-DNEOCLR_LIVE_DIRECT_SEED']
  if a.borrow_stores and mode!='sync' and '-previous-' not in mode:command+=['-DNEOCLR_LIVE_STORE_HANDLE']
  if a.compare_scalar_validation and mode != 'sync' and 'previous' not in mode: command += ['-DNEOCLR_GC_SCALAR_STORAGE_VALIDATION']
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
   profiler=None
   trial={'mode':mode,'pair':pair,'requests':a.requests,'warmup':a.warmup};r['trials'].append(trial)
   try:
    while True:
     line=events.get(timeout=30)
     if line.startswith('READY '):break
    _,port,control=line.split();trial['startupSeconds']=time.perf_counter()-started
    def request():
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
     return time.perf_counter()-before
    for _ in range(a.warmup):request()
    if a.profile_cpu:
     profile_path=out/f'{mode}-{pair}-cpu.txt'
     profile_command=['sample',str(proc.pid),'1','-file',str(profile_path)]
     profiler=subprocess.Popen(profile_command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
     trial['cpuProfile']={'command':profile_command,'path':str(profile_path)}
    if a.concurrency==1:
     client_cpu=time.process_time();measured=time.perf_counter()
     latencies=[request() for _ in range(a.requests)]
    else:
     ready=threading.Barrier(a.concurrency+1,timeout=10)
     start_requests=threading.Event()
     def client(count):
      ready.wait()
      assert start_requests.wait(timeout=10)
      return [request() for _ in range(count)]
     with concurrent.futures.ThreadPoolExecutor(max_workers=a.concurrency) as pool:
      futures=[pool.submit(client,a.requests//a.concurrency+(worker<a.requests%a.concurrency)) for worker in range(a.concurrency)]
      ready.wait()
      client_cpu=time.process_time();measured=time.perf_counter()
      start_requests.set()
      latencies=[latency for future in futures for latency in future.result()]
    trial['clientCpuSeconds']=time.process_time()-client_cpu
    trial['clientConcurrency']=a.concurrency
    assert len(latencies)==a.requests
    trial['requestSeconds']=time.perf_counter()-measured
    trial['requestsPerSecond']=a.requests/trial['requestSeconds']
    trial['clientLatencyMs']={str(q):quantile(latencies,q)*1000 for q in [.5,.95,.99,1]}
    with socket.create_connection(('127.0.0.1',int(control)),timeout=10):pass
    proc.wait(timeout=30)
    for reader in readers:reader.join(timeout=2)
    if profiler is not None:
     profile_stdout,profile_stderr=profiler.communicate(timeout=30)
     trial['cpuProfile'].update(exitCode=profiler.returncode,stdout=profile_stdout,stderr=profile_stderr)
     assert profiler.returncode==0 and profile_path.is_file() and 'Call graph:' in profile_path.read_text()
     trial['cpuProfile']['sha256']=hashlib.sha256(profile_path.read_bytes()).hexdigest()
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
     if mode.endswith('-index'):
      trial['indexMetadata']=list(map(int,numbers('INDEX_METADATA')))
      assert len(trial['indexMetadata'])==3 and trial['indexMetadata'][0]>0 and trial['indexMetadata'][1]==0
     trial['storeCounts']=list(map(int,numbers('STORES')))
     if a.profile_store_walks:trial['storeWalkCounts']=list(map(int,numbers('STORE_WALKS')))
     trial['storeLookupCounts']=list(map(int,numbers('STORE_LOOKUPS')))
     assert trial['storeCounts'][4]==0
     if mode.startswith('guarded-sync'):assert counts==[0]*6
     else:assert counts[1]>0
     if a.disable_store_cache or mode.endswith('-no-cache'):assert trial['storeLookupCounts'][0]==0
     else:assert trial['storeLookupCounts'][0]>0
   finally:
    if profiler is not None and profiler.poll() is None:
     profiler.kill();profiler.communicate(timeout=10)
    if proc.poll() is None:proc.kill()
    proc.wait()
    for reader in readers:reader.join(timeout=2)
    trial.update(exitCode=proc.returncode,stdout=lines,stderr=errors,processWallSeconds=time.perf_counter()-started)
    (out/'progress.json').write_text(json.dumps(r,indent=2)+'\n')
 r['allocationEquality']=len({(t['allocations'],t['gcProfile']['allocatedBytes']) for t in r['trials']})==1
 if not a.allow_allocation_variation:assert r['allocationEquality']
 r['summary']={mode:{field:statistics.median(t[field] for t in r['trials'] if t['mode']==mode) for field in ['requestsPerSecond','requestSeconds','processCpuSeconds','peakRssBytes']} for mode in executables}
 for path in [a.sync_object.resolve(),a.live_object.resolve(),Path(__file__),recipe_path,host,source,provider,base/'native-providers.py',base.parent/'concurrent-live/native-scopes.py',base.parent/'concurrent-mutation-gate/gate.c',base.parent/'concurrent-mutation-gate/gate.h',base.parent/'concurrent-live/store.h',*[Path(x) for x in recipe if Path(x).is_file()],*[p for p in (root/'runtime/native').rglob('*') if p.suffix in ['.c','.h']]]:
  r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 if a.previous_live_object:r['inputs'][str(a.previous_live_object.resolve())]=hashlib.sha256(a.previous_live_object.read_bytes()).hexdigest()
 if a.previous_inline_object:r['inputs'][str(a.previous_inline_object.resolve())]=hashlib.sha256(a.previous_inline_object.read_bytes()).hexdigest()
 if a.include_guarded_sync:r['inputs'][str(guarded_source)]=hashlib.sha256(guarded_source.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

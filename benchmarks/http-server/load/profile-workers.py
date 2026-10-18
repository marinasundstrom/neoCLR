#!/usr/bin/env python3
"""Untimed worker-count attribution with existing HTTP allocation/GC instrumentation."""
import argparse
import asyncio
import importlib.util
import json
import os
import socket
from pathlib import Path
import subprocess
import sys
ROOT=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(ROOT/'benchmarks/http-server'))
spec=importlib.util.spec_from_file_location('http_profile',ROOT/'benchmarks/http-server/profile.py')
profile=importlib.util.module_from_spec(spec);spec.loader.exec_module(profile)
spec=importlib.util.spec_from_file_location('load_driver',Path(__file__).with_name('run.py'))
load=importlib.util.module_from_spec(spec);spec.loader.exec_module(load)
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--control-stop',action='store_true')
p.add_argument('--workers',type=int,nargs='+',default=[1,4,16])
p.add_argument('--build',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
build=json.loads(a.build.read_text())
cmd=next(c['command'][:] for c in build['commands'] if c.get('phase')=='host-compile-link')
socket_source=ROOT/'docs/experiments/aot-console/socket-listener.c'
text=socket_source.read_text();hook='        neoclr_os_close(scope->slots[i].descriptor);';assert text.count(hook)==1
socket_copy=out/'socket.c';socket_copy.write_text('extern void neoclr_http_profile_request(void);\n'+text.replace(hook,'        if (!scope->slots[i].listener) neoclr_http_profile_request();\n'+hook))
collector=ROOT/'docs/experiments/aot-console/native-gc.c'
text=(ROOT/'runtime/native/gc/native-gc.c').read_text();hook='    statistics.allocations++;';assert text.count(hook)==1
collector_copy=out/'collector.c';collector_copy.write_text('#include <stdint.h>\nextern void neoclr_http_profile_allocated(uint32_t, uint64_t);\n'+text.replace(hook,hook+'\n    neoclr_http_profile_allocated(kind, bytes);'))
cmd=[str(socket_copy) if v==str(socket_source) else str(collector_copy) if v==str(collector) else v for v in cmd]
cmd+=['-DNEOCLR_GC_PROFILE','-DNEOCLR_ALLOCATION_SITES',str(ROOT/'benchmarks/http-server/profile-gc.c'),str(ROOT/'benchmarks/http-server/allocation-sites.c')]
report=dict(scope='Untimed 32 sequential plain requests plus /stop; allocation window closes 8..32 excludes stop. GC totals include startup/shutdown. Same guest object, native source and heap; vary worker count and explicit GC policy only.',build=build,controlStop=a.control_stop,workerCounts=a.workers,inputs={},links=[],runs=[])
for item in cmd:
 if Path(item).is_file():report['inputs'][str(Path(item).resolve())]=profile.sha(item)
for item in (ROOT/'runtime/native').rglob('*'):
 if item.suffix in ('.c','.h'):report['inputs'][str(item)]=profile.sha(item)
try:
 for policy in ('stress','paced'):
  command=cmd[:];app=out/('app-'+policy);command[command.index('-o')+1]=str(app)
  if policy=='paced':command+=['-DNEOCLR_GC_PACED']
  linked=subprocess.run(command,capture_output=True,text=True)
  report['links'].append(dict(command=command,exitCode=linked.returncode,stderr=linked.stderr))
  linked.check_returncode()
  for workers in a.workers:
   output=out/f'{policy}-{workers}.jsonl'
   guest=load.Guest(['/usr/bin/env','NEOCLR_HTTP_PROFILE_OUTPUT='+str(output),str(app),str(workers)])
   try:
    port,_=guest.line()
    control_port=int(guest.line()[0]) if a.control_stop else None
    for _ in range(32):assert asyncio.run(load.request(int(port),'/plain',10,15))=='ok'
    assert asyncio.run(load.request(int(port),'/stop',0,20))=='ok'
    if control_port is not None:
     with socket.create_connection(('127.0.0.1',control_port),timeout=10):pass
    guest.line(b'DONE');assert int(guest.line()[0])==32;assert int(guest.line()[0])==0;assert int(guest.line()[0])==0;guest.finish()
   finally:guest.close()
   counters=[json.loads(line) for line in output.read_text().splitlines()]
   assert counters[0]['failures']==0 and counters[0]['allocations']==counters[0]['reclaimed']
   assert counters[1]['requests']==(34 if a.control_stop else 33)
   report['runs'].append(dict(policy=policy,workers=workers,gc=counters[0],requestAllocations=counters[1],allocationSites=profile.describe_sites(counters[2]['sites'],build,counters[1])))
   print(policy,workers,counters[0],flush=True)
   (out/'results.json').write_text(json.dumps(report,indent=2)+'\n')
 for item,digest in report['inputs'].items():assert profile.sha(item)==digest,item
 report['passed']=True
finally:(out/'results.json').write_text(json.dumps(report,indent=2)+'\n')

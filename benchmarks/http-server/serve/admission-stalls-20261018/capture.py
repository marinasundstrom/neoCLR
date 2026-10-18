#!/usr/bin/env python3
"""Relink fixed Serve with bounded diagnostic GC/scheduler interval tracing."""
import argparse, hashlib, json, os, subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[3]
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--build',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--duration',type=float,default=20)
p.add_argument('--rate',type=float,default=400)
a=p.parse_args();b=a.build.resolve();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
m=json.loads((b/'build.json').read_text());assert m['passed']
for src,name,changes in [(ROOT/'runtime/native/gc/native-gc.c','native-gc.c',[('int32_t neoclr_gc_collect_v1(', 'int32_t trace_actual_collect(')]),
    (ROOT/'docs/experiments/aot-console/socket-listener.c','socket-listener.c',[
    ('void neoclr_scheduler_wait_v1(', 'void trace_actual_wait('),('int32_t neoclr_scheduler_acquire_v1(', 'int32_t trace_actual_acquire(')])]:
 s=src.read_text()
 for old,new in changes:
  assert s.count(old)==1;s=s.replace(old,new)
 (out/name).write_text(s)
# The async pump delegates to the shared wait hook, so it is recorded once.
s=(ROOT/'runtime/native/async/native-runtime.c').read_text()
assert 'if (!ready) neoclr_scheduler_wait_v1(c);' in s
(out/'native-runtime.c').write_text(s)
c=next(c['command'][:] for c in reversed(m['commands']) if c['phase']=='host-compile-link')
for name in ['native-gc.c','socket-listener.c']:
 assert sum(x.endswith('/aot-console/'+name) for x in c)==1
 c=[str(out/name) if x.endswith('/aot-console/'+name) else x for x in c]
c=[str(out/'native-runtime.c') if x.endswith('/async/native-runtime.c') else x for x in c]
c[c.index('-o')+1]=str(out/'app');c+=['-I',str(ROOT/'runtime/native/gc'),'-I',str(ROOT/'docs/experiments/aot-console'),str(HERE/'trace.c')]
r=subprocess.run(c,capture_output=True,text=True)
inputs=[Path(x) for x in c if Path(x).is_file()]+[b/'build.json',Path(__file__),ROOT/'benchmarks/http-server/load/run.py']
(out/'build.json').write_text(json.dumps(dict(command=c,exitCode=r.returncode,stderr=r.stderr,inputs={str(x):hashlib.sha256(x.read_bytes()).hexdigest() for x in inputs}),indent=2)+'\n');r.check_returncode()
subprocess.run(['python3',str(ROOT/'benchmarks/http-server/serve/diagnose.py'),'--executable',str(out/'app'),'--rate',str(a.rate),'--duration',str(a.duration),'--output',str(out/'requests')],env=dict(os.environ,NEOCLR_STALL_TRACE=str(out/'trace.json')),check=True)

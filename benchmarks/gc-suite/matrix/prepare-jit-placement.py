"""Pair system/arena JIT allocation with identical qualified native hosts.
The arena switch is test-runner-only and reserves 64 MiB of address space.
"""
import argparse
import hashlib
import json
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--build',type=Path,required=True)
p.add_argument('--runner',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--policy',choices=['fixed','fallback'],default='fixed')
a=p.parse_args();root=Path(__file__).resolve().parents[3]
source=a.build.resolve()/'build.json';built=json.loads(source.read_text());assert built['passed']
for path,digest in built['inputs'].items():
 assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest,('stale build',path)
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=True,scope=__doc__,arenaReservationBytes=64*1024*1024,arenaPolicy=a.policy,recipes={},inputs=dict(built['inputs']))
for mode in ['sync','live']:
 old=built['recipes'][mode+'-jit'];command=old['command'].copy();command[0]=str(a.runner.resolve())
 env=dict(old['env']);env.pop('NEOCLR_TEST_JIT_ARENA',None)
 r['recipes'][mode+'-jit']=dict(old,command=command,env=env)
 prefix='fallback' if a.policy=='fallback' else 'arena'
 r['recipes'][prefix+'-'+mode+'-jit']=dict(old,command=command,env=dict(env,NEOCLR_TEST_JIT_ARENA='fallback' if a.policy=='fallback' else '1'))
for path in [source,a.runner.resolve(),Path(__file__).resolve(),root/'runtime/codegen/src/jit_memory.rs',root/'runtime/codegen/src/jit.rs',root/'runtime/codegen/src/jit_memory_fallback.rs']:
 r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
(out/'build.json').write_text(json.dumps(r,indent=2)+'\n')

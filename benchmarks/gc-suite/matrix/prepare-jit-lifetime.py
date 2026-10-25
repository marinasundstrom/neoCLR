"""Compare retained/released compilation state with identical JIT placement/hosts."""
import argparse
import hashlib
import json
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--build',type=Path,required=True)
p.add_argument('--runner',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();root=Path(__file__).resolve().parents[3]
source=a.build.resolve()/'build.json';built=json.loads(source.read_text());assert built['passed']
for path,digest in built['inputs'].items():
 assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest,('stale build',path)
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=True,scope=__doc__,recipes={},inputs=dict(built['inputs']))
for mode in ['sync','live']:
 old=built['recipes'][mode+'-jit'];command=old['command'].copy();command[0]=str(a.runner.resolve())
 env=dict(old['env'],NEOCLR_TEST_JIT_ARENA='fallback');env.pop('NEOCLR_TEST_KEEP_JIT_COMPILATION',None)
 for retained in [True,False]:
  selected=dict(env)
  if retained:selected['NEOCLR_TEST_KEEP_JIT_COMPILATION']='1'
  r['recipes'][('retained-' if retained else 'released-')+mode+'-jit']=dict(old,command=command,env=selected,compilationStateReleased=not retained)
paths=[source,a.runner.resolve(),Path(__file__).resolve()]
paths += [root/'runtime/codegen/src'/name for name in ['lib.rs','jit.rs','jit_memory.rs','jit_memory_fallback.rs']]
for path in paths:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
(out/'build.json').write_text(json.dumps(r,indent=2)+'\n')

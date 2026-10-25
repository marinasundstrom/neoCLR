"""Pair qualified fresh HTTP controls and scalar-store candidates for interleaving."""
import argparse,hashlib,json
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--control',type=Path,required=True)
p.add_argument('--candidate',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
paths=[a.control.resolve()/'build.json',a.candidate.resolve()/'build.json']
control,candidate=[json.loads(path.read_text()) for path in paths]
assert control['passed'] and candidate['passed']
assert not control['scalarArrayStores'] and candidate['scalarArrayStores']
r=dict(passed=False,scope=__doc__,recipes=dict(control['recipes']),inputs={})
for mode in ['aot','jit']:r['recipes']['scalar-'+mode]=candidate['recipes']['live-'+mode]
for report in [control,candidate]:
 for path,digest in report['inputs'].items():
  assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest,('stale input',path)
  assert path not in r['inputs'] or r['inputs'][path]==digest,('different shared input',path)
  r['inputs'][path]=digest
# These generated sources and unchanged sync objects must match byte-for-byte.
for name in ['native-gc.c','text-arena.c','sync.o']:
 assert (a.control/name).read_bytes()==(a.candidate/name).read_bytes(),name
for path in [*paths,Path(__file__)]:r['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
r['passed']=True
(out/'build.json').write_text(json.dumps(r,indent=2)+'\n')

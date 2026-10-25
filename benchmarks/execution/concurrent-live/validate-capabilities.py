"""Check independent native-scope and borrowed-store feature bits in generated hosts."""
import argparse,hashlib,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False);base=Path(__file__).resolve().parent;root=base.parents[2]
r=dict(passed=False,commands=[],inputs={})
def run(cmd):
 cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=120)
 r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
try:
 recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
 for scope in [False,True]:
  source=out/f'host-{scope}.c';run(['python3',base/('native-scopes.py' if scope else 'generate.py'),'--output',source]);s=source.read_text()
  start=s.index('#define NEOCLR_LIVE_HOST_VERSION');end=s.index('\n}',s.index('uint64_t experiment_gc_live_capabilities_v1',start))+2
  for borrowed in [False,True]:
   c=out/f'query-{scope}-{borrowed}.c';expected=15+(16 if scope else 0)+(32 if borrowed else 0)
   c.write_text('#include <stdint.h>\n#include <assert.h>\n'+s[start:end]+f'\nint main(void) {{ assert(!experiment_gc_live_capabilities_v1(0)); assert(!experiment_gc_live_capabilities_v1(2)); assert(experiment_gc_live_capabilities_v1(1)=={expected}); }}\n')
   exe=c.with_suffix('');cmd=recipe[:recipe.index('-std=c11')]+['-std=c11','-O2','-Wall','-Wextra','-Werror',c,'-o',exe]
   if borrowed:cmd+=['-DNEOCLR_LIVE_STORE_HANDLE']
   run(cmd);run([exe])
  r['inputs'][str(source)]=hashlib.sha256(source.read_bytes()).hexdigest()
 for f in [base/'contract.h',base/'native-scopes.py',Path(__file__)]:r['inputs'][str(f)]=hashlib.sha256(f.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

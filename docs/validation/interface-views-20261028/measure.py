from pathlib import Path
import json,hashlib,subprocess,os
root=Path.cwd();w=root/'target/interface-views-20261028';out=w/'paired';out.mkdir(exist_ok=True)
b=dict(passed=True,scope='Paired 67e111ab1 baseline and immutable interface-view cache; identical retained Preview18 guest and native hosts from the preceding cohort.',recipes={},inputs={})
def pin(p):
 p=Path(p).resolve();b['inputs'][str(p)]=hashlib.sha256(p.read_bytes()).hexdigest()
for collector in ['sync','live']:
 selection=w/'hosts'/(collector+'-selection.json');pin(selection)
 for s in json.loads(selection.read_text()):
  p=Path(s)
  if p.is_absolute() and p.is_file():pin(p)
 for lazy in [False,True]:
  for variant in ['baseline','indexed']:
   runner=w/(variant+'-runner');pin(runner)
   env={'NEOCLR_LIVE_SELECTION_COMMAND':str(selection),'NEOCLR_LIVE_SELECTION_MODE':collector,'NEOCLR_JIT_LAZY':str(int(lazy))}
   if collector=='live':env['NEOCLR_LIVE_STACK_POLICY']='qualified'
   b['recipes'][variant+'-'+('lazy' if lazy else 'eager')+'-'+collector+'-jit']=dict(execution='jit',collector=collector,command=[str(runner),'mutation_store::tests::full_http_selection_compiles_with_live_gc','--exact','--ignored','--nocapture'],env=env)
for p in [w/'candidate-runtime_lookup.rs',w/'candidate-interfaces.rs',w/'candidate-metadata.rs',w/'hosts/owned-host.c',w/'hosts/native-gc.c',w/'hosts/text-arena.c',Path(__file__),root/'Cargo.lock']:pin(p)
(out/'build.json').write_text(json.dumps(b,indent=2)+'\n')
def run(c,name):
 print(name,flush=True)
 with open(w/(name+'.log'),'w') as f:r=subprocess.run(list(map(str,c)),stdout=f,stderr=subprocess.STDOUT)
 assert r.returncode==0,(name,r.returncode)
for variant in ['baseline','indexed']:run([w/(variant+'-runner'),'--list'],variant+'-loader')
base=['python3','benchmarks/gc-suite/matrix/http.py','--build',out,'--samples','json','--protocols','pipeline','--modes',*b['recipes']]
run([*base,'--output',w/'smoke','--rounds','1','--requests','32'],'smoke')
run([*base,'--output',w/'paired-http','--rounds','3','--requests','4096'],'paired-http')
run([*base,'--output',w/'paired-memory','--rounds','1','--requests','4096','--process-memory'],'paired-memory')
run(['python3','benchmarks/gc-suite/matrix/profile-lazy-startup.py','--build',out,'--output',w/'startup-profile','--modes','baseline-lazy-live-jit','indexed-lazy-live-jit','--kind','cpu'],'startup-profile')
(w/'completed.json').write_text('{"passed":true}\n')

"""Qualify exact private read-ahead sources with retained native GC toolchain recipes."""
import argparse,hashlib,json,os,subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--wire',action='store_true',help='Qualify real connection response persistence')
p.add_argument('--driver',action='store_true',help='Qualify persistent driver admission and shutdown')
a=p.parse_args();root=Path(__file__).resolve().parents[3];base=Path(__file__).resolve().parent
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,commands=[],runs=[],inputs={})
runner=root/'target/release/deps/neoclr_codegen-12f33f26d2ff4df0'
bundle=root/'target/uri-efficiency-bundle/bundle'
stem='Driver' if a.driver else 'Wire' if a.wire else 'ReadAhead'
a.wire = a.wire or a.driver
assembly=base/('bin/neoclr/'+stem+'Tests.dll')
expected='Session wire passed\n' if a.wire else 'Read-ahead: 45 windows passed\n'
def run(cmd,env=None,wire=False):
 cmd=list(map(str,cmd))
 if wire:
  from wire_probe import run_wire
  x=run_wire(cmd,root,env,driver=a.driver)
 else:x=subprocess.run(cmd,cwd=root,env=env,capture_output=True,text=True,timeout=300)
 row=dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr);r['commands'].append(row)
 x.check_returncode();return row
try:
 run(['dotnet',bundle/'sdk/tools/rvnc/rvnc.dll','neoclr','--project',base/(stem+'.rvnproj')],dict(os.environ,NeoClrBundleRoot=str(bundle)))
 selection=json.loads((root/'target/gc-patterns-standard-build/compute/live-command.json').read_text())
 context=selection[selection.index('--system'):selection.index('--compile-system')]
 trial=run([bundle/'bin/neoclr','run',assembly,'--execution','interpreter','--instructions','200000000',*context],wire=a.wire)
 assert expected in trial['stdout']
 r['runs'].append(dict(mode='interpreter',passed=True))
 for mode,report,suffix in [('sync','target/gc-suite-standard-v1/standalone/results.json','compute-sync'),('live','target/gc-index-only-paired/results.json','compute-live-budget-1024-16384-index-only-hybrid-inline')]:
  native=json.loads((root/report).read_text())
  recipe=next(c['command'] for c in native['commands'] if '-o' in c['command'] and c['command'][c['command'].index('-o')+1].endswith(suffix))
  selected_assembly=assembly
  if mode=='live':
   # Explicit GC replacement is not admitted by this concurrent test compiler.
   # Keep all assertions/reads; exercise automatic boundary pacing instead.
   source=(base/(stem+'.rvn')).read_text().replace('GC.Collect()', '/* automatic native pacing */')
   (out/(stem+'.rvn')).write_text(source)
   project=(base/(stem+'.rvnproj')).read_text().replace('../../../runtime/',str(root/'runtime')+'/')
   (out/(stem+'.rvnproj')).write_text(project)
   run(['dotnet',bundle/'sdk/tools/rvnc/rvnc.dll','neoclr','--project',out/(stem+'.rvnproj')],dict(os.environ,NeoClrBundleRoot=str(bundle)))
   selected_assembly=out/('bin/neoclr/'+stem+'Tests.dll')
  command=selection.copy();command[2]=str(selected_assembly);command[4]=str(out/(mode+'.o'))
  path=out/(mode+'-command.json');path.write_text(json.dumps(command,indent=2)+'\n')
  env=dict(os.environ,NEOCLR_TEST_OPT_LEVEL='speed',NEOCLR_LIVE_SELECTION_COMMAND=str(path),NEOCLR_LIVE_SELECTION_MODE=mode)
  for flag in ['NEOCLR_LIVE_ELIDE_STACK_ARGUMENTS','NEOCLR_LIVE_FORWARD_STACK_ARGUMENTS','NEOCLR_LIVE_ELIDE_FORWARDED_STACK_ARGUMENTS','NEOCLR_LIVE_SSA_BYREF_ARGUMENTS']:
   env.pop(flag,None)
   if mode=='live':env[flag]='1'
  launch=[runner,'mutation_store::tests::full_http_selection_compiles_with_live_gc','--exact','--ignored','--nocapture']
  run(launch,env)
  link=[str(out/(mode+'.o')) if x.endswith('/compute/'+mode+'.o') else x for x in recipe]
  link[link.index('-o')+1]=str(out/mode)
  if mode=='live':
   link=[x for x in link if not x.startswith(('-DNEOCLR_GC_ALLOCATION_BUDGET=','-DNEOCLR_GC_PUBLICATION_BUDGET='))]
   link+=['-DNEOCLR_GC_ALLOCATION_BUDGET=16','-DNEOCLR_GC_PUBLICATION_BUDGET=64']
  run(link);trial=run([out/mode],wire=a.wire);assert expected in trial['stdout']
  if mode=='live':
   counts=list(map(int,next(x.split()[1:] for x in trial['stderr'].splitlines() if x.startswith('CANDIDATE '))))
   assert counts[1]>0 and counts[5]==0
  r['runs'].append(dict(mode=mode+'-aot',passed=True,collectionPolicy='forced' if mode=='sync' else 'automatic 16/64',stderr=trial['stderr']))
  shared=[x for x in link if not x.endswith('/main.c') and x!=str(out/(mode+'.o'))]+['-dynamiclib']
  shared[shared.index('-o')+1]=str(out/(mode+'.dylib'));run(shared)
  jit=[command[0],'--jit-managed',*command[1:4],str(out/(mode+'.dylib')),*command[5:]]
  path.write_text(json.dumps(jit,indent=2)+'\n')
  trial=run(launch,env,wire=a.wire);assert expected in trial['stdout']
  result=json.loads(next(x[len('JIT_RESULT '):] for x in trial['stderr'].splitlines() if x.startswith('JIT_RESULT ')))
  assert result['guestExitCode']==0
  if mode=='live':
   counts=list(map(int,next(x.split()[1:] for x in trial['stderr'].splitlines() if x.startswith('CANDIDATE '))))
   assert counts[1]>0 and counts[5]==0
  r['runs'].append(dict(mode=mode+'-jit',passed=True,collectionPolicy='forced' if mode=='sync' else 'automatic 16/64',stderr=trial['stderr']))
 for path in [Path(__file__),base/'wire_probe.py',base/(stem+'.rvn'),base/(stem+'.rvnproj'),root/'runtime/raven/src/System/Web/Http/HttpBodyInput.rvn',root/'runtime/raven/src/System/Web/Http/HttpWire.rvn',root/'runtime/raven/src/System/Web/Http/HttpWireLimits.rvn',root/'runtime/raven/src/System/Web/Http/HttpReadAhead.rvn',root/'runtime/raven/src/System/Web/Http/HttpConnection.rvn',assembly,runner]:
  r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 for path in (root/'runtime/raven/src/System/Web').rglob('*.rvn'):r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 for path in out.glob('*.rvn*'):r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

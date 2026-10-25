"""Build sustained versions of existing samples with matched sync/live selection.

Preserve each sample body and its console work; only rename Main to RunSample and
add an async batching entry. This is compilation/admission evidence, not timing.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
from xml.sax.saxutils import escape

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--suite',choices=['existing','allocation-patterns'],default='existing')
p.add_argument('--stable-byrefs',action='store_true')
p.add_argument('--runner',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--iterations',type=int,default=128)
p.add_argument('--only',nargs='+')
a=p.parse_args();assert a.iterations>0
root=Path(__file__).resolve().parents[3];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
reference_path=root/'target/gc-mode-comparison-final/results.json'
reference=json.loads(reference_path.read_text())
prior={c['sample']:c for c in reference['cases']}
recipe=next(c['command'].copy() for c in prior['tasks']['runs'] if c.get('mode')=='jit' and not c.get('diagnostic'))
source_recipe=next(c['command'].copy() for c in json.loads((root/'target/primitive-comparison-build/build.json').read_text())['commands'] if c.get('phase')=='source-compilation')
bundle=Path(source_recipe[1]).parents[3]
samples=root/'docs/experiments/raven-target/samples'
cases=[
 ('tasks',[samples/'library-tasks.rvn'],prior['tasks']['expected']),
 ('collections',[samples/'application-order-collections.rvn'],prior['collections']['expected']),
 ('arrays',[samples/'library-array-tour.rvn'],(samples/'library-array-tour.expected.txt').read_text()),
 ('strings',[samples/'library-grapheme-strings.rvn'],(samples/'library-grapheme-strings.expected.txt').read_text()),
 ('reflection',[samples/'library-reflection.rvn'],(samples/'library-reflection.expected.txt').read_text()),
 ('async',[samples/'library-async.rvn'],'Suspended\n42\n'),
 ('cancellation',[samples/'library-async-cancellation.rvn'],'Cancelled\n'),
 ('json',[root/'docs/experiments/json-object-mapping/Main.rvn',root/'docs/experiments/json-object-mapping/Mapping.rvn'],'JSON object mapping checks passed\n'),
]
if a.suite=='allocation-patterns':
 manifest=json.loads((root/'benchmarks/gc-suite/samples.json').read_text())
 cases=[(c['name'],[root/'benchmarks/gc-suite'/c['source']],c['expected']) for c in manifest['cases']]
if a.only:
 assert set(a.only)<=set(c[0] for c in cases)
 cases=[c for c in cases if c[0] in a.only]
r={'passed':False,'suite':a.suite,'stableByrefs':a.stable_byrefs,'scope':__doc__,'iterations':a.iterations,'cases':[],'inputs':{}}
def record(path):
 path=Path(path)
 if path.is_file():r['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
def run(command,env):
 command=list(map(str,command));x=subprocess.run(command,cwd=root,env=env,capture_output=True,text=True,timeout=300)
 return {'command':command,'exitCode':x.returncode,'stdout':x.stdout,'stderr':x.stderr}
r['optimizationLevel']=os.environ.get('NEOCLR_TEST_OPT_LEVEL','none')
try:
 for name,sources,expected in cases:
  directory=out/name;directory.mkdir();copied=[]
  row={'sample':name,'expectedPerIteration':expected,'source':None,'native':[]};r['cases'].append(row)
  wrapped=0
  for source in sources:
   record(source);text=source.read_text()
   if 'func Main()' in text:
    assert text.count('func Main()')==1;wrapped+=1
    asynchronous='async func Main()' in text
    text=text.replace('func Main()', 'func RunSample()',1)
    invocation=('await ' if asynchronous else '')+'RunSample()'
    text+='\nasync func Main() -> System.Concurrency.Tasks.Task<()> {\n    var iteration = 0\n    while iteration < '+str(a.iterations)+' {\n        '+invocation+'\n        iteration += 1\n    }\n}\n'
   path=directory/source.name;path.write_text(text);record(path);copied.append(path)
  assert wrapped==1
  project=directory/'App.rvnproj'
  includes=''.join('<Compile Include="'+escape(str(path))+'"/>' for path in copied)
  project.write_text('<Project Sdk="Microsoft.NET.Sdk"><Import Project="'+escape(str(bundle/'lib/NeoCLR.ClassLibrary.props'))+'"/><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><AssemblyName>App</AssemblyName><EnableDefaultCompileItems>false</EnableDefaultCompileItems></PropertyGroup><ItemGroup>'+includes+'</ItemGroup></Project>')
  record(project)
  command=source_recipe.copy();command[-1]=str(project)
  row['source']=run(command,dict(os.environ,NeoClrBundleRoot=str(bundle)))
  if row['source']['exitCode']!=0:continue
  assembly=directory/'bin/neoclr/App.dll';assert assembly.is_file();record(assembly)
  for mode in ['sync','live']:
   command=recipe.copy();command.pop(1);command[2]=str(assembly);command[4]=str(directory/(mode+'.o'))
   path=directory/(mode+'-command.json');path.write_text(json.dumps(command,indent=2)+'\n');record(path)
   env=dict(os.environ,NEOCLR_LIVE_SELECTION_COMMAND=str(path),NEOCLR_LIVE_SELECTION_MODE=mode)
   for key in ['NEOCLR_LIVE_ELIDE_STACK_ARGUMENTS','NEOCLR_LIVE_FORWARD_STACK_ARGUMENTS','NEOCLR_LIVE_ELIDE_FORWARDED_STACK_ARGUMENTS','NEOCLR_LIVE_SSA_BYREF_ARGUMENTS']:
    env.pop(key,None)
    if mode=='live' and a.stable_byrefs:env[key]='1'

   invocation=[a.runner.resolve(),'mutation_store::tests::full_http_selection_compiles_with_live_gc','--exact','--ignored','--nocapture']
   result=run(invocation,env);result.update(mode=mode,selectionCommand=command,environment={k:env[k] for k in ['NEOCLR_TEST_OPT_LEVEL','NEOCLR_LIVE_SELECTION_COMMAND','NEOCLR_LIVE_SELECTION_MODE','NEOCLR_LIVE_ELIDE_STACK_ARGUMENTS','NEOCLR_LIVE_FORWARD_STACK_ARGUMENTS','NEOCLR_LIVE_ELIDE_FORWARDED_STACK_ARGUMENTS','NEOCLR_LIVE_SSA_BYREF_ARGUMENTS'] if k in env});row['native'].append(result)
   for item in command:record(item)
  (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
 for path in [reference_path,source_recipe[1],a.runner,Path(__file__),bundle/'manifest.json']:record(path)
 r['passed']=all(c['source']['exitCode']==0 and len(c['native'])==2 and all(n['exitCode']==0 for n in c['native']) for c in r['cases'])
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
raise SystemExit(0 if r['passed'] else 1)

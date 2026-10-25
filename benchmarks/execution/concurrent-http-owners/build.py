"""Qualify live GC with isolated HTTP application owners, not shared managed heaps."""
import argparse,gzip,json,os,subprocess,hashlib
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);p.add_argument('--runner',type=Path,required=True);p.add_argument('--sync-object',type=Path);p.add_argument('--sample',choices=['task','plain'],default='task');a=p.parse_args()
root=Path(__file__).resolve().parents[3];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r={'passed':False,'sample':a.sample,'commands':[],'inputs':{}}
def record(path):
 path=Path(path)
 if path.is_file():r['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
def run(cmd,env=None):
 cmd=list(map(str,cmd));x=subprocess.run(cmd,env=env,capture_output=True,text=True,timeout=300)
 r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
 for item in cmd:record(item)
try:
 fixture=root/'docs/experiments/http-worker-integration/build-summary.json';record(fixture)
 build=json.loads(fixture.read_text());selection=next(c['command'].copy() for c in build['commands'] if c.get('phase')=='native-compilation')
 recipe_file=root/'benchmarks/execution/concurrent-workloads/scalar-frame-http-plain.json.gz';record(recipe_file)
 recipes=json.loads(gzip.decompress(recipe_file.read_bytes()))
 if a.sample=='plain':
  from xml.sax.saxutils import escape
  source=root/'benchmarks/execution/concurrent-http-owners/Plain.rvn';record(source)
  bundle=Path(selection[selection.index('--system')+1]).parent.parent
  project=out/'Plain.rvnproj'
  project.write_text('<Project Sdk="Microsoft.NET.Sdk"><Import Project="'+escape(str(bundle/'lib/NeoCLR.ClassLibrary.props'))+'"/><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><AssemblyName>Plain</AssemblyName><EnableDefaultCompileItems>false</EnableDefaultCompileItems></PropertyGroup><ItemGroup><Compile Include="'+escape(str(source))+'"/></ItemGroup></Project>')
  run(['dotnet',bundle/'sdk/tools/rvnc/rvnc.dll','neoclr','--project',project],dict(os.environ,NeoClrBundleRoot=str(bundle)))
  selection[2]=str(out/'bin/neoclr/Plain.dll');assert Path(selection[2]).is_file()
 compiler_file=root/'benchmarks/execution/concurrent-workloads/forward-elision-http-build.json.gz';record(compiler_file)
 compiler=json.loads(gzip.decompress(compiler_file.read_bytes()))['commands'][0]['command']
 if a.runner:compiler[0]=str(a.runner.resolve())
 for mode in ['sync','live']:
  command=selection.copy();command[4]=str(out/(mode+'.o'))
  path=out/(mode+'-selection.json');path.write_text(json.dumps(command))
  if mode=='sync' and a.sync_object:
   assert a.sample=='task'
   import shutil
   record(a.sync_object);shutil.copy2(a.sync_object,out/'sync.o')
   r['reusedSyncObject']=str(a.sync_object.resolve())
  else:run(compiler,dict(os.environ,NEOCLR_TEST_OPT_LEVEL='speed',NEOCLR_LIVE_SELECTION_MODE=mode,NEOCLR_LIVE_SELECTION_COMMAND=str(path)))
  for item in command:record(item)
 run(['python3',root/'benchmarks/execution/concurrent-live/native-scopes.py','--output',out/'native-gc.c'])
 run(['python3',root/'benchmarks/execution/concurrent-store/native-providers.py','--output',out/'text-arena.c'])
 source=root/'target/concurrent-workloads/scalar-frame-http-plain/owned-host.c';record(source)
 text=source.read_text();needle='static void report_measurement(void) {';assert text.count(needle)==1
 text=text.replace(needle,needle+'\n flockfile(stderr);')
 assert text.endswith('}\n');text=text[:-2]+' funlockfile(stderr);\n}\n';(out/'owned-host.c').write_text(text)
 main=root/'benchmarks/execution/concurrent-http-owners/main.c'
 for mode,recipe_name in [('sync','sync'),('live','live-inline-index')]:
  original=next(c['command'] for c in recipes['commands'] if '-o' in c['command'] and Path(c['command'][c['command'].index('-o')+1]).name==recipe_name)
  for workers in [1,4]:
   cmd=[]
   for item in original:
    if item.endswith('/owned-host.c'):item=str(out/'owned-host.c')
    elif mode=='live' and item.endswith('/native-gc.c'):item=str(out/'native-gc.c')
    elif mode=='live' and item.endswith('/text-arena.c'):item=str(out/'text-arena.c')
    elif item.endswith('/main.c'):item=str(main)
    elif item.endswith('.o'):item=str(out/(mode+'.o'))
    cmd.append(item)
   cmd[cmd.index('-o')+1]=str(out/f'{mode}-{workers}')
   cmd+=['-DOWNERS='+str(workers),'-DNEOCLR_HTTP_WORKERS='+str(workers),'-I',str(root/'runtime/native/workers')]
   run(cmd)
 for path in (root/'runtime/native').rglob('*'):
  if path.suffix in ['.c','.h','.inc']:record(path)
 for path in [Path(__file__),main,root/'runtime/codegen/src/values.rs',root/'runtime/codegen/src/mutation_admission.rs']:record(path)
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

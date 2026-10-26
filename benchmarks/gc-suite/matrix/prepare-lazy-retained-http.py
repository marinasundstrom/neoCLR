"""Build a retained-object HTTP workload against pinned lazy-JIT/native controls.

A rolling window holds 4096 Person objects. Each response replaces one slot;
shutdown verifies every survivor's expected ID and the ring cursor.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import runpy

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--build',type=Path,required=True)
p.add_argument('--compiler',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--workload',choices=['retained','complex','service'],default='retained')
a=p.parse_args();root=Path(__file__).resolve().parents[3]
source=a.build.resolve()/'build.json';b=json.loads(source.read_text());assert b['passed']
for path,digest in b['inputs'].items():
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest,path
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,scope=__doc__,workload=a.workload+'-http-4096',recipes={},inputs=dict(b['inputs']),commands=[])
try:
    recipe=b['recipes']['arena-control-sync-jit']
    original=json.loads(Path(recipe['env']['NEOCLR_LIVE_SELECTION_COMMAND']).read_text())
    library=Path(original[original.index('--system')+1]).parent
    sample=root/('benchmarks/execution/lazy-jit/'+a.workload+'-http/Benchmark.rvn')
    model=root/'benchmarks/http-server/Model.rvn'
    web=root/'runtime/raven/src/System/Web'
    project=out/'Retained.rvnproj'
    project.write_text(f'''<Project Sdk="Microsoft.NET.Sdk">
  <Import Project="{library}/NeoCLR.ClassLibrary.props" />
  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework>
    <OutputType>Exe</OutputType>
    <AssemblyName>RetainedBenchmarkTests</AssemblyName>
    <RavenNeoClrRuntimeAsyncHelper></RavenNeoClrRuntimeAsyncHelper>
    <EnableDefaultCompileItems>false</EnableDefaultCompileItems>
  </PropertyGroup>
  <ItemGroup>
    <Reference Remove="System.Web" />
    <Compile Include="{sample}" />
    <Compile Include="{model}" />
    <Compile Include="{library}/RuntimeAwait.rvn" />
    <Compile Include="{web}/**/*.rvn" />
  </ItemGroup>
</Project>\n''')
    command=['dotnet',str(a.compiler.resolve()),'neoclr','--project',str(project)]
    x=subprocess.run(command,capture_output=True,text=True,timeout=180)
    r['commands'].append(dict(command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
    app=out/'bin/neoclr/RetainedBenchmarkTests.dll';reflection=app.with_suffix('.reflection.json')
    assert app.is_file()
    def run(command):
        command=list(map(str,command))
        result=subprocess.run(command,capture_output=True,text=True,timeout=180)
        r['commands'].append(dict(command=command,exitCode=result.returncode,stdout=result.stdout,stderr=result.stderr))
        result.check_returncode()
    helper=library.parent/'tools/run-native-sample.py'
    runtime=library.parent/'bin/neoclr'
    runpy.run_path(str(helper))['reflection_roots'](runtime,app,['Person'],run)
    for path in [helper,runtime]:
        r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
    assert reflection.is_file()
    for mode in ['arena-eager-sync-jit','arena-control-sync-jit','arena-eager-live-jit','arena-control-live-jit']:
        old=b['recipes'][mode];selection=json.loads(Path(old['env']['NEOCLR_LIVE_SELECTION_COMMAND']).read_text())
        selection[selection.index('--closed-world')+1]=str(app)
        selection[selection.index('--reflection-roots')+1]=str(reflection)
        path=out/(mode+'.json');path.write_text(json.dumps(selection,indent=2)+'\n')
        r['recipes'][mode]=dict(old,env=dict(old['env'],NEOCLR_LIVE_SELECTION_COMMAND=str(path)))
        r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
    for path in [source,sample,model,project,app,reflection,library/'RuntimeAwait.rvn',library/'NeoCLR.ClassLibrary.props',a.compiler.resolve(),Path(__file__).resolve(),*web.rglob('*.rvn')]:
        r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
    r['passed']=True
finally:(out/'build.json').write_text(json.dumps(r,indent=2)+'\n')

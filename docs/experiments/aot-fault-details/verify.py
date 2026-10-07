#!/usr/bin/env python3
"""Build Raven System.Fail and compare interpreter/native message, frames and exit."""
import argparse, hashlib, json, os, re, shutil, subprocess, tempfile
from pathlib import Path
ROOT = Path(__file__).resolve().parents[3]
p = argparse.ArgumentParser(description=__doc__)
for key in ('compiler', 'runtime', 'aot', 'bundle', 'output'):
    p.add_argument('--' + key, type=Path, required=True)
a = p.parse_args()
compiler, runtime, aot, bundle, output = (getattr(a,k).resolve() for k in ('compiler','runtime','aot','bundle','output'))
output.mkdir(parents=True, exist_ok=False)
base = ROOT / 'docs/experiments/aot-fault-details'
source, host, render = (base/n for n in ('fail.rvn','host.c','render.c'))
core, seed, library, ownership = (bundle/'lib'/n for n in ('Core.dll','System.runtime.neox','System.Runtime.dll','ownership.json'))
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
report = dict(profile='aot-fault-details-v1', baseRevision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(), SDKROOT=os.environ.get('SDKROOT'),
    inputs={str(p):sha(p) for p in (compiler,runtime,aot,core,seed,library,ownership,source,host,render,base/'fault-details.h')},commands=[])
def save(): (output/'validation.json').write_text(json.dumps(report,indent=2)+'\n')
def run(args, expected=0):
    r=subprocess.run(list(map(str,args)),cwd=ROOT,capture_output=True,text=True,timeout=120)
    report['commands'].append(dict(command=r.args,exit=r.returncode,stdout=r.stdout if len(r.stdout)<4000 else dict(sha256=hashlib.sha256(r.stdout.encode()).hexdigest(),bytes=len(r.stdout)),stderr=r.stderr))
    save(); assert r.returncode==expected,r.stderr
    return r
assembly, obj, binary = (output/n for n in ('Failure.dll','failure.o','failure'))
run(['dotnet',compiler,'neoclr','--core-reference',core,'--runtime-seed',seed,'--reference',library,'--bootstrap-intrinsics','--bootstrap-ownership',ownership,'--object-library','System.Runtime','-o',assembly,source])
context=['--system',seed,'--module',library,'--object-root',library]
interpreted=run([runtime,'run',assembly]+context,expected=1)
assert interpreted.stdout=='' and '[code=UserFault]' in interpreted.stderr
frames=re.findall(r'^  at ([^(]+)\(.* IL instruction (\d+)',interpreted.stderr,re.M)
assert len(frames)==5,interpreted.stderr
context+=['--compile-system','--bind-user-fault']
inspection=json.loads(run([aot,'--inspect',assembly,'@entry','--closed-world']+context).stdout)
assert inspection['admission']['accepted'] is True
selection=json.loads(run([aot,'--closed-world',assembly,'@entry',obj]+context).stdout)
assert selection==inspection['selection']
report['selection']={k:selection[k] for k in ('loadSet','functions','nativeBindings')}
run(['clang','-arch','arm64','-std=c11','-Wall','-Wextra','-Werror',host,render,obj,'-o',binary])
assert run(['nm','-u',obj]).stdout==''
deps=[line.split()[0] for line in run(['otool','-L',binary]).stdout.splitlines()[1:]]
assert deps==['/usr/lib/libSystem.B.dylib']
with tempfile.TemporaryDirectory() as d:
    installed=Path(d)/'failure';shutil.copy2(binary,installed)
    r=subprocess.run([installed],cwd=d,env={},capture_output=True,text=True,timeout=10)
    assert r.returncode==1 and r.stdout==''
    assert r.stderr.startswith('UserFault: The operation failed: världen 🌍\n'),r.stderr
    native_frames=re.findall(r'^   at (.+) \[instruction (\d+)\]$',r.stderr,re.M)
    assert native_frames==frames,(native_frames,frames)
    report['native']=dict(exit=r.returncode,stdout=r.stdout,stderr=r.stderr,framesMatchInterpreter=True,messagePreserved=True,executableOnlyDirectory=True,emptyEnvironment=True,dynamicDependencies=deps)
report['artifacts']={p.name:sha(p) for p in (assembly,obj,binary)}
save();print('Passed: Raven System.Fail preserves message/frames, exits 1 and runs standalone.')

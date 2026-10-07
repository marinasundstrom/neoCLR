#!/usr/bin/env python3
"""Validate ordinary Raven Console.ReadByte with linked standalone native input."""
import argparse, hashlib, json, os, shutil, subprocess, tempfile
from pathlib import Path
ROOT = Path(__file__).resolve().parents[3]
p = argparse.ArgumentParser(description=__doc__)
for key in ('runtime', 'aot', 'bundle', 'output'):
    p.add_argument('--' + key, type=Path, required=True)
a = p.parse_args()
runtime, aot, bundle, output = (getattr(a,k).resolve() for k in ('runtime','aot','bundle','output'))
output.mkdir(parents=True, exist_ok=False)
base = ROOT/'docs/experiments/aot-console'
faults = base.parent/'aot-fault-details'
assembly = base.parent/'aot-input/ReadByte.pe'
seed, library = (bundle/'lib'/n for n in ('System.runtime.neox','System.Runtime.dll'))
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
report = dict(profile='aot-console-read-byte-v1', baseRevision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(), SDKROOT=os.environ.get('SDKROOT'), inputs={str(p):sha(p) for p in (runtime,aot,seed,library,assembly,assembly.with_name('read-byte.rvn'),base/'console.c',base/'console.h',faults/'host.c',faults/'render.c',faults/'fault-details.h')},commands=[])
def save(): (output/'validation.json').write_text(json.dumps(report,indent=2)+'\n')
def run(args, expected=0, **kwargs):
    r = subprocess.run(list(map(str,args)),cwd=ROOT,capture_output=True,timeout=120,**kwargs)
    report['commands'].append(dict(command=r.args,exit=r.returncode,stdout=r.stdout.decode() if len(r.stdout)<4000 else dict(sha256=hashlib.sha256(r.stdout).hexdigest(),bytes=len(r.stdout)),stderr=r.stderr.decode()))
    save(); assert r.returncode==expected,r.stderr
    return r
context=['--system',seed,'--module',library,'--object-root',library]
for data, expected in ((b'*',0),(b'A',2),(b'\x00',2),(b'\xff',2),(b'',0)):
    r=run([runtime,'run',assembly]+context,expected=expected,input=data)
    assert not r.stdout and not r.stderr
context+=['--compile-system','--bind-user-fault','--bind-console-read-byte']
inspection=json.loads(run([aot,'--inspect',assembly,'@entry','--closed-world']+context).stdout)
assert inspection['admission']['accepted'] is True,inspection['admission']
obj,binary=output/'input.o',output/'input'
selection=json.loads(run([aot,'--closed-world',assembly,'@entry',obj]+context).stdout)
assert selection==inspection['selection']
report['selection']={k:selection[k] for k in ('loadSet','functions','nativeBindings')}
def link(adapter, target):
    run(['clang','-arch','arm64','-std=c11','-Wall','-Wextra','-Werror','-I',base,faults/'host.c',faults/'render.c',adapter,obj,'-o',target])
link(base/'console.c',binary)
assert run(['nm','-u',obj]).stdout.decode().strip()=='_neoclr_console_read_byte_v1'
deps=[line.split()[0] for line in run(['otool','-L',binary]).stdout.decode().splitlines()[1:]]
assert deps==['/usr/lib/libSystem.B.dylib']
with tempfile.TemporaryDirectory() as d:
    installed=Path(d)/'input';shutil.copy2(binary,installed)
    outcomes=[]
    for data,expected in ((b'*',0),(b'A',2),(b'\x00',2),(b'\xff',2),(b'',0)):
        r=subprocess.run([installed],cwd=d,env={},input=data,capture_output=True,timeout=10)
        assert r.returncode==expected and not r.stdout and not r.stderr
        outcomes.append(dict(inputHex=data.hex(),exit=r.returncode))
    # A closed descriptor causes a real stdio read failure, not EOF.
    r=subprocess.run([installed],cwd=d,env={},preexec_fn=lambda: os.close(0),capture_output=True,timeout=10)
    assert r.returncode==1 and not r.stdout and not r.stderr
report['native']=dict(outcomes=outcomes,closedInputExit=1,executableOnlyDirectory=True,emptyEnvironment=True,dynamicDependencies=deps)
for value,expected in ((-2,1),(-3,1),(256,1),(-4,1)):
    stub=output/'stub.c';stub.write_text('#include "console.h"\nint32_t neoclr_console_read_byte_v1(void) { return %d; }\n'%value)
    target=output/('stub'+str(value));link(stub,target)
    r=run([target],expected=expected)
    assert not r.stdout
    if value in (-2,-3): assert not r.stderr
    else:
        assert r.stderr.startswith(b'UserFault: invalid native I/O status\n')
        wrapper = next(f['name'] for f in selection['functions'] if f['name'].endswith('.M_5265616442797465'))
        assert wrapper.encode() in r.stderr and b'neoCLR.Runtime.ConsoleReadByte' not in r.stderr
report['artifacts']={p.name:sha(p) for p in (obj,binary)}
save();print('Passed: Raven ReadByte byte/EOF/error outcomes, invalid service faults, standalone deployment.')

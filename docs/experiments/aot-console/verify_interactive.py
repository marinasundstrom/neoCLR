#!/usr/bin/env python3
"""Compile a Raven input/output app and compare interpreter/native Console behavior."""
import argparse, hashlib, json, os, shutil, subprocess, tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
p=argparse.ArgumentParser(description=__doc__)
for key in ('compiler','runtime','aot','bundle','output'): p.add_argument('--'+key,type=Path,required=True)
mode=p.add_mutually_exclusive_group()
mode.add_argument('--numeric',action='store_true',help='Validate invocation-owned numeric text and ABI v4')
mode.add_argument('--text-values',action='store_true',help='Validate copied records and Some<string>')
mode.add_argument('--references',action='store_true',help='Validate bounded reference aliases and Console output')
mode.add_argument('--arrays',action='store_true',help='Validate packed byte arrays and Console output')
mode.add_argument('--characters',action='store_true',help='Validate UTF-8 character output with statically linked segmentation')
mode.add_argument('--small-integers',action='store_true',help='Validate signed and unsigned narrow integer Console output')
mode.add_argument('--wide-integers',action='store_true',help='Validate 64-bit and native-width integer Console output')
a=p.parse_args()
compiler,runtime,aot,bundle,output=(getattr(a,k).resolve() for k in ('compiler','runtime','aot','bundle','output'))
output.mkdir(parents=True,exist_ok=False)
base=ROOT/'docs/experiments/aot-console';faults=base.parent/'aot-fault-details'
source=base/('wide-integers.rvn' if a.wide_integers else 'small-integers.rvn' if a.small_integers else 'characters.rvn' if a.characters else 'bytes.rvn' if a.arrays else 'reference-cell.rvn' if a.references else 'numbers.rvn' if a.numeric else 'text-values.rvn' if a.text_values else 'interactive.rvn')
uses_arena=a.numeric or a.references or a.arrays or a.small_integers or a.wide_integers
host=base/'text-host.c' if uses_arena else faults/'host.c'
core,seed,library,ownership=(bundle/'lib'/n for n in ('Core.dll','System.runtime.neox','System.Runtime.dll','ownership.json'))
sha=lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
report=dict(profile='aot-console-wide-integers-v1' if a.wide_integers else 'aot-console-small-integers-v1' if a.small_integers else 'aot-console-characters-v1' if a.characters else 'aot-console-byte-arrays-v1' if a.arrays else 'aot-console-references-v1' if a.references else 'aot-console-numeric-v1' if a.numeric else 'aot-console-text-values-v1' if a.text_values else 'aot-console-interactive-v1',baseRevision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),SDKROOT=os.environ.get('SDKROOT'),inputs={str(p):sha(p) for p in (compiler,runtime,aot,core,seed,library,ownership,source,base/'console.c',base.parent/'aot-scalar/console.c',host,faults/'render.c',base/'text-arena.h',base/'text-arena.c')},commands=[])
def save(): (output/'validation.json').write_text(json.dumps(report,indent=2)+'\n')
def run(args,expected=0,**kwargs):
    r=subprocess.run(list(map(str,args)),cwd=ROOT,capture_output=True,timeout=120,**kwargs)
    report['commands'].append(dict(command=r.args,exit=r.returncode,stdout=r.stdout.decode() if len(r.stdout)<4000 else dict(sha256=hashlib.sha256(r.stdout).hexdigest(),bytes=len(r.stdout)),stderr=r.stderr.decode()))
    save();assert r.returncode==expected,r.stderr
    return r
assembly,obj,binary=(output/n for n in ('Interactive.dll','interactive.o','interactive'))
run(['dotnet',compiler,'neoclr','--core-reference',core,'--runtime-seed',seed,'--reference',library,'--bootstrap-intrinsics','--bootstrap-ownership',ownership,'--object-library','System.Runtime','-o',assembly,source])
context=['--system',seed,'--module',library,'--object-root',library]
cases=[(b'*',0),(b'A',0 if a.numeric else 2),(b'',0)]
if a.text_values or a.references or a.arrays or a.characters or a.small_integers or a.wide_integers: cases=[(b'',0)]
if a.numeric: cases += [(b'\x00',0),(b'\xff',0)]
interpreted=[run([runtime,'run',assembly]+context,expected=code,input=data) for data,code in cases]
read_end,write_end=os.pipe();os.close(read_end)
failed=run([runtime,'run',assembly]+context,expected=1,input=b'*',pass_fds=(write_end,),preexec_fn=lambda:os.dup2(write_end,1))
assert failed.stderr.startswith(b'RuntimeError: Runtime error\n')
flags=context+['--compile-system','--bind-user-fault','--bind-console-read-byte','--bind-console-write-line']
if a.wide_integers: flags += ['--bind-integer-text']
if a.characters: flags += ['--bind-character-text']
if uses_arena: flags += ['--bind-int32-to-string']
if a.references or a.arrays: flags += ['--reference-arena']
inspection=json.loads(run([aot,'--inspect',assembly,'@entry','--closed-world']+flags).stdout)
assert inspection['admission']['accepted'] is True,inspection['admission']
selection=json.loads(run([aot,'--closed-world',assembly,'@entry',obj]+flags).stdout)
assert inspection['selection']==selection
report['selection']={k:selection[k] for k in ('loadSet','functions','nativeBindings')}
native_text=[]
if a.characters:
    manifest=ROOT/'tools/aot-native-text/Cargo.toml'
    run(['cargo','build','--locked','--release','--manifest-path',manifest])
    native_text=[manifest.parent/'target/release/libneoclr_aot_native_text.a']
    for path in [manifest,manifest.parent/'Cargo.lock',manifest.parent/'src/lib.rs',*native_text]: report['inputs'][str(path)]=sha(path)
run(['clang','-arch','arm64','-std=c11','-Wall','-Wextra','-Werror',host,faults/'render.c',base/'console.c',base.parent/'aot-scalar/console.c',*([base/'text-arena.c'] if uses_arena else []),obj,*native_text,'-o',binary])
imports={'_neoclr_console_read_byte_v1','_neoclr_console_write_line_utf8_v1'}
if a.text_values or a.references or a.arrays or a.characters or a.small_integers or a.wide_integers: imports.remove('_neoclr_console_read_byte_v1')
if a.characters: imports.add('_neoclr_is_single_grapheme_v1')
if uses_arena and not a.wide_integers: imports.add('_neoclr_int32_to_string_v1')
if a.wide_integers: imports.update(['_neoclr_int64_to_string_v1', '_neoclr_uint64_to_string_v1'])
if a.references: imports.add('_neoclr_allocate_object_v1')
if a.arrays: imports.add('_neoclr_allocate_bytes_v1')
assert set(run(['nm','-u',obj]).stdout.decode().split())==imports
deps=[line.split()[0] for line in run(['otool','-L',binary]).stdout.decode().splitlines()[1:]]
assert deps==['/usr/lib/libSystem.B.dylib']
with tempfile.TemporaryDirectory() as d:
    installed=Path(d)/'interactive';shutil.copy2(binary,installed)
    outcomes=[]
    for (data,code),reference in zip(cases,interpreted):
        r=subprocess.run([installed],cwd=d,env={},input=data,capture_output=True,timeout=10)
        assert (r.returncode,r.stdout,r.stderr)==(code,reference.stdout,reference.stderr)
        outcomes.append(dict(inputHex=data.hex(),exit=code,stdout=r.stdout.decode()))
    r=subprocess.run([installed],cwd=d,env={},input=b'*',pass_fds=(write_end,),preexec_fn=lambda:os.dup2(write_end,1),capture_output=True,timeout=10)
    assert r.returncode==1 and r.stderr==failed.stderr and not r.stdout,(r,failed)
    report['native']=dict(outcomes=outcomes,outputFailureMatchesInterpreter=True,outputFault=r.stderr.decode(),executableOnlyDirectory=True,emptyEnvironment=True,dynamicDependencies=deps)
os.close(write_end)
report['artifacts']={p.name:sha(p) for p in (assembly,obj,binary)}
save();print('Passed:',report['profile'],'standalone output and exact fault parity')

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
mode.add_argument('--multi-values',action='store_true',help='Validate several Option/Result shapes alongside Console input')
mode.add_argument('--stream-views',action='store_true',help='Validate ordinary Console standard stream construction and interface views')
mode.add_argument('--input-stream',action='store_true',help='Validate standard input interface calls and stream outcomes')
p.add_argument('--reuse-compilation',type=Path,help='Reuse a successful compiler command from prior validation evidence with matching producer/source hashes')
mode.add_argument('--output-stream',action='store_true',help='Validate standard output/error Write/Flush interface calls')
mode.add_argument('--collections',action='store_true',help='Validate byte ArrayList growth and inherited Sequence views')
mode.add_argument('--utf8-text',action='store_true',help='Validate String instance byte counts and scalar slices')
mode.add_argument('--boxed-cases',action='store_true',help='Validate boxed empty library union cases')
mode.add_argument('--text-writer',action='store_true',help='Validate ordinary Console.Write and standard-error text writer')
mode.add_argument('--text-reader',action='store_true',help='Validate Console.ReadLine and UTF-8 input')
a=p.parse_args()
compiler,runtime,aot,bundle,output=(getattr(a,k).resolve() for k in ('compiler','runtime','aot','bundle','output'))
output.mkdir(parents=True,exist_ok=False)
base=ROOT/'docs/experiments/aot-console';faults=base.parent/'aot-fault-details'
source=base/('text-reader.rvn' if a.text_reader else 'text-writer.rvn' if a.text_writer else 'boxed-cases.rvn' if a.boxed_cases else 'utf8-text.rvn' if a.utf8_text else 'collections.rvn' if a.collections else 'output-stream.rvn' if a.output_stream else 'input-stream.rvn' if a.input_stream else 'stream-views.rvn' if a.stream_views else 'multiple-values.rvn' if a.multi_values else 'wide-integers.rvn' if a.wide_integers else 'small-integers.rvn' if a.small_integers else 'characters.rvn' if a.characters else 'bytes.rvn' if a.arrays else 'reference-cell.rvn' if a.references else 'numbers.rvn' if a.numeric else 'text-values.rvn' if a.text_values else 'interactive.rvn')
uses_arena=a.text_reader or a.text_writer or a.boxed_cases or a.utf8_text or a.collections or a.output_stream or a.input_stream or a.stream_views or a.numeric or a.multi_values or a.references or a.arrays or a.small_integers or a.wide_integers
host=base/'text-host.c' if uses_arena else faults/'host.c'
core,seed,library,ownership=(bundle/'lib'/n for n in ('Core.dll','System.runtime.neox','System.Runtime.dll','ownership.json'))
sha=lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
report=dict(profile='aot-console-text-reader-v1' if a.text_reader else 'aot-console-text-writer-v1' if a.text_writer else 'aot-console-boxed-cases-v1' if a.boxed_cases else 'aot-console-utf8-text-v1' if a.utf8_text else 'aot-console-collections-v1' if a.collections else 'aot-console-output-stream-v1' if a.output_stream else 'aot-console-input-stream-v1' if a.input_stream else 'aot-console-stream-views-v1' if a.stream_views else 'aot-console-multiple-values-v1' if a.multi_values else 'aot-console-wide-integers-v1' if a.wide_integers else 'aot-console-small-integers-v1' if a.small_integers else 'aot-console-characters-v1' if a.characters else 'aot-console-byte-arrays-v1' if a.arrays else 'aot-console-references-v1' if a.references else 'aot-console-numeric-v1' if a.numeric else 'aot-console-text-values-v1' if a.text_values else 'aot-console-interactive-v1',baseRevision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),SDKROOT=os.environ.get('SDKROOT'),inputs={str(p):sha(p) for p in (compiler,runtime,aot,core,seed,library,ownership,source,base/'console.c',base.parent/'aot-scalar/console.c',host,faults/'render.c',base/'text-arena.h',base/'text-arena.c')},commands=[])
def save(): (output/'validation.json').write_text(json.dumps(report,indent=2)+'\n')
def run(args,expected=0,**kwargs):
    r=subprocess.run(list(map(str,args)),cwd=ROOT,capture_output=True,timeout=120,**kwargs)
    report['commands'].append(dict(command=r.args,exit=r.returncode,stdout=r.stdout.decode() if len(r.stdout)<4000 else dict(sha256=hashlib.sha256(r.stdout).hexdigest(),bytes=len(r.stdout)),stderr=r.stderr.decode()))
    save();assert r.returncode==expected,r.stderr
    return r
assembly,obj,binary=(output/n for n in ('Interactive.dll','interactive.o','interactive'))
if a.reuse_compilation:
    prior=json.loads(a.reuse_compilation.read_text())
    for path in (compiler,core,seed,library,ownership,source):
        assert prior['inputs'][str(path)]==sha(path),f'Producer/source changed: {path}'
    producer=prior['commands'][0]
    assert producer['exit']==0 and producer['command'][:3]==['dotnet',str(compiler),'neoclr']
    emitted=Path(producer['command'][producer['command'].index('-o')+1])
    shutil.copy2(emitted,assembly)
    report['reusedCompilation']=dict(evidence=str(a.reuse_compilation.resolve()),evidenceSha256=sha(a.reuse_compilation),assemblySha256=sha(assembly),command=producer)
    save()
else:
    run(['dotnet',compiler,'neoclr','--core-reference',core,'--runtime-seed',seed,'--reference',library,'--bootstrap-intrinsics','--bootstrap-ownership',ownership,'--object-library','System.Runtime','-o',assembly,source])
context=['--system',seed,'--module',library,'--object-root',library]
cases=[(b'*',0),(b'A',0 if a.numeric or a.multi_values or a.input_stream else 2),(b'',0)]
if a.text_values or a.references or a.arrays or a.characters or a.small_integers or a.wide_integers or a.stream_views or a.output_stream or a.collections or a.utf8_text or a.boxed_cases or a.text_writer: cases=[(b'',0)]
if a.numeric or a.input_stream: cases += [(b'\x00',0),(b'\xff',0)]
if a.text_reader: cases=[(b'Raven\n',0),('é😀\r\n'.encode(),0),(b'a\x00z\n',0),(b'\n',0),(b'',0),(b'last',0),(b'first\nsecond\n',0),(b'\xff\n',1),(b'\xf0\x9f',1),(b'x'*129+b'\n',1)]
execution_context=context+(['--instructions','10000000'] if a.text_reader else [])
interpreted=[run([runtime,'run',assembly]+execution_context,expected=code,input=data) for data,code in cases]
read_end,write_end=os.pipe();os.close(read_end)
failed=run([runtime,'run',assembly]+execution_context,expected=6 if a.output_stream else 1,input=b'*',pass_fds=(write_end,),preexec_fn=lambda:os.dup2(write_end,1))
assert not failed.stderr if a.output_stream else failed.stderr.startswith(b'UserFault: console output failed\n' if a.text_writer or a.text_reader else b'RuntimeError: Runtime error\n')
flags=context+['--compile-system','--bind-user-fault','--bind-console-read-byte','--bind-console-write-line']
if a.output_stream or a.text_writer or a.text_reader: flags += ['--bind-console-stream-output']
if a.utf8_text or a.text_writer or a.text_reader: flags += ['--bind-utf8-text']
if a.wide_integers: flags += ['--bind-integer-text']
if a.characters: flags += ['--bind-character-text']
if uses_arena: flags += ['--bind-int32-to-string']
if a.references or a.arrays or a.stream_views or a.input_stream or a.output_stream or a.collections or a.utf8_text or a.boxed_cases or a.text_writer or a.text_reader: flags += ['--reference-arena']
inspection=json.loads(run([aot,'--inspect',assembly,'@entry','--closed-world']+flags).stdout)
assert inspection['admission']['accepted'] is True,inspection['admission']
selection=json.loads(run([aot,'--closed-world',assembly,'@entry',obj]+flags).stdout)
assert inspection['selection']==selection
report['selection']={k:selection[k] for k in ('loadSet','functions','nativeBindings','types','specialization','verifiedInterfaceRelationships','staticPrimitiveOwners','objectBaseProjection','interfaceDispatch','staticOwnerProjections','stringInstanceProjections','emptyRecordBoxes','emptyRecordBoxSites','arrayBackingProjection') if k in selection}
native_text=[]
if a.characters:
    manifest=ROOT/'tools/aot-native-text/Cargo.toml'
    run(['cargo','build','--locked','--release','--manifest-path',manifest])
    native_text=[manifest.parent/'target/release/libneoclr_aot_native_text.a']
    for path in [manifest,manifest.parent/'Cargo.lock',manifest.parent/'src/lib.rs',*native_text]: report['inputs'][str(path)]=sha(path)
run(['clang','-arch','arm64','-std=c11','-Wall','-Wextra','-Werror',host,faults/'render.c',base/'console.c',base.parent/'aot-scalar/console.c',*([base/'text-arena.c'] if uses_arena else []),obj,*native_text,'-o',binary])
imports={'_neoclr_console_read_byte_v1','_neoclr_console_write_line_utf8_v1'}
if a.text_values or a.references or a.arrays or a.characters or a.small_integers or a.wide_integers or a.stream_views or a.output_stream or a.collections or a.utf8_text or a.boxed_cases or a.text_writer: imports.remove('_neoclr_console_read_byte_v1')
if a.utf8_text or a.text_writer or a.text_reader: imports.update(['_neoclr_string_byte_count_v1','_neoclr_string_slice_utf8_v1'])
if a.characters: imports.add('_neoclr_is_single_grapheme_v1')
if uses_arena and not (a.wide_integers or a.stream_views or a.output_stream or a.boxed_cases): imports.add('_neoclr_int32_to_string_v1')
if a.wide_integers: imports.update(['_neoclr_int64_to_string_v1', '_neoclr_uint64_to_string_v1'])
if a.references or a.stream_views or a.input_stream or a.output_stream or a.collections or a.boxed_cases or a.text_writer or a.text_reader: imports.add('_neoclr_allocate_object_v1')
if a.arrays or a.input_stream or a.output_stream or a.text_writer or a.text_reader: imports.add('_neoclr_allocate_bytes_v1')
if a.collections or a.text_writer or a.text_reader: imports.add('_neoclr_reserve_bytes_v1')
if a.output_stream or a.text_writer or a.text_reader: imports.update(['_neoclr_console_write_bytes_v1','_neoclr_console_flush_v1'])
if a.text_writer or a.text_reader: imports.update(['_neoclr_utf8_encode_v1','_neoclr_check_bytes_initialized_v1'])
if a.text_reader: imports.add('_neoclr_utf8_decode_v1'); imports.discard('_neoclr_int32_to_string_v1')
assert set(run(['nm','-u',obj]).stdout.decode().split())==imports
deps=[line.split()[0] for line in run(['otool','-L',binary]).stdout.decode().splitlines()[1:]]
assert deps==['/usr/lib/libSystem.B.dylib']
with tempfile.TemporaryDirectory() as d:
    installed=Path(d)/'interactive';shutil.copy2(binary,installed)
    outcomes=[]
    for (data,code),reference in zip(cases,interpreted):
        r=subprocess.run([installed],cwd=d,env={},input=data,capture_output=True,timeout=10)
        assert (r.returncode,r.stdout,r.stderr)==(code,reference.stdout,reference.stderr),(data,r,reference)
        outcomes.append(dict(inputHex=data.hex(),exit=code,stdout=r.stdout.decode(),stderr=r.stderr.decode()))
    r=subprocess.run([installed],cwd=d,env={},input=b'*',pass_fds=(write_end,),preexec_fn=lambda:os.dup2(write_end,1),capture_output=True,timeout=10)
    assert r.returncode==(6 if a.output_stream else 1) and r.stderr==failed.stderr and not r.stdout,(r,failed)
    report['native']=dict(outcomes=outcomes,outputFailureMatchesInterpreter=True,outputFailureExit=r.returncode,outputFault=r.stderr.decode(),executableOnlyDirectory=True,emptyEnvironment=True,dynamicDependencies=deps)
os.close(write_end)
report['artifacts']={p.name:sha(p) for p in (assembly,obj,binary)}
save();print('Passed:',report['profile'],'standalone output and exact fault parity')

#!/usr/bin/env python3
"""Verify empty and pending callback settlement in interpreter, JIT and AOT (macOS)."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import xml.etree.ElementTree as ET

p = argparse.ArgumentParser(description=__doc__)
for name in ('library', 'compiler', 'runtime', 'backend', 'host', 'output'):
    p.add_argument('--' + name, type=Path, required=True)
a = p.parse_args()
lib, out = a.library.resolve(), a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
source = Path(__file__).with_name('Contracts.rvn').resolve()
manifest = json.loads((lib/'bundle.json').read_text())
modules = [lib/(name+'.dll') for name in manifest['assemblyNames']]
context = ['--system',lib/'System.runtime.neox','--object-root',lib/'System.Runtime.dll']
for module in modules:
    context += ['--module',module]
report = dict(passed=False, commands=[], inputs={}, source=source.read_text())
for path in [source, Path(__file__), lib/'bundle.json', *modules, a.compiler, a.compiler.parent/'Raven.CodeAnalysis.dll', a.compiler.parent/'Raven.CodeAnalysis.NeoClr.dll', lib/'System.runtime.neox', lib/'NeoCLR.ClassLibrary.props', a.runtime, a.backend, a.host]:
    report['inputs'][str(path.resolve())] = hashlib.sha256(path.read_bytes()).hexdigest()
def run(label, command):
    result = subprocess.run(list(map(str, command)), capture_output=True, text=True, timeout=240)
    report['commands'].append(dict(label=label, command=result.args, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
    if result.returncode:
        raise RuntimeError(label + ': ' + result.stdout + result.stderr)
    return result
try:
    project = ET.Element('Project', Sdk='Microsoft.NET.Sdk')
    ET.SubElement(project, 'Import', Project=str(lib/'NeoCLR.ClassLibrary.props'))
    properties = ET.SubElement(project, 'PropertyGroup')
    for key,value in dict(TargetFramework='net10.0',OutputType='Exe',AssemblyName='TaskCompletion',EnableDefaultCompileItems='false').items():
        ET.SubElement(properties,key).text=value
    ET.SubElement(ET.SubElement(project,'ItemGroup'),'Compile',Include=str(source))
    ET.ElementTree(project).write(out/'App.rvnproj',encoding='unicode')
    run('compile',['dotnet',a.compiler,'neoclr','--project',out/'App.rvnproj'])
    assembly=out/'bin/neoclr/TaskCompletion.dll'
    run('metadata',[a.runtime,'disassemble',assembly,out/'module.neoil'])
    text=(out/'module.neoil').read_text()
    metadata,_=json.JSONDecoder().raw_decode(text[text.index('{'):])
    expected='Task completion contracts passed\n'
    interpreted=run('interpreter',[a.runtime,'run',assembly,'--execution','interpreter',*context])
    assert interpreted.stdout==expected, interpreted.stdout
    flags='--compile-system --bind-user-fault --bind-stack-trace --reference-arena --native-gc --bind-console-read-byte --bind-console-write-line --bind-console-stream-output --bind-int32-to-string --bind-utf8-text --native-stack-budget --bind-integer-text --bind-character-text --bind-system-culture --bind-time-zone-rules --bind-cryptography --bind-tls --bind-task-queue --bind-scheduler --bind-timers --bind-socket-listener --bind-socket-accept --bind-socket-transfer --bind-socket-client --bind-runtime-async'.split()
    common=[*context,*flags]
    jit=run('jit',[a.backend,'--jit-run','--closed-world',assembly,'@entry',a.host,*common])
    assert jit.stdout==expected, jit.stdout
    obj=out/'app.o'
    run('aot-compile',[a.backend,'--closed-world',assembly,'@entry',obj,*common])
    launcher=out/'main.c'
    launcher.write_text('#include <stdint.h>\nextern int32_t neoclr_entry_v4(int32_t,int32_t*,void*);\nextern int32_t neoclr_jit_run_v1(const void*,int32_t,int32_t,const char**);\nint main(int n,const char**v){return neoclr_jit_run_v1((const void*)&neoclr_entry_v4,0,n,v);}\n')
    app=out/'app'
    run('link',['clang',launcher,obj,a.host,'-o',app])
    identity=run('host-id',['otool','-D',a.host]).stdout.splitlines()[1]
    run('host-bind',['install_name_tool','-change',identity,a.host.resolve(),app])
    native=run('aot',[app])
    assert native.stdout==expected, native.stdout
    report['passed']=True
finally:
    with gzip.GzipFile(out/'results.json.gz','wb',mtime=0) as f:
        f.write((json.dumps(report,indent=2)+'\n').encode())

#!/usr/bin/env python3
"""Build the same Serve guest against a selected source-built library cohort."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import xml.etree.ElementTree as ET

p = argparse.ArgumentParser(description=__doc__)
for key in ('library', 'compiler', 'backend', 'link-build', 'output'):
    p.add_argument('--' + key, type=Path, required=True)
p.add_argument('--source', type=Path, default=Path(__file__).resolve().parents[1]/'Server.rvn')
a = p.parse_args()
root = Path(__file__).resolve().parents[4]
lib, out = a.library.resolve(), a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
report = dict(passed=False, commands=[], inputs={})
def run(phase, command):
    result = subprocess.run(list(map(str, command)), capture_output=True, text=True)
    report['commands'].append(dict(phase=phase, command=result.args, exitCode=result.returncode,
                                  stdout=result.stdout, stderr=result.stderr))
    result.check_returncode()
try:
    project = ET.Element('Project', Sdk='Microsoft.NET.Sdk')
    ET.SubElement(project, 'Import', Project=str(lib/'NeoCLR.ClassLibrary.props'))
    props = ET.SubElement(project, 'PropertyGroup')
    for key, value in dict(TargetFramework='net10.0', OutputType='Exe', AssemblyName='App', EnableDefaultCompileItems='false').items():
        ET.SubElement(props, key).text = value
    ET.SubElement(ET.SubElement(project, 'ItemGroup'), 'Compile', Include=str(a.source.resolve()))
    ET.ElementTree(project).write(out/'App.rvnproj', encoding='unicode')
    run('source-compilation', ['dotnet', a.compiler.resolve(), 'neoclr', '--project', out/'App.rvnproj'])
    shutil.copy2(out/'bin/neoclr/App.dll', out/'app.dll')
    old = json.loads((a.link_build/'build.json').read_text())
    flags = next(c['command'] for c in old['commands'] if c['phase']=='native-compilation')
    flags = flags[flags.index('--compile-system'):]
    context = ['--system', lib/'System.runtime.neox', '--object-root', lib/'System.Runtime.dll']
    modules = [lib/(name+'.dll') for name in json.loads((lib/'bundle.json').read_text())['assemblyNames']]
    for module in modules: context += ['--module', module]
    run('native-compilation', [a.backend.resolve(), '--closed-world', out/'app.dll', '@entry', out/'app.o', *context, *flags])
    command = next(c['command'] for c in old['commands'] if c['phase']=='host-compile-link')[:]
    command = [str(out/'app.o') if x.endswith('/app.o') else x for x in command]
    for name, extra in [('app', []), ('app-paced', ['-DNEOCLR_GC_PACED'])]:
        command[command.index('-o')+1] = str(out/name)
        run('host-compile-link', command+extra)
    inputs = [Path(__file__), a.source, a.compiler, a.compiler.parent/'Raven.CodeAnalysis.dll', a.compiler.parent/'Raven.CodeAnalysis.NeoClr.dll', a.backend, *modules, lib/'System.runtime.neox', lib/'build-evidence.json', out/'app.o', out/'app.dll', out/'app', out/'app-paced']
    inputs += [Path(x) for x in command if Path(x).is_file()]
    inputs += [x for x in (root/'runtime/native').rglob('*') if x.suffix in ('.c','.h')]
    report['inputs'] = {str(x.resolve()):hashlib.sha256(x.read_bytes()).hexdigest() for x in inputs}
    report['passed'] = True
finally:
    (out/'build.json').write_text(json.dumps(report, indent=2)+'\n')

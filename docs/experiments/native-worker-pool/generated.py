#!/usr/bin/env python3
"""Qualify compiled Raven on private worker contexts, not shared-heap Task/HTTP."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
from xml.sax.saxutils import escape

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--shared', action='store_true', help='Test serialized callbacks on the pool heap')
p.add_argument('--bundle', type=Path, required=True)
p.add_argument('--libraries', type=Path, required=True)
p.add_argument('--validation', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
here = Path(__file__).resolve().parent
root = here.parents[2]
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
bundle, libraries = a.bundle.resolve(), a.libraries.resolve()
source = here/('Shared.rvn' if a.shared else 'Generated.rvn')
host = here/('shared-host.c' if a.shared else 'generated-host.c')
report = dict(passed=False, commands=[], inputs={}, scope='Shared serialized callback bridge' if a.shared else __doc__)
def record(path):
    report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
def run(command):
    command = list(map(str, command))
    try:
        r = subprocess.run(command, capture_output=True, text=True, timeout=60)
    except subprocess.TimeoutExpired:
        report['commands'].append(dict(command=command, outcome='timeout', timeoutSeconds=60))
        raise
    stdout = r.stdout
    if len(stdout) > 32000:
        artifact = out/('command-' + str(len(report['commands'])) + '.stdout')
        artifact.write_text(stdout)
        record(artifact)
        stdout = 'Large output retained at ' + str(artifact)
    report['commands'].append(dict(command=command, exitCode=r.returncode, stdout=stdout, stderr=r.stderr))
    r.check_returncode()
    return r.stdout
try:
    project = out/'App.rvnproj'
    project.write_text('<Project Sdk="Microsoft.NET.Sdk"><Import Project="'+escape(str(libraries/'NeoCLR.ClassLibrary.props'))+'" /><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Library</OutputType><AssemblyName>WorkerGenerated</AssemblyName><EnableDefaultCompileItems>false</EnableDefaultCompileItems></PropertyGroup><ItemGroup><Compile Include="'+escape(str(source))+'" /></ItemGroup></Project>')
    compiler = bundle/'sdk/tools/rvnc/rvnc.dll'
    codegen = bundle/'bin/neoclr-codegen'
    for f in [compiler, codegen, bundle/'bin/neoclr', libraries/'System.Runtime.dll', libraries/'System.runtime.neox', source, host, Path(__file__).resolve(), project]:
        record(f)
    run(['dotnet', compiler, 'neoclr', '--project', project])
    assembly = out/'bin/neoclr/WorkerGenerated.dll'
    record(assembly)
    listing = out/'listing.txt'
    run([bundle/'bin/neoclr', 'disassemble', assembly, listing])
    names = re.findall(r'"name": "(NeoMetadata_[A-F0-9]+\.F_43616C63756C617465)"', listing.read_text())
    assert len(names) == 1, names
    run([codegen, '--closed-world', assembly, names[0], out/'code.o', *(['--native-host-bootstrap', '--bind-user-fault', '--bind-int32-to-string', '--bind-utf8-text'] if a.shared else []), '--compile-system', '--reference-arena', '--native-gc', '--system', libraries/'System.runtime.neox', '--module', libraries/'System.Runtime.dll', '--object-root', libraries/'System.Runtime.dll'])
    baseline = json.loads(a.validation.read_text())
    base = baseline['commands'][0]
    assert base['exitCode'] == 0 and not any('sanitize' in s for s in base['command'])
    command = [str(host) if s.endswith('/test.c') else s for s in base['command']]
    command.extend([str(root/'runtime/native/memory/text-arena.c'), str(out/'code.o')])
    for f in [*list((root/'runtime/native/gc').glob('*.[ch]')), *list((root/'runtime/native/memory').glob('*.[ch]')), *list((root/'runtime/native/workers').glob('*.[ch]'))]:
        record(f)
    for variant, flags in [('optimized', []), ('ubsan', ['-fsanitize=undefined', '-fno-sanitize-recover=all'])]:
        cmd = command.copy()
        cmd[cmd.index('-o')+1] = str(out/variant)
        run(cmd + flags)
        run([out/variant])
    report['passed'] = True
finally:
    (out/'generated-validation.json').write_text(json.dumps(report, indent=2)+'\n')
print('PASS generated worker contexts in optimized and UBSan builds')

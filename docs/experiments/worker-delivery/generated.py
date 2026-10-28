#!/usr/bin/env python3
"""Qualify generated shared closures with bounded owner completion under both collectors."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
from xml.sax.saxutils import escape

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--codegen', type=Path, help='Explicit matching development code generator')
p.add_argument('--bundle', type=Path, required=True)
p.add_argument('--libraries', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
here = Path(__file__).resolve().parent
root = here.parents[2]
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
bundle, libraries = a.bundle.resolve(), a.libraries.resolve()
source = here/'Generated.rvn'
host = here/'generated-chain.c'
report = dict(passed=False, commands=[], inputs={}, scope=__doc__)
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
    codegen = a.codegen.resolve() if a.codegen else bundle/'bin/neoclr-codegen'
    for f in [compiler, codegen, bundle/'bin/neoclr', libraries/'System.Runtime.dll', libraries/'System.runtime.neox', source, host, Path(__file__).resolve(), project]:
        record(f)
    for name in ['gc_points.rs', 'gc_probe.rs', 'values.rs', 'fault_details.rs', 'lib.rs']:
        record(root/'runtime/codegen/src'/name)
    run(['dotnet', compiler, 'neoclr', '--project', project])
    assembly = out/'bin/neoclr/WorkerGenerated.dll'
    record(assembly)
    listing = out/'listing.txt'
    run([bundle/'bin/neoclr', 'disassemble', assembly, listing])
    names = re.findall(r'"name": "(NeoMetadata_[A-F0-9]+\.F_43616C63756C617465)"', listing.read_text())
    assert len(names) == 1, names
    run([codegen, '--closed-world', assembly, names[0], out/'code.o', '--native-host-bootstrap', '--bind-user-fault', '--bind-int32-to-string', '--bind-utf8-text', '--compile-system', '--reference-arena', '--native-gc', '--system', libraries/'System.runtime.neox', '--module', libraries/'System.Runtime.dll', '--object-root', libraries/'System.Runtime.dll'])
    native = root/'runtime/native'
    legacy = root/'docs/experiments/aot-console'
    live = native/'gc/concurrent'
    cc = [run(['xcrun','--find','clang']).strip(), '-isysroot', run(['xcrun','--show-sdk-path']).strip()]
    run(['python3',live/'generate.py','--output',out/'live-gc.c'])
    policy = json.loads((live/'policy.json').read_text())
    for mode in ('sync','concurrent'):
        defs = ['-DNEOCLR_NATIVE_GC'] if mode=='sync' else ['-D'+d for d in policy['defines']]+['-DDELIVERY_CONCURRENT','-I',live]
        gc = [native/'gc/native-gc.c'] if mode=='sync' else [out/'live-gc.c',live/'gate.c']
        for variant, flags in [('optimized', []), ('ubsan', ['-fsanitize=undefined', '-fno-sanitize-recover=all'])]:
            binary = out/f'{mode}-{variant}'
            run([*cc,'-std=c11','-O2','-Wall','-Wextra','-Werror','-Wno-deprecated-declarations',
                 '-pthread','-Wl,-dead_strip','-DNEOCLR_WORKER_DELIVERY',*defs,*flags,
                 '-I',native/'async','-I',native/'gc','-I',native/'workers','-I',legacy,
                 host,out/'code.o',native/'async/native-runtime.c',legacy/'socket-listener.c',
                 legacy/'native-stack.c',native/'memory/text-arena.c',native/'workers/native-workers.c',
                 native/'gc/root-probe.c',*gc,'-o',binary])
            run([binary])
    for f in [*native.rglob('*.c'),*native.rglob('*.h'),live/'policy.json',live/'generate.py',
              legacy/'socket-listener.c',legacy/'native-stack.c',out/'live-gc.c',out/'code.o']:
        record(f)
    report['passed'] = True
finally:
    (out/'generated-validation.json').write_text(json.dumps(report, indent=2)+'\n')
print('PASS generated closure delivery under sync/concurrent GC, optimized/UBSan')

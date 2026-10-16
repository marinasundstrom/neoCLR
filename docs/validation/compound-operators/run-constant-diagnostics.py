#!/usr/bin/env python3
"""Verify matching compile-time overflow rejection on .NET and native targets."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
from xml.sax.saxutils import escape

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--dotnet-compiler', type=Path, required=True)
parser.add_argument('--native-compiler', type=Path, required=True)
parser.add_argument('--core', type=Path, required=True)
parser.add_argument('--bundle', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
source = Path(__file__).resolve().with_name('ConstantOverflow.rvn')
output = args.output.resolve()
output.mkdir(parents=True, exist_ok=False)
project = output / 'native'
project.mkdir()
(project / 'Main.rvn').write_text(source.read_text())
(project / 'App.rvnproj').write_text(f'''<Project Sdk="Microsoft.NET.Sdk">
<Import Project="{escape(str(args.bundle.resolve() / 'NeoCLR.ClassLibrary.props'))}"/>
<PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType>
<AssemblyName>ConstantOverflow</AssemblyName></PropertyGroup></Project>
''')
report = {'passed': False, 'commands': []}
for target, command, image in (
    ('dotnet', ['dotnet', str(args.dotnet_compiler.resolve()), str(source), '--framework', 'net11.0',
                '--raven-core', str(args.core.resolve()), '-o', str(output / 'Rejected.dll')], output / 'Rejected.dll'),
    ('native', ['dotnet', str(args.native_compiler.resolve()), 'neoclr', '--project', str(project / 'App.rvnproj')],
     project / 'bin/neoclr/ConstantOverflow.dll'),
):
    result = subprocess.run(command, text=True, capture_output=True, timeout=300)
    log = result.stdout + result.stderr
    (output / (target + '.log')).write_text(log)
    report['commands'].append({'target': target, 'command': command, 'exitCode': result.returncode,
                               'output': log, 'assemblyExists': image.exists()})
    (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    assert result.returncode != 0 and log.count('RAV0374') >= 4 and not image.exists(), log
report['files'] = {str(p.resolve()): hashlib.sha256(p.read_bytes()).hexdigest() for p in
                   (source, Path(__file__), args.dotnet_compiler, args.native_compiler,
                    args.dotnet_compiler.parent / 'Raven.CodeAnalysis.dll',
                    args.native_compiler.parent / 'Raven.CodeAnalysis.dll', args.bundle / 'bundle.json')}
report['passed'] = True
(output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')

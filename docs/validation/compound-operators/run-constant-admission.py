#!/usr/bin/env python3
"""Retain explicit native admission limits for optional parameters and static initialization."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
from xml.sax.saxutils import escape

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--compiler', type=Path, required=True)
parser.add_argument('--bundle', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
output = args.output.resolve()
output.mkdir(parents=True, exist_ok=False)
report = {'passed': False, 'cases': []}
for name, reason in [('ConstantDefaults', 'callable declaration'), ('ReadonlyFields', 'static field initializers')]:
    source = Path(__file__).resolve().with_name(name + '.rvn')
    project = output / name
    project.mkdir()
    (project / 'Main.rvn').write_text(source.read_text())
    (project / 'App.rvnproj').write_text(f'<Project Sdk="Microsoft.NET.Sdk"><Import Project="{escape(str(args.bundle.resolve()))}/NeoCLR.ClassLibrary.props"/><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><AssemblyName>{name}</AssemblyName></PropertyGroup></Project>')
    command = ['dotnet', str(args.compiler.resolve()), 'neoclr', '--project', str(project / 'App.rvnproj')]
    result = subprocess.run(command, capture_output=True, text=True, timeout=300)
    log = result.stdout + result.stderr
    assert result.returncode != 0 and 'NEOMETA001' in log and reason in log, log
    assert not (project / 'bin/neoclr' / (name + '.dll')).exists()
    report['cases'].append({'source': str(source), 'sourceSha256': hashlib.sha256(source.read_bytes()).hexdigest(), 'command': command, 'exitCode': result.returncode, 'output': log})
report['files'] = {str(p.resolve()): hashlib.sha256(p.read_bytes()).hexdigest() for p in (Path(__file__), args.compiler.parent / 'Raven.CodeAnalysis.NeoClr.dll', args.compiler.parent / 'NeoCLR.Metadata.Experimental.dll', args.bundle / 'bundle.json')}
report['passed'] = True
report['compilerSha256'] = hashlib.sha256(args.compiler.read_bytes()).hexdigest()
report['codeAnalysisSha256'] = hashlib.sha256((args.compiler.parent / 'Raven.CodeAnalysis.dll').read_bytes()).hexdigest()
(output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')

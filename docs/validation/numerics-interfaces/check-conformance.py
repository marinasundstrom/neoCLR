#!/usr/bin/env python3
"""Check that the complex prototype requires explicit nominal capability conformance."""
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
source = Path(__file__).with_name('ComplexGeneric.rvn').read_text()
output = args.output.resolve()
output.mkdir(parents=True, exist_ok=False)
results = []
for name, text in (
    ('nominal-required', source.replace('Complex : AdditionOperators,', 'Complex :')),
    ('number-not-implied', source.replace('where T: AdditionOperators', 'where T: Number')),
):
    project = output / name
    project.mkdir()
    (project / 'Main.rvn').write_text(text)
    (project / 'App.rvnproj').write_text(f'''<Project Sdk="Microsoft.NET.Sdk">
<Import Project="{escape(str(args.bundle.resolve() / 'NeoCLR.ClassLibrary.props'))}"/>
<PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><AssemblyName>Conformance</AssemblyName></PropertyGroup>
</Project>''')
    command = ['dotnet', str(args.compiler.resolve()), 'neoclr', '--project', str(project / 'App.rvnproj')]
    run = subprocess.run(command, text=True, capture_output=True, timeout=300)
    log = run.stdout + run.stderr
    results.append(dict(case=name, command=command, exitCode=run.returncode, output=log,
                        sourceSha256=hashlib.sha256(text.encode()).hexdigest()))
    assert run.returncode != 0 and 'constraint' in log.lower(), log
report = dict(passed=True, cases=results)
(output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
print('Both nominal conformance checks passed')

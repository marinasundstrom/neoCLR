"""Qualify postfix carrier ! through Raven native emission and the neoCLR interpreter."""
import argparse
import json
from pathlib import Path
import subprocess
from xml.sax.saxutils import escape

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--dotnet', default='dotnet')
parser.add_argument('--compiler', type=Path, required=True)
parser.add_argument('--libraries', type=Path, required=True)
parser.add_argument('--runtime', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
output = args.output.resolve()
output.mkdir(parents=True, exist_ok=False)
libraries = args.libraries.resolve()
(output / 'App.rvnproj').write_text(f'''<Project Sdk="Microsoft.NET.Sdk">
  <Import Project="{escape(str(libraries / 'NeoCLR.ClassLibrary.props'))}" />
  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework>
    <OutputType>Exe</OutputType>
    <AssemblyName>CarrierUnwrap</AssemblyName>
    <EnableDefaultCompileItems>false</EnableDefaultCompileItems>
  </PropertyGroup>
  <ItemGroup><Compile Include="Main.rvn" /></ItemGroup>
</Project>
''')
checks = [
    ('result-success', 'let value: Result<int, string> = .Ok(42)\nreturn value!', 42),
    ('optional-success', 'let value: Optional<int> = .Some(42)\nreturn value!', 42),
    ('result-failure', 'let value: Result<int, string> = .Failure("expected")\nreturn value!', None),
    ('optional-failure', 'let value: Optional<int> = .None\nreturn value!', None),
    ('chaining', 'let value: Result<string, int> = .Ok("abc")\nreturn value!.Length', 3),
]
results = []
for name, body, expected in checks:
    source = 'import System.*\nfunc Main() -> int {\n' + body + '\n}\n'
    (output / 'Main.rvn').write_text(source)
    (output / f'{name}.rvn.txt').write_text(source)
    project = output / 'App.rvnproj'
    built = subprocess.run([args.dotnet, str(args.compiler.resolve()), 'neoclr', '--project', str(project)], capture_output=True, text=True)
    (output / f'{name}.build.log').write_text(built.stdout + built.stderr)
    assert built.returncode == 0, f'{name}: compilation failed; see {output}'
    artifact = output / 'bin/neoclr/CarrierUnwrap.dll'
    command = [str(args.runtime.resolve()), 'run', str(artifact), '--execution', 'interpreter',
               '--system', str(libraries / 'System.runtime.neox'),
               '--module', str(libraries / 'System.Runtime.dll'),
               '--object-root', str(libraries / 'System.Runtime.dll')]
    ran = subprocess.run(command, capture_output=True, text=True)
    (output / f'{name}.run.log').write_text(ran.stdout + ran.stderr)
    if expected is None:
        assert ran.returncode != 0 and 'Carrier unwrap failed' in ran.stderr, (name, ran.stdout, ran.stderr)
    else:
        assert ran.returncode == expected, (name, ran.returncode, ran.stdout, ran.stderr)
    results.append({'case': name, 'exitCode': ran.returncode, 'passed': True})
(output / 'validation.json').write_text(json.dumps(results, indent=2) + '\n')
print(json.dumps(results, indent=2))

#!/usr/bin/env python3
"""Compile source-authored structural intersections and execute trait dispatch."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--compiler', type=Path, required=True)
p.add_argument('--library', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
lib, out = a.library.resolve(), a.output.resolve()
out.mkdir(parents=True, exist_ok=True)
project = out / 'Probe.rvnproj'
project.write_text(f'''<Project Sdk="Microsoft.NET.Sdk"><Import Project="{lib}/NeoCLR.ClassLibrary.props"/><PropertyGroup><RavenNeoClrStructuralCompositions>true</RavenNeoClrStructuralCompositions><RavenNeoClrRuntimeAsyncHelper></RavenNeoClrRuntimeAsyncHelper><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><AssemblyName>SourceIntersection</AssemblyName><EnableDefaultCompileItems>false</EnableDefaultCompileItems></PropertyGroup><ItemGroup><Compile Remove="{lib}/RuntimeAwait.rvn"/><Compile Remove="{lib}/RuntimeGenerator.rvn"/><Compile Include="{Path(__file__).parent.resolve()}/Main.rvn" /></ItemGroup></Project>''')
subprocess.run(['dotnet', str(a.compiler.resolve()), 'neoclr', '--project', str(project)], check=True, cwd=ROOT)
catalog = json.loads((lib / 'bundle.json').read_text())
command = [str(ROOT / 'target/debug/neoclr'), 'run', str(out / 'bin/neoclr/SourceIntersection.dll'), '--system', str(lib / 'System.runtime.neox'), '--object-root', str(lib / 'System.Runtime.dll'), '--execution', 'interpreter', '--instructions', '100000000']
for name in ['System.Runtime']:
    command += ['--module', str(lib / f'{name}.dll')]
result = subprocess.run(command, capture_output=True, text=True, timeout=180)
print(result.stdout + result.stderr, end='')
report = dict(exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr,
              profile='Source-authored intersection signatures and trait dispatch; no token substitutions',
              libraryHashes=catalog['files'])
report['sha256'] = {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest() for path in [ROOT / 'target/debug/neoclr', out / 'bin/neoclr/SourceIntersection.dll', Path(__file__).with_name('Main.rvn')]}
(out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
if result.returncode or 'Source structural intersection passed' not in result.stdout:
    raise SystemExit('source intersection consumer failed')

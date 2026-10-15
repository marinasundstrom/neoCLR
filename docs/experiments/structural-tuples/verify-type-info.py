#!/usr/bin/env python3
"""Compile the Raven descriptor consumer and substitute explicit native test tokens."""
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
project.write_text(f'''<Project Sdk="Microsoft.NET.Sdk"><Import Project="{lib}/NeoCLR.ClassLibrary.props"/><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><AssemblyName>TupleInfo</AssemblyName><EnableDefaultCompileItems>false</EnableDefaultCompileItems></PropertyGroup><ItemGroup><Compile Include="{Path(__file__).parent.resolve()}/TypeInfo.rvn" /></ItemGroup></Project>''')
subprocess.run(['dotnet', str(a.compiler.resolve()), 'neoclr', '--project', str(project)], check=True, cwd=ROOT)
subprocess.run(['cargo', 'build', '--bin', 'neoclr', '--example', 'tuple_info_fixture'], check=True, cwd=ROOT)
subprocess.run([str(ROOT / 'target/debug/examples/tuple_info_fixture'), str(out / 'bin/neoclr/TupleInfo.dll'), str(out / 'TupleInfo.neox')], check=True, cwd=ROOT)
catalog = json.loads((lib / 'bundle.json').read_text())
for name, digest in catalog['files'].items():
    if hashlib.sha256((lib / name).read_bytes()).hexdigest() != digest:
        raise SystemExit('library hash mismatch: ' + name)
command = [str(ROOT / 'target/debug/neoclr'), 'run', str(out / 'TupleInfo.neox'), '--system', str(lib / 'System.runtime.neox'), '--object-root', str(lib / 'System.Runtime.dll'), '--execution', 'interpreter', '--instructions', '100000000']
for name in ['System.Runtime']:
    command += ['--module', str(lib / f'{name}.dll')]
result = subprocess.run(command, capture_output=True, text=True, timeout=180)
print(result.stdout + result.stderr, end='')
report = dict(exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr,
              fixture='Two native LoadTypeToken substitutions isolate descriptor reflection from source emission',
              libraryHashes={name: digest for name, digest in catalog['files'].items() if name in ['Core.dll', 'System.Runtime.dll', 'System.runtime.neox', 'NeoCLR.ClassLibrary.props']},
              bundleManifestSha256=hashlib.sha256((lib / 'bundle.json').read_bytes()).hexdigest())
report['sha256'] = {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest() for path in [ROOT / 'target/debug/neoclr', out / 'TupleInfo.neox', Path(__file__).with_name('TypeInfo.rvn')]}
(out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
if result.returncode or 'Tuple descriptors passed' not in result.stdout:
    raise SystemExit('descriptor consumer failed')

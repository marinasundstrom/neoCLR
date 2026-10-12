#!/usr/bin/env python3
"""Build and run the people-query sample with an installed native JIT bundle."""
import argparse
import hashlib
import json
from pathlib import Path
import runpy
import shutil
import subprocess
from xml.sax.saxutils import quoteattr

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--bundle', required=True, type=Path)
p.add_argument('--port', required=True, type=int)
p.add_argument('--output', type=Path, default=Path('build'))
a = p.parse_args()
root, out = a.bundle.resolve(), a.output.resolve()
if not (root / 'lib/System.Data.PostgreSql.dll').is_file():
    p.error('This bundle does not contain the optional PostgreSQL provider')
out.mkdir(parents=True, exist_ok=True)
shutil.copyfile(Path(__file__).with_name('Main.rvn'), out / 'Main.rvn')
(out / 'App.rvnproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><Import Project=' +
    quoteattr(str(root / 'lib/NeoCLR.ClassLibrary.props')) +
    '/><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType>' +
    '<AssemblyName>People</AssemblyName></PropertyGroup></Project>')
inputs = [Path(__file__).resolve(), Path(__file__).with_name('Main.rvn').resolve(),
          root / 'manifest.json', root / 'sdk/tools/rvnc/rvnc.dll', root / 'bin/neoclr']
report = dict(passed=False, commands=[], inputsSha256={
    str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in inputs})

def run(command):
    command = list(map(str, command))
    result = subprocess.run(command, cwd=out, capture_output=True, text=True, timeout=180)
    report['commands'].append(dict(command=command, exitCode=result.returncode,
                                   stdout=result.stdout, stderr=result.stderr))
    if result.returncode:
        raise RuntimeError(result.stderr)
    return result

try:
    run(['dotnet', root / 'sdk/tools/rvnc/rvnc.dll', 'neoclr', '--project', out / 'App.rvnproj'])
    runtime, assembly = root / 'bin/neoclr', out / 'bin/neoclr/People.dll'
    prepare = runpy.run_path(str(root / 'tools/run-native-sample.py'))['reflection_roots']
    roots = prepare(runtime, assembly, ['Person'], run)
    result = run([runtime, 'run', assembly, *roots, '--', str(a.port)])
    if result.stdout != "42: Café team\n43: O'Brien\n" or result.stderr:
        raise RuntimeError('Unexpected sample output: ' + result.stdout + result.stderr)
    print(result.stdout, end='')
    report['passed'] = True
finally:
    (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')

#!/usr/bin/env python3
"""Build the controlled JSON HTTP handler with explicit matching reflection roots."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import runpy
import subprocess
from xml.sax.saxutils import escape

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
root = Path(__file__).resolve().parents[4]
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
build = json.loads(a.build.read_text())
assert build['passed']
source = next(c['command'][:] for c in build['commands'] if c.get('phase') == 'source-compilation')
native = next(c['command'][:] for c in build['commands'] if c.get('phase') == 'native-compilation')
bundle = Path(source[1]).parents[3]
manifest = json.loads((bundle/'manifest.json').read_text())
report = dict(passed=False, commands=[], inputs={})

def record(path):
    path = Path(path).resolve()
    report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()

def run(cmd):
    cmd = list(map(str, cmd))
    r = subprocess.run(cmd, env=dict(os.environ, NeoClrBundleRoot=str(bundle)), capture_output=True, text=True, timeout=180)
    report['commands'].append(dict(command=cmd, exitCode=r.returncode, stdoutSha256=hashlib.sha256(r.stdout.encode()).hexdigest(), stderr=r.stderr))
    assert r.returncode == 0, (r.stderr, r.stdout[-2000:])
    for item in cmd:
        path = Path(item)
        if path.is_absolute() and path.is_file(): record(path)
    return r

try:
    sources = [Path(__file__).with_name('JsonServer.rvn'), root/'benchmarks/http-server/Model.rvn']
    project = out/'JsonComparison.rvnproj'
    includes = ''.join(f'<Compile Include="{escape(str(s.resolve()))}"/>' for s in sources)
    project.write_text(f'<Project Sdk="Microsoft.NET.Sdk"><Import Project="{escape(str(bundle/"lib/NeoCLR.ClassLibrary.props"))}"/><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><AssemblyName>JsonComparison</AssemblyName><EnableDefaultCompileItems>false</EnableDefaultCompileItems></PropertyGroup><ItemGroup>{includes}</ItemGroup></Project>')
    source[-1] = str(project)
    run(source)
    assembly = next(out.rglob('JsonComparison.dll'))
    roots = runpy.run_path(str(root/'scripts/run-native-sample.py'))['reflection_roots'](bundle/manifest['runtime'], assembly, ['Person'], run)
    native[2], native[4] = str(assembly), str(out/'app.o')
    run(native+list(map(str, roots)))
    for path in [*sources, Path(__file__), a.build, root/'scripts/run-native-sample.py', root/'runtime/codegen/src/gc_probe.rs']: record(path)
    report['passed'] = True
finally:
    (out/'build.json').write_text(json.dumps(report, indent=2)+'\n')
print('JSON handler compiled with matching Person reflection roots')

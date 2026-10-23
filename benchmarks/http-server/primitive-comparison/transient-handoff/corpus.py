#!/usr/bin/env python3
"""Collect work counts from existing repository samples under the same native host."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
from xml.sax.saxutils import escape

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--diagnostic', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
root = Path(__file__).resolve().parents[4]
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
build = json.loads(a.build.read_text())
recipes = {c['phase']: c['command'] for c in build['commands'] if 'phase' in c}
bundle = Path(recipes['source-compilation'][1]).parents[3]
diag = json.loads(a.diagnostic.read_text())
assert build['passed'] and diag['passed']
report = dict(passed=False, commands=[], inputs={}, cases=[], scope='Two complete sample invocations, one isolated owner, paced GC; counts include setup/teardown. Not capacity or per-HTTP-request results.')

def record(path):
    path = Path(path).resolve()
    report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()

def run(command, label):
    command = list(map(str, command))
    r = subprocess.run(command, env=dict(os.environ, NeoClrBundleRoot=str(bundle)), capture_output=True, text=True, timeout=180)
    (out/(label+'.stdout')).write_text(r.stdout)
    report['commands'].append(dict(command=command, exitCode=r.returncode, stdoutSha256=hashlib.sha256(r.stdout.encode()).hexdigest(), stderr=r.stderr))
    assert r.returncode == 0, (label, r.stderr, r.stdout[-2000:])
    for item in command:
        path = Path(item)
        if path.is_absolute() and path.is_file(): record(path)
    return r

try:
    for name, expected in [('tasks', 'Completion queued'), ('collections', 'Filtered orders share identity'), ('json', 'JSON object mapping checks passed')]:
        directory = out/name
        directory.mkdir()
        sources = {'tasks': [root/'docs/experiments/raven-target/samples/library-tasks.rvn'], 'collections': [root/'docs/experiments/raven-target/samples/application-order-collections.rvn'], 'json': [root/'docs/experiments/json-object-mapping/Main.rvn', root/'docs/experiments/json-object-mapping/Mapping.rvn']}[name]
        adapted = []
        for source in sources:
            record(source)
            text = source.read_text()
            # The shared native host requires an async entry to export queue-drain
            # support. Preserve the sample body and use the same entry wrapper as HTTP.
            text = text.replace('func Main() {', 'async func Main() -> System.Concurrency.Tasks.Task<()> {')
            copy = directory/source.name
            copy.write_text(text)
            adapted.append(copy)
            record(copy)
        sources = adapted
        project = directory/'App.rvnproj'
        includes = ''.join(f'<Compile Include="{escape(str(source))}"/>' for source in sources)
        project.write_text(f'<Project Sdk="Microsoft.NET.Sdk"><Import Project="{escape(str(bundle / "lib/NeoCLR.ClassLibrary.props"))}"/><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><AssemblyName>App</AssemblyName><EnableDefaultCompileItems>false</EnableDefaultCompileItems></PropertyGroup><ItemGroup>{includes}</ItemGroup></Project>')
        cmd = recipes['source-compilation'][:]
        cmd[-1] = str(project)
        run(cmd, name+'-source')
        cmd = recipes['native-compilation'][:]
        original = cmd[4]
        cmd[2] = str(next(directory.rglob('App.dll')))
        obj = directory/'app.o'
        cmd[4] = str(obj)
        run(cmd, name+'-native')
        cmd = diag['commands'][0]['command'][:]
        old_obj = next(s for s in cmd if s.endswith('.o'))
        cmd = [str(obj) if s == old_obj else s for s in cmd]
        binary = directory/'app'
        cmd[cmd.index('-o')+1] = str(binary)
        run(cmd, name+'-host')
        result = run([binary], name+'-run')
        assert result.stdout.count(expected) == 2, (name, result.stdout)
        errors = result.stderr.splitlines()
        assert len(errors) == 2, errors
        _, handoffs, boundaries = next(s for s in errors if s.startswith('COUNTS ')).split()
        _, allocs, payload, collections = next(s for s in errors if s.startswith('GC ')).split()
        report['cases'].append(dict(sample=name, invocations=2, transientPublications=int(handoffs), boundaries=int(boundaries), allocations=int(allocs), payloadBytes=int(payload), collections=int(collections), objectBytes=obj.stat().st_size, stdout=result.stdout))
    record(__file__)
    record(a.build)
    record(a.diagnostic)
    report['passed'] = True
finally:
    (out/'corpus.json').write_text(json.dumps(report, indent=2)+'\n')
print('Ordinary tasks, collections and JSON samples passed with work counts')

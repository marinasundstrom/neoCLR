#!/usr/bin/env python3
"""Exercise the ordinary public CLI reference and legacy interpreter library."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
from xml.sax.saxutils import escape

ROOT = Path(__file__).resolve().parents[3]
p = argparse.ArgumentParser(description=__doc__)
for name in ('compiler', 'bridge', 'runtime', 'output'):
    p.add_argument('--' + name, type=Path, required=True)
a = p.parse_args()
compiler, bridge, runtime, output = (getattr(a, n).resolve() for n in ('compiler', 'bridge', 'runtime', 'output'))
output.mkdir(parents=True, exist_ok=False)
(output / 'demo').mkdir()
core = output / 'demo/NeoCLR.CoreProbe.dll'
source = Path(__file__).with_name('Public.rvn').resolve()
report = {'commands': [], 'inputs': {str(f): hashlib.sha256(f.read_bytes()).hexdigest() for f in (compiler, bridge, runtime, source)}}

def run(command):
    r = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True, timeout=180)
    report['commands'].append({'command': r.args, 'exit': r.returncode, 'stdout': r.stdout, 'stderr': r.stderr})
    (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    if r.returncode:
        raise RuntimeError(r)

run(['dotnet', bridge, '--reference-core', core])
project = output / 'Public.rvnproj'
project.write_text(f'<Project><PropertyGroup><OutputType>Exe</OutputType><AssemblyName>Public</AssemblyName><NeoCLRRoot>{escape(str(output))}</NeoCLRRoot></PropertyGroup><Import Project="{escape(str(ROOT / "build/NeoCLR.Raven.props"))}"/><ItemGroup><Compile Include="{escape(str(source))}"/></ItemGroup></Project>')
run(['dotnet', compiler, project, '--no-project-restore', '-o', output / 'compiled'])
run(['dotnet', bridge, '--import', output / 'compiled/Public.dll', core, output / 'imported'])
run(['python3', ROOT / 'docs/experiments/raven-target/collection_library.py', output / 'System.neoil'])
run([runtime, 'run', output / 'imported/App.neoil', '--system', output / 'System.neoil', '--instructions', '100000000'])
print('Public StringBuilder and String.Join CLI consumer passed')

#!/usr/bin/env python3
"""Build and execute the opt-in Raven/native structural composition POC."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--raven', type=Path, required=True, help='Companion experimental Raven checkout')
parser.add_argument('--output', type=Path, default=ROOT / 'target/structural-composition-poc')
args = parser.parse_args()
raven = args.raven.resolve()
output = args.output.resolve()
output.mkdir(parents=True, exist_ok=True)
subprocess.run(['cargo', 'build', '--features', 'experimental-compositions', '--bin', 'neoclr'], cwd=ROOT, check=True)
subprocess.run(['dotnet', 'run', '--project', 'tools/NeoClrMetadataProbe',
    f'-p:NeoClrMetadataProject={ROOT}/tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj',
    '-p:WarningLevel=0', '--', '--composition-calls',
    str(ROOT / 'api-docs/reference/NeoCLR.CoreProbe.dll'), str(output)], cwd=raven, check=True)
results = []
for profile in ['', 'generic']:
    directory = output / profile
    for name in ['First', 'Second', 'Direct', 'Discard', 'Local']:
        run = subprocess.run([str(ROOT / 'target/debug/neoclr'), 'run', str(directory / f'{name}.neox'),
            '--module', str(directory / 'CompositionPocProvider.neox'), '--execution', 'interpreter', '--show-result'],
            cwd=ROOT, capture_output=True, text=True)
        label = f'{profile}/{name}' if profile else name
        if run.returncode != 42 or 'Int32(42)' not in run.stdout + run.stderr:
            raise RuntimeError(f'{label}: exit {run.returncode}\n{run.stdout}\n{run.stderr}')
        results.append({'name': label, 'exit': run.returncode, 'result': 'Int32(42)',
            'source': (directory / f'{name}.rvn').read_text().strip()})
        print(f'PASS {label}: Int32(42)')
for kind in ['Union', 'Intersection']:
    for producer in ['Host', 'Raven']:
        name = f'{kind}{producer}'
        directory = output / 'wrappers'
        run = subprocess.run([str(ROOT / 'target/debug/neoclr'), 'run', str(directory / f'{name}.neox'),
            '--module', str(directory / f'{kind}Provider.neox'), '--execution', 'interpreter', '--show-result'],
            cwd=ROOT, capture_output=True, text=True)
        if run.returncode != 42 or 'Int32(42)' not in run.stdout + run.stderr:
            raise RuntimeError(f'wrappers/{name}: exit {run.returncode}\n{run.stdout}\n{run.stderr}')
        results.append({'name': f'wrappers/{name}', 'exit': run.returncode, 'result': 'Int32(42)',
            'producer': producer, 'contract': f'Wrapper<{kind}<A,B>>'})
        print(f'PASS wrappers/{name}: Int32(42)')
def revision(path):
    return {'commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=path, text=True).strip(),
            'dirty': bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=path, text=True).strip())}
report = {'neoclr': revision(ROOT), 'raven': revision(raven), 'feature': 'experimental-compositions',
          'mode': 'interpreter', 'programs': results,
          'artifacts': {str(p.relative_to(output)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(output.rglob('*.neox'))}}
(output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
print(f'Validation: {output / "validation.json"}')

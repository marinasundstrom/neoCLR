#!/usr/bin/env python3
"""Validate real Task composition transport against an explicitly supplied native library bundle."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--raven', type=Path, required=True)
parser.add_argument('--inputs', type=Path, required=True, help='Directory containing matching Core.dll, System.Runtime.dll and System.runtime.neox')
parser.add_argument('--output', type=Path, default=ROOT / 'target/task-composition-await')
args = parser.parse_args()
inputs, output, raven = args.inputs.resolve(), args.output.resolve(), args.raven.resolve()
output.mkdir(parents=True, exist_ok=True)
subprocess.run(['cargo', 'build', '--features', 'experimental-compositions', '--bin', 'neoclr'], cwd=ROOT, check=True)
subprocess.run(['dotnet', 'run', '--project', 'tools/NeoClrMetadataProbe',
    f'-p:NeoClrMetadataProject={ROOT}/tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj',
    '-p:WarningLevel=0', '--', '--task-composition-await', str(inputs), str(output)], cwd=raven, check=True)
names = ['TaskCompositionLocal'] + [f'Await{mode}{kind}' for mode in ['Completed', 'Pending'] for kind in ['Response', 'HttpError', 'JsonError']]
def execute(name):
    run = subprocess.run([str(ROOT / 'target/debug/neoclr'), 'run', str(output / f'{name}.neox'),
        '--module', str(output / 'TaskCompositionProvider.neox'), '--module', str(inputs / 'System.Runtime.dll'),
        '--system', str(inputs / 'System.runtime.neox'), '--object-root', str(inputs / 'System.Runtime.dll'),
        '--execution', 'interpreter', '--show-result'], capture_output=True, text=True, timeout=180)
    if run.returncode != 42 or 'Int32(42)' not in run.stdout + run.stderr:
        raise RuntimeError(f'{name}: exit {run.returncode}\n{run.stdout}\n{run.stderr}')
    print(f'PASS {name}: Int32(42)', flush=True)
    return {'name': name, 'result': 'Int32(42)'}
with ThreadPoolExecutor(max_workers=2) as executor:
    results = list(executor.map(execute, names))
def revision(path):
    return {'commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=path, text=True).strip(),
            'dirty': bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=path, text=True).strip())}
report = {'neoclr': revision(ROOT), 'raven': revision(raven), 'feature': 'experimental-compositions',
          'programs': results, 'runtimeSha256': hashlib.sha256((ROOT / 'target/debug/neoclr').read_bytes()).hexdigest(),
          'await': 'completed and pending; all alternatives; payload live across a second await',
          'artifacts': {name: hashlib.sha256((output / name).read_bytes()).hexdigest() for name in
                        ['TaskCompositionProvider.neox'] + [name + '.neox' for name in names]},
          'inputs': {name: hashlib.sha256((inputs / name).read_bytes()).hexdigest()
                     for name in ['Core.dll', 'System.Runtime.dll', 'System.runtime.neox']}}
(output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
print('PASS real Task composition locals and await result delivery')

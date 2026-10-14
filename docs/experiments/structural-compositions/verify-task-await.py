#!/usr/bin/env python3
"""Validate real Task composition transport against an explicitly supplied native library bundle."""
import argparse
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
subprocess.run(['dotnet', 'run', '--project', 'tools/NeoClrMetadataProbe',
    f'-p:NeoClrMetadataProject={ROOT}/tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj',
    '-p:WarningLevel=0', '--', '--task-composition-await', str(inputs), str(output)], cwd=raven, check=True)
run = subprocess.run([str(ROOT / 'target/debug/neoclr'), 'run', str(output / 'TaskCompositionLocal.neox'),
    '--module', str(output / 'TaskCompositionProvider.neox'), '--module', str(inputs / 'System.Runtime.dll'),
    '--system', str(inputs / 'System.runtime.neox'), '--object-root', str(inputs / 'System.Runtime.dll'),
    '--execution', 'interpreter', '--show-result'], capture_output=True, text=True, timeout=180)
if run.returncode != 42 or 'Int32(42)' not in run.stdout + run.stderr:
    raise RuntimeError(f'exit {run.returncode}\n{run.stdout}\n{run.stderr}')
def revision(path):
    return {'commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=path, text=True).strip(),
            'dirty': bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=path, text=True).strip())}
report = {'neoclr': revision(ROOT), 'raven': revision(raven), 'feature': 'experimental-compositions',
          'result': 'Int32(42)', 'await': 'binds; state-machine field storage remains rejected',
          'inputs': {name: hashlib.sha256((inputs / name).read_bytes()).hexdigest()
                     for name in ['Core.dll', 'System.Runtime.dll', 'System.runtime.neox']}}
(output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
print('PASS real Task composition source local: Int32(42); await storage remains pending')

#!/usr/bin/env python3
"""Validate ready/pending/cancelled value async and compare managed allocation counts."""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
parser.add_argument('--runner', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--case', choices=['ready', 'one', 'two', 'cancel', 'ready-cancel', 'payloads'])
args = parser.parse_args()
bundle = args.toolchain_root.resolve()
output = args.output.resolve()
output.mkdir(parents=True, exist_ok=True)
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
template = Path(__file__).with_name('Matrix.rvn').read_text()
results = {}
for case, ready, finish, expected in [
    ('ready', '_ = first.Complete(1)\n        _ = second.Complete(1)', '_ = second.Complete(1)', '42\n'),
    ('one', '_ = second.Complete(1)', '_ = second.Complete(1)', '42\n'),
    ('two', '', '_ = second.Complete(1)', '42\n'),
    ('cancel', '', '_ = second.Cancel()', 'Cancelled\n'),
    ('ready-cancel', '_ = first.Cancel()', '_ = second.Complete(1)', 'Cancelled\n'),
    ('payloads', '', '', 'Unit\nExpected error\n'),
]:
    if args.case and args.case != case:
        continue
    source = Path(__file__).with_name('Payloads.rvn').read_text() if case == 'payloads' else template
    results[case] = {}
    for policy, heap in [('heap', 'true'), ('value', 'false')]:
        name = f'{case}-{policy}'
        with tempfile.TemporaryDirectory(prefix='neoclr-async-matrix-') as folder:
            root = Path(folder)
            (root / 'Main.rvn').write_text(source.replace('// READY', ready).replace('// FINISH', finish))
            (root / 'Probe.rvnproj').write_text(f'''<Project DefaultTargets="Build">
  <Import Project="$(NeoCLRRoot)/build/NeoCLR.Raven.props" />
  <PropertyGroup><RavenHeapAsyncStateMachines>{heap}</RavenHeapAsyncStateMachines></PropertyGroup>
  <ItemGroup><Compile Include="Main.rvn" /></ItemGroup>
  <Import Project="$(NeoCLRRoot)/build/NeoCLR.Raven.targets" />
</Project>
''')
            build = subprocess.run(['dotnet', 'msbuild', str(root / 'Probe.rvnproj'), '-nologo', '-v:minimal',
                                    '-p:Configuration=Release', '-t:NeoCLRImport'], env=env,
                                   capture_output=True, text=True, timeout=120)
            (output / f'{name}-build.log').write_text(build.stdout + build.stderr)
            assert build.returncode == 0, f'{name}: build/import failed; see logs'
            app = next((root / 'obj').glob('**/imported/App.neoil'))
            shutil.copyfile(app, output / f'{name}.neoil')
            run = subprocess.run([str(args.runner.resolve()), str(app), str(bundle / 'lib/System.neoil'), '96'],
                                 capture_output=True, text=True, timeout=120)
            (output / f'{name}-run.log').write_text(run.stdout + run.stderr)
            assert run.returncode == 0 and run.stdout == expected, f'{name}: {run.stdout}\n{run.stderr}'
            stats = {key: int(value) for key, value in re.findall(r'(\w+)=(\d+)', run.stderr)}
            assert stats['collections'] > 1 and stats['live'] == 0, stats
            results[case][policy] = stats
            print(name, stats, flush=True)
    delta = results[case]['heap']['allocated'] - results[case]['value']['allocated']
    assert delta == (1 if case in ('ready', 'ready-cancel', 'payloads') else 0), (case, delta)
(output / 'results.json').write_text(json.dumps(results, indent=2) + '\n')
print('All async state ownership and allocation comparisons passed')

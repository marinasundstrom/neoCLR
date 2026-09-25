"""Check the development HTTP addressing contract without using network I/O."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
parser.add_argument('--runner', type=Path, required=True)
args = parser.parse_args()
bundle = args.toolchain_root.resolve()
here = Path(__file__).resolve().parent
cases = json.loads((here / 'cases.json').read_text())
with tempfile.TemporaryDirectory(prefix='neoclr-http-base-') as folder:
    root = Path(folder)
    shutil.copyfile(here / 'HttpBase.rvnproj', root / 'HttpBase.rvnproj')
    source = (here / 'Main.rvn').read_text()
    for key, marker, function in [('resolved', 'RESOLUTION_CASES', 'Resolved'),
                                  ('rejected', 'REJECTED_CASES', 'Rejected')]:
        statements = ['    ' + function + '(' + ', '.join(json.dumps(arg) for arg in row) + ')'
                      for row in cases[key]]
        source = source.replace('    // ' + marker, '\n'.join(statements))
    (root / 'Main.rvn').write_text(source)
    env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
    built = subprocess.run(['dotnet', 'msbuild', str(root / 'HttpBase.rvnproj'),
                            '-nologo', '-v:minimal'], env=env, capture_output=True,
                           text=True, timeout=240)
    assert built.returncode == 0, built.stdout + built.stderr
    result = subprocess.run([str(args.runner.resolve()), str(root / 'bin/neoclr/Debug/App.neoil'),
                             str(bundle / 'lib/System.neoil'), '256', '100000000'],
                            capture_output=True, text=True, timeout=300)
    assert result.returncode == 0, result.stdout + result.stderr
    assert 'HTTP base address and overload checks passed' in result.stdout, result.stdout
    assert 'live=0' in result.stderr, result.stderr
    print(result.stdout + result.stderr, flush=True)

    comparison = root / 'reference'
    comparison.mkdir()
    shutil.copyfile(here / 'Reference.cs', comparison / 'Program.cs')
    (comparison / 'Reference.csproj').write_text(
        '<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework>'
        '<OutputType>Exe</OutputType></PropertyGroup></Project>')
    baseline = subprocess.run(['dotnet', 'run', '--project', str(comparison / 'Reference.csproj'),
                               '--', str(here / 'cases.json')],
                              capture_output=True, text=True, timeout=150)
    assert baseline.returncode == 0, baseline.stdout + baseline.stderr
    print(baseline.stdout, flush=True)

"""Focused cancellation lifecycle, access and .NET comparison checks."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
parser.add_argument('--runner', type=Path, required=True)
parser.add_argument('--guards-only', action='store_true', help='Skip an already validated main runtime sample')
args = parser.parse_args()
bundle = args.toolchain_root.resolve()
here = Path(__file__).resolve().parent
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))

with tempfile.TemporaryDirectory(prefix='neoclr-cancellation-') as folder:
    root = Path(folder)
    shutil.copyfile(here / 'Cancellation.rvnproj', root / 'Cancellation.rvnproj')
    main = root / 'Main.rvn'

    def build():
        return subprocess.run(['dotnet', 'msbuild', str(root / 'Cancellation.rvnproj'),
                               '-nologo', '-v:minimal'], env=env, capture_output=True,
                              text=True, timeout=240)

    def run():
        return subprocess.run([str(args.runner.resolve()), str(root / 'bin/neoclr/Debug/App.neoil'),
                               str(bundle / 'lib/System.neoil'), '256', '100000000'],
                              capture_output=True, text=True, timeout=300)

    if not args.guards_only:
        shutil.copyfile(here / 'Main.rvn', main)
        built = build()
        assert built.returncode == 0, built.stdout + built.stderr
        result = run()
        assert result.returncode == 0, result.stdout + result.stderr
        assert 'Cancellation ownership and callback checks passed' in result.stdout, result.stdout
        assert 'live=0' in result.stderr, result.stderr
        print(result.stdout + result.stderr, flush=True)

    main.write_text('import System.*\nimport System.Concurrency.*\n'
                    'func Main() {\n    let source = CancellationTokenSource()\n'
                    '    let token = source.Token\n    source.Dispose()\n'
                    '    _ = token.Register(() => Console.WriteLine("Must not run"))\n}\n')
    built = build()
    assert built.returncode == 0, built.stdout + built.stderr
    result = run()
    assert result.returncode != 0, result.stdout + result.stderr
    assert 'Cancellation source is disposed' in result.stderr, result.stderr
    assert 'Must not run' not in result.stdout, result.stdout
    print('Registration after source disposal rejected', flush=True)

    main.write_text('import System.Concurrency.*\nfunc Main() {\n'
                    '    let source = CancellationTokenSource()\n'
                    '    _ = source.IsDisposed\n}\n')
    rejected = build()
    diagnostic = rejected.stdout + rejected.stderr
    assert rejected.returncode != 0 and 'IsDisposed' in diagnostic and 'RAV' in diagnostic, diagnostic
    print('Internal cancellation helper unavailable to Raven applications', flush=True)

    comparison = root / 'reference'
    comparison.mkdir()
    shutil.copyfile(here / 'Reference.cs', comparison / 'Program.cs')
    (comparison / 'Reference.csproj').write_text(
        '<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework>'
        '<OutputType>Exe</OutputType></PropertyGroup></Project>')
    baseline = subprocess.run(['dotnet', 'run', '--project', str(comparison / 'Reference.csproj')],
                              capture_output=True, text=True, timeout=150)
    assert baseline.returncode == 0, baseline.stdout + baseline.stderr
    print(baseline.stdout, flush=True)

#!/usr/bin/env python3
"""Compile a standard union and inspect its CLI shape before attempting import."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bundle', required=True, type=Path)
parser.add_argument('--expect-import-rejection', action='store_true',
                    help='Verify the known bridge gap; this does not validate runtime execution.')
args = parser.parse_args()
bundle = args.bundle.resolve()
source = Path(__file__).resolve().parent
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))

with tempfile.TemporaryDirectory(prefix='neoclr-union-') as directory:
    work = Path(directory)
    for name in ('Probe.rvnproj', 'Errors.rvn', 'Main.rvn'):
        shutil.copyfile(source / name, work / name)
    compiled = work / 'compiled'
    compiled.mkdir()
    subprocess.run(['dotnet', str(bundle / 'raven-sdk/tools/rvnc/rvnc.dll'),
                    str(work / 'Probe.rvnproj'), '--no-project-restore',
                    '--configuration', 'Debug', '-o', str(compiled)], env=env, check=True)
    assembly = compiled / 'Probe.dll'
    subprocess.run(['dotnet', 'run', '--project', str(source / 'Shape.csproj'),
                    '--', str(assembly)], check=True)
    result = subprocess.run(['dotnet', str(bundle / 'tools/bridge/Probe.dll'),
                             '--import', str(assembly), str(bundle / 'demo/NeoCLR.CoreProbe.dll'),
                             str(work / 'imported')], capture_output=True, text=True)
    diagnostic = result.stdout + result.stderr
    if args.expect_import_rejection:
        expected = 'Unsupported Result profile type: System.Networking.Sockets.SocketError&'
        if result.returncode == 0 or expected not in diagnostic:
            raise SystemExit('Expected bridge rejection changed; investigate:\n' + diagnostic)
        print('Source compilation and CLI inspection succeeded. Known import rejection: ' + expected)
        print('No runtime execution or GC validation claimed.')
    else:
        print(diagnostic)
        result.check_returncode()
        print('Import succeeded; runtime semantics and GC still require separate validation.')

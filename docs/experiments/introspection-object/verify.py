#!/usr/bin/env python3
"""Build the TypeInfo Object contract sample using a matching development bundle."""
import argparse
import os
import re
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
parser.add_argument('--runner', type=Path, required=True, help='measure_async runner to check collection and reclamation')
args = parser.parse_args()
bundle = args.toolchain_root.resolve()
with tempfile.TemporaryDirectory(prefix='neoclr-introspection-object-') as folder:
    root = Path(folder)
    for name in ('Main.rvn', 'IntrospectionObject.rvnproj'):
        shutil.copyfile(HERE / name, root / name)
    env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
    build = subprocess.run(['dotnet', 'msbuild', str(root / 'IntrospectionObject.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=120)
    assert build.returncode == 0, build.stdout + build.stderr
    run = subprocess.run([str(args.runner.resolve()), str(root / 'bin/neoclr/Debug/App.neoil'), str(bundle / 'lib/System.neoil'), '96'], capture_output=True, text=True, timeout=120)
    assert run.returncode == 0, run.stdout + run.stderr
    assert run.stdout == (HERE / 'expected.txt').read_text(), repr(run.stdout)
    stats = {key: int(value) for key, value in re.findall(r'(\w+)=(\d+)', run.stderr)}
    assert stats['collections'] > 1 and stats['live'] == 0, stats
    print(run.stderr, end='')
    print(run.stdout, end='')

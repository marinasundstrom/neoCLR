#!/usr/bin/env python3
"""Build the Object equality sample using a matching development bundle."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
args = parser.parse_args()
bundle = args.toolchain_root.resolve()
with tempfile.TemporaryDirectory(prefix='neoclr-object-equality-') as folder:
    root = Path(folder)
    for name in ('Main.rvn', 'ObjectEquality.rvnproj'):
        shutil.copyfile(HERE / name, root / name)
    env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
    build = subprocess.run(['dotnet', 'msbuild', str(root / 'ObjectEquality.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=120)
    assert build.returncode == 0, build.stdout + build.stderr
    run = subprocess.run([str(bundle / 'bin/neoclr'), 'run', str(root / 'bin/neoclr/Debug/App.neoil'), '--system', str(bundle / 'lib/System.neoil')], capture_output=True, text=True, timeout=120)
    assert run.returncode == 0, run.stdout + run.stderr
    assert run.stdout == (HERE / 'expected.txt').read_text(), repr(run.stdout)
    assert not run.stderr, run.stderr
    print('Object identity, class equality and hash sample passed')

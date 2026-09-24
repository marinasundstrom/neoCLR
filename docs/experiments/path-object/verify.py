#!/usr/bin/env python3
"""Build the Path Object contract sample using a matching development bundle."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
parser.add_argument('--audit', action='store_true', help='Report current TypeInfo behavior; not a desired-contract test')
args = parser.parse_args()
bundle = args.toolchain_root.resolve()
with tempfile.TemporaryDirectory(prefix='neoclr-path-object-') as folder:
    root = Path(folder)
    for name in ('Main.rvn', 'PathObject.rvnproj'):
        shutil.copyfile(HERE / name, root / name)
    if args.audit:
        shutil.copyfile(HERE / 'Audit.rvn', root / 'Main.rvn')
    env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
    build = subprocess.run(['dotnet', 'msbuild', str(root / 'PathObject.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=120)
    assert build.returncode == 0, build.stdout + build.stderr
    run = subprocess.run([str(bundle / 'bin/neoclr'), 'run', str(root / 'bin/neoclr/Debug/App.neoil'), '--system', str(bundle / 'lib/System.neoil')], capture_output=True, text=True, timeout=120)
    assert run.returncode == 0, run.stdout + run.stderr
    if not args.audit:
        assert run.stdout == (HERE / 'expected.txt').read_text(), repr(run.stdout)
    assert not run.stderr, run.stderr
    print(run.stdout, end='')

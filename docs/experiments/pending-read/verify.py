"""Verify the queued pending-read cancellation contract and actual GC retention."""
import argparse
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', required=True, type=Path)
args = parser.parse_args()
bundle = args.toolchain_root.resolve()
with tempfile.TemporaryDirectory(prefix='neoclr-pending-read-') as folder:
    root = Path(folder)
    for name in ('PendingRead.rvn', 'Main.rvn', 'PendingRead.rvnproj'):
        shutil.copyfile(HERE / name, root / name)
    env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
    built = subprocess.run(['dotnet', 'msbuild', str(root / 'PendingRead.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=120)
    assert built.returncode == 0, built.stdout + built.stderr
    result = subprocess.run([str(bundle / 'bin/neoclr'), 'run', str(root / 'bin/neoclr/Debug/App.neoil'), '--system', str(bundle / 'lib/System.neoil'), '--gc-stats'], capture_output=True, text=True, timeout=60)
    assert result.returncode == 0, result.stdout + result.stderr
    assert result.stdout == (HERE / 'expected.txt').read_text(), result.stdout
    collections = re.search(r'collections=(\d+)', result.stderr)
    assert collections and int(collections[1]) > 0, result.stderr
    live = re.search(r'live=(\d+)', result.stderr)
    assert live and int(live[1]) == 0, result.stderr
    print(result.stdout, end='')
    print(result.stderr, end='')
    print('Pending-read Task cancellation and GC contract: passed')

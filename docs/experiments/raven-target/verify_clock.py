"""Compare the Raven/neoCLR local-clock sample with the host wall clock."""
import argparse
from datetime import datetime, timedelta, timezone
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('project', type=Path)
parser.add_argument('--raven', type=Path, required=True)
parser.add_argument('--runtime', type=Path, required=True)
args = parser.parse_args()
bridge = Path(__file__).resolve().parent
with tempfile.TemporaryDirectory(prefix='neoclr-clock-') as temporary:
    root = Path(temporary)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    shutil.copyfile(bridge / 'samples/library-clock.rvn', root / 'Main.rvn')
    start = time.time()
    result = subprocess.run([sys.executable, str(bridge / 'run_project.py'), str(root / 'Demo.rvnproj'),
                             '--raven', str(args.raven.resolve()), '--runtime', str(args.runtime.resolve())],
                            capture_output=True, text=True, timeout=90)
    end = time.time()
    if result.returncode:
        raise AssertionError(result.stdout + result.stderr)
    parts = [int(value) for value in result.stdout.splitlines()[-7:]]
    observed = datetime(*parts[:6], tzinfo=timezone(timedelta(seconds=parts[6])))
    if not start - 2 <= observed.timestamp() <= end + 2:
        raise AssertionError((observed.isoformat(), start, end))
    local = datetime.fromtimestamp(observed.timestamp()).astimezone()
    if local.utcoffset() != observed.utcoffset() or local.replace(tzinfo=None) != observed.replace(tzinfo=None):
        raise AssertionError((observed.isoformat(), local.isoformat()))
    print('Local calendar components and UTC offset match the host clock:', observed.isoformat())

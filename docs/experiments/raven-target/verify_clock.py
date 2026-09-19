"""Compare the Raven/neoCLR local-clock sample with the host wall clock."""
import argparse
from runner_options import add_toolchain_arguments, runner_arguments
from datetime import datetime
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('project', type=Path)
add_toolchain_arguments(parser)
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
                             *runner_arguments(args), '--runtime', str(args.runtime.resolve())],
                            capture_output=True, text=True, timeout=90)
    end = time.time()
    if result.returncode:
        raise AssertionError(result.stdout + result.stderr)
    parts = [int(value) for value in result.stdout.splitlines()[-6:]]
    observed = datetime(*parts)
    # LocalDateTime exposes calendar components, not an offset. Compare the host's
    # possible local seconds across the run, including either side of a DST fold.
    if not any(datetime.fromtimestamp(second) == observed
               for second in range(int(start) - 2, int(end) + 3)):
        raise AssertionError((observed.isoformat(), start, end))
    print('Local calendar components match the host clock:', observed.isoformat())

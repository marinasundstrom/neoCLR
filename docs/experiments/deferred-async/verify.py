"""Check a hoisted structured Result across pending await and collection."""
import argparse
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
parser.add_argument('--runner', type=Path, required=True)
args = parser.parse_args()
bundle = args.toolchain_root.resolve()
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
with tempfile.TemporaryDirectory(prefix='neoclr-deferred-async-') as folder:
    root = Path(folder)
    for name in ('Main.rvn', 'Probe.rvnproj'):
        shutil.copyfile(Path(__file__).with_name(name), root / name)
    build = subprocess.run(['dotnet', 'msbuild', str(root / 'Probe.rvnproj'), '-nologo', '-v:minimal',
                            '-t:NeoCLRImport'], env=env, capture_output=True, text=True, timeout=180)
    assert build.returncode == 0, build.stdout + build.stderr
    app = next((root / 'obj').glob('**/imported/App.neoil'))
    source = app.read_text()
    types = re.split(r'(?m)^\.type ', source)
    states = [t for t in types if '.implements System.Runtime.CompilerServices.IAsyncStateMachine\n' in t]
    assert states and all('.field deferred ' in t for t in states), source
    assert all('.field deferred ' not in t for t in types if t not in states), source
    run = subprocess.run([str(args.runner.resolve()), str(app), str(bundle / 'lib/System.neoil'), '96'],
                         capture_output=True, text=True, timeout=120)
    assert run.returncode == 0 and run.stdout == 'Retained error\n', run.stdout + run.stderr
    stats = {key: int(value) for key, value in re.findall(r'(\w+)=(\d+)', run.stderr)}
    assert stats['collections'] > 1 and stats['live'] == 0, stats
    print('Deferred async Result:', stats)

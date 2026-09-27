"""Compile and execute retained constructor execution through public APIs."""
import argparse
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
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
with tempfile.TemporaryDirectory(prefix='neoclr-attributes-') as folder:
    root = Path(folder)
    for name in ('Main.rvn', 'Routes.rvn', 'Construction.rvnproj'):
        shutil.copyfile(here / name, root / name)
    built = subprocess.run(['dotnet', 'msbuild', str(root / 'Construction.rvnproj'), '-nologo', '-v:minimal', '-t:NeoCLRImport'], env=env, capture_output=True, text=True, timeout=240)
    assert built.returncode == 0, built.stdout + built.stderr
    imported = next((root / 'obj').glob('**/imported/App.neoil'))
    result = subprocess.run([str(args.runner.resolve()), str(imported), str(bundle / 'lib/System.neoil'), '512', '100000000'], capture_output=True, text=True, timeout=180)
    assert result.returncode == 0, result.stdout + result.stderr
    assert result.stdout == 'Retained union constructor checks passed\n', result.stdout
    assert 'live=0' in result.stderr, result.stderr
    print(result.stdout + result.stderr)

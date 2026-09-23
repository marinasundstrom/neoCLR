"""Build the standalone Storage POC and check its output and native file effects."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', required=True, type=Path)
args = parser.parse_args()
bundle = args.toolchain_root.resolve()
with tempfile.TemporaryDirectory(prefix='neoclr-storage-poc-') as folder:
    root = Path(folder)
    for name in ('Main.rvn', 'StoragePoc.rvnproj'):
        shutil.copyfile(HERE / name, root / name)
    env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
    build = subprocess.run(['dotnet', 'msbuild', str(root / 'StoragePoc.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=120)
    assert build.returncode == 0, build.stdout + build.stderr
    work = root / 'work'
    (work / 'storage-demo/examples').mkdir(parents=True)
    result = subprocess.run([str(bundle / 'bin/neoclr'), 'run', str(root / 'bin/neoclr/Debug/App.neoil'), '--system', str(bundle / 'lib/System.neoil')], cwd=work, capture_output=True, text=True, timeout=60)
    assert result.returncode == 0, result.stdout + result.stderr
    assert result.stdout == (HERE / 'expected.txt').read_text(), result.stdout
    assert (work / 'storage-demo/message.txt').read_bytes() == 'Hello, värld!'.encode('utf-8')
    assert not (work / 'storage-demo/missing.txt').exists()
    assert not (work / 'message.txt').exists()
    print(result.stdout, end='')
    print('Standalone platform Storage POC: passed')

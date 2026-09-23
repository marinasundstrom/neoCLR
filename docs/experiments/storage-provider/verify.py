"""Compile the provider experiment and verify disk effects in an isolated directory."""
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
with tempfile.TemporaryDirectory(prefix='neoclr-storage-provider-') as folder:
    root = Path(folder)
    for name in ('Storage.rvn', 'Streams.rvn', 'ByteRoundTrip.rvn', 'Main.rvn', 'StorageExplorer.rvnproj'):
        shutil.copyfile(HERE / name, root / name)
    env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
    built = subprocess.run(['dotnet', 'msbuild', str(root / 'StorageExplorer.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=120)
    assert built.returncode == 0, built.stdout + built.stderr
    work = root / 'work'
    work.mkdir()
    # No sample output is written into the checkout or a user-selected directory.
    result = subprocess.run([str(bundle / 'bin/neoclr'), 'run', str(root / 'bin/neoclr/Debug/App.neoil'), '--system', str(bundle / 'lib/System.neoil')], cwd=work, capture_output=True, text=True, timeout=60)
    assert result.returncode == 0, result.stdout + result.stderr
    assert result.stdout == (HERE / 'expected.txt').read_text(), result.stdout
    assert (work / 'notes.txt').read_bytes() == 'Hello, värld!'.encode('utf-8')
    assert (work / 'stream.txt').read_bytes() == 'Hello, värld!'.encode('utf-8')
    assert not (root / 'outside.txt').exists(), 'Unexpected traversal output'
    print(result.stdout, end='')
    print('Same Raven workflow with disk and memory providers: passed')

    # A second normal SDK build checks wrapper errors separately from the product sample.
    shutil.copyfile(HERE / 'Contracts.rvn', root / 'Main.rvn')
    contracts = subprocess.run(['dotnet', 'msbuild', str(root / 'StorageExplorer.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=120)
    assert contracts.returncode == 0, contracts.stdout + contracts.stderr
    checked = subprocess.run([str(bundle / 'bin/neoclr'), 'run', str(root / 'bin/neoclr/Debug/App.neoil'), '--system', str(bundle / 'lib/System.neoil')], cwd=work, capture_output=True, text=True, timeout=60)
    assert checked.returncode == 0, checked.stdout + checked.stderr
    assert checked.stdout == 'Stream wrapper contracts: passed\n', checked.stdout
    assert (work / 'failed-write.bin').read_bytes() == b'', 'Rejected write modified disk'
    assert (work / 'stream.txt').read_bytes() == 'Hello, värld!'.encode('utf-8')
    print(checked.stdout, end='')

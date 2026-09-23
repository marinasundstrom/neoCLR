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
    for name in ('Storage.rvn', 'Path.rvn', 'Streams.rvn', 'ByteRoundTrip.rvn', 'Main.rvn', 'StorageExplorer.rvnproj'):
        shutil.copyfile(HERE / name, root / name)
    env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
    built = subprocess.run(['dotnet', 'msbuild', str(root / 'StorageExplorer.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=120)
    assert built.returncode == 0, built.stdout + built.stderr
    work = root / 'work'
    work.mkdir()
    (work / 'sandbox').mkdir()
    # No sample output is written into the checkout or a user-selected directory.
    result = subprocess.run([str(bundle / 'bin/neoclr'), 'run', str(root / 'bin/neoclr/Debug/App.neoil'), '--system', str(bundle / 'lib/System.neoil')], cwd=work, capture_output=True, text=True, timeout=60)
    assert result.returncode == 0, result.stdout + result.stderr
    assert result.stdout == (HERE / 'expected.txt').read_text(), result.stdout
    assert (work / 'sandbox/notes.txt').read_bytes() == 'Hello, värld!'.encode('utf-8')
    assert (work / 'sandbox/stream.txt').read_bytes() == 'Hello, värld!'.encode('utf-8')
    assert not (work / 'notes.txt').exists(), 'Logical root escaped provider mapping'
    assert not (work / 'stream.txt').exists(), 'Logical root escaped provider mapping'
    assert not (root / 'outside.txt').exists(), 'Unexpected traversal output'
    print(result.stdout, end='')
    print('Same Raven workflow with disk and memory providers: passed')

    # A second normal SDK build checks wrapper errors separately from the product sample.
    shutil.copyfile(HERE / 'Contracts.rvn', root / 'Main.rvn')
    contracts = subprocess.run(['dotnet', 'msbuild', str(root / 'StorageExplorer.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=120)
    assert contracts.returncode == 0, contracts.stdout + contracts.stderr
    checked = subprocess.run([str(bundle / 'bin/neoclr'), 'run', str(root / 'bin/neoclr/Debug/App.neoil'), '--system', str(bundle / 'lib/System.neoil')], cwd=work / 'sandbox', capture_output=True, text=True, timeout=60)
    assert checked.returncode == 0, checked.stdout + checked.stderr
    assert checked.stdout == 'Stream wrapper contracts: passed\n', checked.stdout
    assert (work / 'sandbox/failed-write.bin').read_bytes() == b'', 'Rejected write modified disk'
    assert (work / 'sandbox/stream.txt').read_bytes() == 'Hello, värld!'.encode('utf-8')
    print(checked.stdout, end='')

    shutil.copyfile(HERE / 'PathContracts.rvn', root / 'Main.rvn')
    paths = subprocess.run(['dotnet', 'msbuild', str(root / 'StorageExplorer.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=120)
    assert paths.returncode == 0, paths.stdout + paths.stderr
    parsed = subprocess.run([str(bundle / 'bin/neoclr'), 'run', str(root / 'bin/neoclr/Debug/App.neoil'), '--system', str(bundle / 'lib/System.neoil')], cwd=work / 'sandbox', capture_output=True, text=True, timeout=60)
    assert parsed.returncode == 0, parsed.stdout + parsed.stderr
    assert parsed.stdout == 'Validated Path and provider resolution: passed\n', parsed.stdout
    print(parsed.stdout, end='')

    shutil.copyfile(HERE / 'MemoryContracts.rvn', root / 'Main.rvn')
    memory = subprocess.run(['dotnet', 'msbuild', str(root / 'StorageExplorer.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=120)
    assert memory.returncode == 0, memory.stdout + memory.stderr
    coherent = subprocess.run([str(bundle / 'bin/neoclr'), 'run', str(root / 'bin/neoclr/Debug/App.neoil'), '--system', str(bundle / 'lib/System.neoil')], cwd=work / 'sandbox', capture_output=True, text=True, timeout=60)
    assert coherent.returncode == 0, coherent.stdout + coherent.stderr
    assert coherent.stdout == 'Coherent memory storage contracts: passed\n', coherent.stdout
    print(coherent.stdout, end='')

    shutil.copyfile(HERE / 'LookupContracts.rvn', root / 'Main.rvn')
    lookup = subprocess.run(['dotnet', 'msbuild', str(root / 'StorageExplorer.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=120)
    assert lookup.returncode == 0, lookup.stdout + lookup.stderr
    found = subprocess.run([str(bundle / 'bin/neoclr'), 'run', str(root / 'bin/neoclr/Debug/App.neoil'), '--system', str(bundle / 'lib/System.neoil')], cwd=work / 'sandbox', capture_output=True, text=True, timeout=60)
    assert found.returncode == 0, found.stdout + found.stderr
    assert found.stdout == 'Typed disk and memory lookup contracts: passed\n', found.stdout
    print(found.stdout, end='')

    (work / 'sandbox/context').mkdir()
    (work / 'sandbox/context/nested').mkdir()
    shutil.copyfile(HERE / 'DirectoryContracts.rvn', root / 'Main.rvn')
    directory = subprocess.run(['dotnet', 'msbuild', str(root / 'StorageExplorer.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=120)
    assert directory.returncode == 0, directory.stdout + directory.stderr
    resolved = subprocess.run([str(bundle / 'bin/neoclr'), 'run', str(root / 'bin/neoclr/Debug/App.neoil'), '--system', str(bundle / 'lib/System.neoil')], cwd=work / 'sandbox', capture_output=True, text=True, timeout=60)
    assert resolved.returncode == 0, resolved.stdout + resolved.stderr
    assert resolved.stdout == 'Relative directory lookup contracts: passed\n', resolved.stdout
    print(resolved.stdout, end='')

    # Ordinary callers cannot construct an unvalidated Path or mutate its spelling.
    for source, diagnostic in [
        ('namespace StorageExperiment\nfunc Main() { let path = Path("../bypass") }', 'inaccessible'),
        ('namespace StorageExperiment\nfunc Main() { let path = RequirePath("ok")\npath.Text = "../bypass" }', 'read-only'),
    ]:
        (root / 'Main.rvn').write_text(source)
        rejected = subprocess.run(['dotnet', 'msbuild', str(root / 'StorageExplorer.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=120)
        assert rejected.returncode != 0, 'Path invariant bypass compiled'
        assert diagnostic in (rejected.stdout + rejected.stderr).lower(), rejected.stdout + rejected.stderr
    print('Path constructor and immutable text: rejected invalid callers')

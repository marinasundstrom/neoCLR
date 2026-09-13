"""Run the saved-source file API against isolated files and verify actual bytes."""
import argparse
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
from runner_options import add_toolchain_arguments, runner_arguments

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('project', type=Path)
add_toolchain_arguments(parser)
parser.add_argument('--runtime', required=True, type=Path)
args = parser.parse_args()
bridge = Path(__file__).resolve().parent
with tempfile.TemporaryDirectory(prefix='neoclr-file-project-') as temporary:
    root = Path(temporary)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    source = (bridge / 'samples/library-files.rvn').read_text()
    (root / 'Main.rvn').write_text(source)
    command = [sys.executable, str(bridge / 'run_project.py'), str(root / 'Demo.rvnproj'),
               *runner_arguments(args), '--runtime', str(args.runtime.resolve())]
    result = subprocess.run(command, cwd=root, capture_output=True, text=True, timeout=90)
    expected = 'Written\nCompleted\nHello, värld!\nWrite too large\nHello, värld!\nRead too large\n'
    if result.returncode or not result.stdout.endswith(expected):
        raise AssertionError(result.stdout + result.stderr)
    if (root / 'neoclr-file-demo.txt').read_bytes() != 'Hello, värld!'.encode():
        raise AssertionError('Rejected write changed the file bytes')
    (root / 'invalid.bin').write_bytes(b'\xff')
    source = source[:source.index('func Main()')] + '''func Main() {
        ShowRead(Load("missing.txt", 64))
        ShowRead(Load("invalid.bin", 64))
        ShowRead(Load("invalid.bin", -1))
    }'''
    (root / 'Main.rvn').write_text(source)
    result = subprocess.run(command, cwd=root, capture_output=True, text=True, timeout=90)
    if result.returncode or not result.stdout.endswith('Not found\nInvalid UTF-8\nInvalid limit\n'):
        raise AssertionError(result.stdout + result.stderr)
print('File round-trip bytes, non-destructive rejected write, missing/UTF-8/limit Results passed')

"""Verify Enum helper overloads through an isolated Raven SDK application."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bundle', required=True, type=Path)
parser.add_argument('--runner', required=True, type=Path)
args = parser.parse_args()
bundle = args.bundle.resolve()
here = Path(__file__).resolve().parent
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
with tempfile.TemporaryDirectory(prefix='neoclr-enum-helpers-') as folder:
    root = Path(folder)
    shutil.copyfile(here / 'Enums.rvnproj', root / 'Enums.rvnproj')
    shutil.copyfile(here / 'Main.rvn', root / 'Main.rvn')
    def build():
        return subprocess.run(['dotnet', 'msbuild', str(root / 'Enums.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=180)
    result = build()
    assert result.returncode == 0, result.stdout + result.stderr
    command = [str(args.runner.resolve()), str(root / 'bin/neoclr/Debug/App.neoil'), str(bundle / 'lib/System.neoil'), '64', '1000000']
    result = subprocess.run(command, capture_output=True, text=True, timeout=60)
    assert result.returncode == 0, result.stdout + result.stderr
    assert result.stdout == 'Enum names, typed values, discovery and formatting: passed\n', result.stdout
    print(result.stdout, end='')
    print(result.stderr, end='')
    for method in ('GetNames', 'GetValues'):
        for argument in ('int', 'string'):
            (root / 'Main.rvn').write_text(f'import System.*\nfunc Main() {{ let invalid = Enum.{method}<{argument}>() }}\n')
            result = build()
            assert result.returncode != 0, f'{method} admitted {argument}'
            assert 'error RAV' in result.stdout + result.stderr, result.stdout + result.stderr
    print('Non-enum type arguments: rejected by compiler')
    (root / 'Main.rvn').write_text('import System.*\nfunc Main() { let value: Object = 42\n_ = Enum.GetValues(value.GetType()) }\n')
    result = build()
    assert result.returncode == 0, result.stdout + result.stderr
    result = subprocess.run(command, capture_output=True, text=True, timeout=60)
    assert result.returncode != 0, 'Non-enum TypeInfo accepted'
    assert 'enum reflection requires an enum type' in result.stderr, result.stdout + result.stderr
    print('Non-enum TypeInfo: runtime fault')

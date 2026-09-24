#!/usr/bin/env python3
"""Build the record and HashCode sample using a matching development bundle."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
args = parser.parse_args()
bundle = args.toolchain_root.resolve()
with tempfile.TemporaryDirectory(prefix='neoclr-records-') as folder:
    root = Path(folder)
    for name in ('Main.rvn', 'Records.rvnproj', 'Nested.rvn', 'Nested.rvnproj', 'Defaults.rvn', 'Defaults.rvnproj'):
        shutil.copyfile(HERE / name, root / name)
    env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
    for project, expected in [('Records.rvnproj', 'expected.txt'), ('Nested.rvnproj', 'nested-expected.txt'), ('Defaults.rvnproj', 'defaults-expected.txt')]:
        build = subprocess.run(['dotnet', 'msbuild', str(root / project), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=120)
        assert build.returncode == 0, build.stdout + build.stderr
        run = subprocess.run([str(bundle / 'bin/neoclr'), 'run', str(root / 'bin/neoclr/Debug/App.neoil'), '--system', str(bundle / 'lib/System.neoil')], capture_output=True, text=True, timeout=120)
        assert run.returncode == 0, run.stdout + run.stderr
        assert run.stdout == (HERE / expected).read_text(), repr(run.stdout)
        assert not run.stderr, run.stderr
    for declaration in ('record class Unsupported(Name: string?)', 'record class Unsupported(Value: object)', 'record class Unsupported(Value: int?)', 'record struct Point(X: int)\nrecord class Unsupported(Value: Point?)'):
        (root / 'Main.rvn').write_text('import System.*\n' + declaration + '\nfunc Main() { }\n')
        rejected = subprocess.run(['dotnet', 'msbuild', str(root / 'Records.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=120)
        assert rejected.returncode != 0 and 'RAVT004' in rejected.stdout + rejected.stderr, rejected.stdout + rejected.stderr
    (root / 'Main.rvn').write_text('import System.*\nfunc Ignore(value: string?) { }\nfunc Main() { Ignore(null) }\n')
    rejected = subprocess.run(['dotnet', 'msbuild', str(root / 'Records.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=120)
    assert rejected.returncode != 0 and 'FaultNull' in rejected.stdout + rejected.stderr, rejected.stdout + rejected.stderr
    (root / 'Main.rvn').write_text("""import System.*
record struct Point(X: int, Y: int)
class Writer {
    func Replace(ref value: Point) {
        value = Point(9, 9)
    }
}
func Main() {
    var point = Point(1, 2)
    Writer().Replace(&point)
}
""")
    rejected = subprocess.run(['dotnet', 'msbuild', str(root / 'Records.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=120)
    assert rejected.returncode != 0 and 'declared instance output' in rejected.stdout + rejected.stderr, rejected.stdout + rejected.stderr
    print('Record identity, component equality, display, deconstruction and HashCode sample passed')

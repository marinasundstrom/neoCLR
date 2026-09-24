#!/usr/bin/env python3
"""Compile String sequence construction and reject direct Count/mutation."""
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
here = Path(__file__).resolve().parent
bundle = args.toolchain_root.resolve()
with tempfile.TemporaryDirectory(prefix='neoclr-string-sequence-') as directory:
    root = Path(directory)
    for name in ('Main.rvn', 'StringSequence.rvnproj'):
        shutil.copyfile(here / name, root / name)
    env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
    def build():
        return subprocess.run(['dotnet', 'msbuild', str(root / 'StringSequence.rvnproj'),
                               '-nologo', '-v:minimal'], env=env, capture_output=True,
                              text=True, timeout=120)
    result = build()
    assert result.returncode == 0, result.stdout + result.stderr
    run = subprocess.run([str(args.runner.resolve()), str(root / 'bin/neoclr/Debug/App.neoil'),
                          str(bundle / 'lib/System.neoil'), '256'],
                         capture_output=True, text=True, timeout=120)
    assert run.returncode == 0, run.stdout + run.stderr
    assert run.stdout == (here / 'expected.txt').read_text(), run.stdout
    assert 'live=0' in run.stderr, run.stderr
    print(run.stdout, end='')
    print(run.stderr, end='')
    for name, body in [
        ('Count is interface-only', 'let count = "Foo".Count\nif count != 3 { System.Fault("count") }'),
        ('String indexer is read-only', 'var text = "Foo"\ntext[0] = \'B\''),
        ('Constructor requires characters', 'let text = String(42)'),
    ]:
        (root / 'Main.rvn').write_text('import System.*\nfunc Main() {\n' + body + '\n}\n')
        result = build()
        diagnostics = result.stdout + result.stderr
        assert result.returncode != 0 and 'error RAV' in diagnostics, (name, diagnostics)
        assert 'Unhandled exception' not in diagnostics, diagnostics
        print(name + ': rejected')

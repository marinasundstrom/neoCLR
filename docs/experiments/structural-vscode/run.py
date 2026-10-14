#!/usr/bin/env python3
"""Build and run one sample using only the adjacent experimental toolchain."""
import argparse
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('sample', choices=['Unions', 'Intersections', 'ExternalBounds'])
args = parser.parse_args()

def build(name):
    subprocess.run(['dotnet', str(root / 'tools/rvnc/rvnc.dll'), 'neoclr', '--project', str(root / name / f'{name}.rvnproj')], cwd=root, check=True)

if args.sample == 'ExternalBounds':
    build('Contracts')
build(args.sample)
library = root / 'library'
command = [str(root / 'runtime/neoclr'), 'run', str(root / args.sample / 'bin/neoclr' / f'{args.sample}.dll'), '--system', str(library / 'System.runtime.neox'), '--object-root', str(library / 'System.Runtime.dll'), '--module', str(library / 'System.Runtime.dll'), '--execution', 'interpreter', '--instructions', '100000000']
if args.sample == 'ExternalBounds':
    command += ['--module', str(root / 'Contracts/bin/neoclr/Contracts.dll')]
subprocess.run(command, cwd=root, check=True)

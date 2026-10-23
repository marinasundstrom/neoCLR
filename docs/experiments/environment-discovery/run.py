#!/usr/bin/env python3
"""Compile and run the isolated Raven/.NET API prototype; retain exact evidence."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--compiler', required=True, type=Path)
parser.add_argument('--raven-core', required=True, type=Path)
args = parser.parse_args()
source = Path(__file__).resolve().parent
root = source.parents[2]
output = root / 'target/environment-discovery-prototype'
output.mkdir(parents=True, exist_ok=True)
compiler = args.compiler.resolve()
core = args.raven_core.resolve()
files = [source / name for name in ('Model.rvn', 'DotNetHost.rvn', 'Main.rvn')]
assembly = output / 'Prototype.dll'
command = ['dotnet', str(compiler), '--framework', 'net10.0', '--raven-core', str(core),
           *map(str, files), '-o', str(assembly)]
build = subprocess.run(command, text=True, capture_output=True)
print(build.stdout + build.stderr, end='')
if build.returncode:
    raise SystemExit(build.returncode)
shutil.copy2(core, output / 'Raven.Core.dll')
run = subprocess.run(['dotnet', str(assembly)], text=True, capture_output=True)
print(run.stdout + run.stderr, end='')
passed = run.returncode == 0 and 'PASS environment prototype' in run.stdout
report = {
    'scope': 'Raven on .NET 10; not native neoCLR validation',
    'command': command,
    'sha256': {str(p): hashlib.sha256(p.read_bytes()).hexdigest()
               for p in [compiler, core, *files, assembly]},
    'build_stdout': build.stdout, 'build_stderr': build.stderr,
    'exit_code': run.returncode, 'stdout': run.stdout, 'stderr': run.stderr,
    'passed': passed,
}
(output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
raise SystemExit(0 if passed else 1)

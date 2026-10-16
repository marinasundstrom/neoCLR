#!/usr/bin/env python3
"""Run the shared operator consumer against the ordinary .NET compiler target."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument('--compiler', type=Path, required=True)
parser.add_argument('--core', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--source', type=Path, help='Alternate focused operator consumer')
args = parser.parse_args()
compiler, core, output = (p.resolve() for p in (args.compiler, args.core, args.output))
output.mkdir(parents=True, exist_ok=True)
source = args.source.resolve() if args.source else Path(__file__).resolve().with_name('Main.rvn')
assembly = output / 'OperatorMatrix.dll'
report = {'passed': False, 'commands': []}
for label, command, expected in (
    ('compile', ['dotnet', str(compiler), str(source), '--framework', 'net11.0',
                 '--raven-core', str(core), '-o', str(assembly)], 0),
    ('execute', ['dotnet', str(assembly)], 42),
):
    if label == 'execute':
        shutil.copy2(core, output / 'Raven.Core.dll')
    result = subprocess.run(command, capture_output=True, text=True, timeout=300)
    log = output / (label + '.log')
    log.write_text(result.stdout + result.stderr)
    report['commands'].append({'label': label, 'command': command,
                               'exitCode': result.returncode,
                               'logSha256': hashlib.sha256(log.read_bytes()).hexdigest()})
    (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(label, result.returncode, flush=True)
    if result.returncode != expected:
        raise RuntimeError(log.read_text())
report['files'] = {str(p): hashlib.sha256(p.read_bytes()).hexdigest()
                   for p in (source, Path(__file__).resolve(), compiler,
                             compiler.parent / 'Raven.CodeAnalysis.dll', core, assembly)}
report['passed'] = True
(output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')

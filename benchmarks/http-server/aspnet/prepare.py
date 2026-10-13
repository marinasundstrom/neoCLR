#!/usr/bin/env python3
"""Build the identical terminal Kestrel app as .NET JIT and Native AOT."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, required=True)
a = parser.parse_args()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
report = dict(prepared=False, platform=platform.platform(), builds=[], inputs={}, outputs={})
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
for name in ['Program.cs', 'HttpComparison.csproj', 'global.json', 'prepare.py']:
    report['inputs'][str(ROOT/name)] = sha(ROOT/name)
try:
    report['dotnetInfo'] = subprocess.check_output(['dotnet', '--info'], cwd=ROOT, text=True)
    for mode in ['jit', 'aot']:
        command = ['dotnet', 'publish', '-c', 'Release', '-r', 'osx-arm64',
                   '--self-contained', 'true' if mode == 'aot' else 'false',
                   '-p:PublishAot=' + ('true' if mode == 'aot' else 'false'), '-o', str(out/mode)]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=600)
        report['builds'].append(dict(command=command, cwd=str(ROOT), exitCode=result.returncode,
                                    stdout=result.stdout, stderr=result.stderr))
        if result.returncode:
            raise RuntimeError('Publish failed: ' + mode)
    for p in out.rglob('*'):
        if p.is_file():
            report['outputs'][str(p)] = sha(p)
    report['commands'] = {'jit': ['dotnet', str(out/'jit/HttpComparison.dll')],
                          'aot': [str(out/'aot/HttpComparison')]}
    for p, digest in report['inputs'].items():
        assert sha(Path(p)) == digest, p
    report['prepared'] = True
finally:
    (out/'build.json').write_text(json.dumps(report, indent=2)+'\n')
print(out/'build.json')

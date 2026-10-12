#!/usr/bin/env python3
"""Compile and run forward async result types in both source orders."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--bundle', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--repetitions', type=int, default=3)
a = p.parse_args()
if a.repetitions < 1:
    p.error('--repetitions must be positive')
root = a.bundle.resolve()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
source = Path(__file__).resolve().parent
compiler = root / 'sdk/tools/rvnc/rvnc.dll'
runtime = root / 'bin/neoclr'
report = dict(passed=False, compilerSha256=hashlib.sha256(compiler.read_bytes()).hexdigest(),
              codeAnalysisSha256=hashlib.sha256((compiler.parent / 'Raven.CodeAnalysis.dll').read_bytes()).hexdigest(),
              commands=[])
try:
    for attempt in range(a.repetitions * 2):
        reverse = bool(attempt % 2)
        folder = out / (str(attempt) + ('-types-first' if reverse else '-function-first'))
        folder.mkdir()
        for filename, target in [('Main.rvn', 'Z.rvn' if reverse else 'A.rvn'),
                                 ('Types.rvn', 'A.rvn' if reverse else 'Z.rvn')]:
            (folder / target).write_text((source / filename).read_text())
        props = root / 'lib/NeoCLR.ClassLibrary.props'
        from xml.sax.saxutils import quoteattr
        (folder / 'App.rvnproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><Import Project=' + quoteattr(str(props)) + '/><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><AssemblyName>App</AssemblyName></PropertyGroup></Project>')
        for command, expected in [
            (['dotnet', compiler, 'neoclr', '--project', folder / 'App.rvnproj'], None),
            ([runtime, 'run', folder / 'bin/neoclr/App.dll'], 'Forward async return passed\n')]:
            command = list(map(str, command))
            result = subprocess.run(command, cwd=out, capture_output=True, text=True, timeout=180)
            report['commands'].append(dict(command=command, exitCode=result.returncode,
                                           stdout=result.stdout, stderr=result.stderr))
            assert result.returncode == 0, result.stderr
            if expected is not None:
                assert result.stdout == expected and not result.stderr, (result.stdout, result.stderr)
    report['passed'] = True
finally:
    (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')

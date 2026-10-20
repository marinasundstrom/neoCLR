#!/usr/bin/env python3
"""Validate native record component value semantics and an unsupported-component diagnostic."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('bundle', type=Path)
parser.add_argument('application', type=Path)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
bundle = args.bundle.resolve()
application = args.application.resolve()
rows = []
common = ['--system', str(bundle / 'lib/System.runtime.neox'),
          '--object-root', str(bundle / 'lib/System.Runtime.dll'),
          '--module', str(bundle / 'lib/System.Runtime.dll')]
for engine in ['interpreter', 'jit', 'aot']:
    if engine == 'aot':
        command = [str(application / 'app')]
    else:
        command = [str(bundle / 'bin/neoclr'), 'run', str(application / 'app.dll'),
                   '--execution', engine, *common]
        if engine == 'interpreter':
            command += ['--instructions', '100000000']
        else:
            command += ['--jit-compiler', str(bundle / 'bin/neoclr-codegen'),
                        '--jit-host', str(bundle / 'bin/neoclr-jit-host.dylib')]
    result = subprocess.run(command, text=True, capture_output=True, timeout=120)
    passed = result.returncode == 0 and result.stdout.strip() == 'PASS record components' and not result.stderr
    rows.append(dict(case=engine, command=command, exitCode=result.returncode,
                     stdout=result.stdout, stderr=result.stderr, passed=passed))
    print(f'{engine}: {"PASS" if passed else "FAIL"}', flush=True)
compiler = bundle / 'sdk/tools/rvnc/rvnc.dll'
for case in ['MissingHash', 'MissingContract']:
    command = ['dotnet', str(compiler), 'neoclr', '--project',
               str(Path(__file__).resolve().with_name(case + '.rvnproj'))]
    result = subprocess.run(command, env={**os.environ, 'NeoClrBundleRoot': str(bundle)},
                            text=True, capture_output=True, timeout=120)
    passed = result.returncode != 0 and 'record value semantics require supported component equality and hashing' in result.stdout + result.stderr
    rows.append(dict(case=case, command=command, exitCode=result.returncode,
                     stdout=result.stdout, stderr=result.stderr, passed=passed))
    print(f'{case}: {"PASS" if passed else "FAIL"}', flush=True)
paths = [application / 'app', application / 'app.dll', compiler,
         compiler.with_name('Raven.CodeAnalysis.NeoClr.dll'), bundle / 'bin/neoclr',
         bundle / 'bin/neoclr-codegen', bundle / 'lib/System.Runtime.dll']
report = dict(passed=all(row['passed'] for row in rows), executions=rows,
              hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths})
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(json.dumps(report, indent=2) + '\n')
raise SystemExit(0 if report['passed'] else 1)

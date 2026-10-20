#!/usr/bin/env python3
"""Validate Persistent memory region contracts across execution engines."""
import argparse
import hashlib
import json
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
    passed = result.returncode == 0 and result.stdout.strip() == 'Memory region contracts: passed' and not result.stderr
    rows.append(dict(case=engine, command=command, exitCode=result.returncode,
                     stdout=result.stdout, stderr=result.stderr, passed=passed))
    print(f'{engine}: {"PASS" if passed else "FAIL"}', flush=True)
compiler = bundle / 'sdk/tools/rvnc/rvnc.dll'
paths = [application / 'app', application / 'app.dll', compiler,
         compiler.with_name('Raven.CodeAnalysis.NeoClr.dll'), bundle / 'bin/neoclr',
         bundle / 'bin/neoclr-codegen', bundle / 'lib/System.Runtime.dll']
report = dict(passed=all(row['passed'] for row in rows), executions=rows,
              hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths})
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(json.dumps(report, indent=2) + '\n')
raise SystemExit(0 if report['passed'] else 1)

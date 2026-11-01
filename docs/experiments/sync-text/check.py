#!/usr/bin/env python3
"""Run the mixed text cursor contract against one matching three-engine bundle."""
import argparse
import json
import hashlib
import sys
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'benchmarks/framework'))
from process import RETENTION_WARNING

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('bundle', type=Path)
parser.add_argument('application', type=Path)
parser.add_argument('--output', required=True, type=Path)
args = parser.parse_args()
bundle = args.bundle.resolve()
application = args.application.resolve()
common = ['--system', str(bundle / 'lib/System.runtime.neox'), '--object-root', str(bundle / 'lib/System.Runtime.dll')]
for name in ['System.Runtime', 'System.Data', 'System.Networking', 'System.Web', 'System.Data.PostgreSql']:
    common += ['--module', str(bundle / 'lib' / (name + '.dll'))]
rows = []
for engine in ['interpreter', 'jit', 'aot']:
    if engine == 'aot':
        command = [str(application / 'app')]
    else:
        command = [str(bundle / 'bin/neoclr'), 'run', str(application / 'app.dll'), '--execution', engine, *common]
        if engine == 'interpreter':
            command += ['--instructions', '100000000']
        else:
            command += ['--jit-compiler', str(bundle / 'bin/neoclr-codegen'), '--jit-host', str(bundle / 'bin/neoclr-jit-host.dylib')]
    result = subprocess.run(command, input='éabc\ntail'.encode(), capture_output=True, timeout=120)
    passed = result.returncode == 0 and result.stdout == b'PASS Synchronous text wrappers preserve cursor delimiters limits and ownership\nTests: 1, passed: 1, failed: 0, skipped: 0\n' and result.stderr in ((b'', RETENTION_WARNING) if engine == 'jit' else (b'',))
    rows.append(dict(engine=engine, command=command, exitCode=result.returncode, passed=passed,
                     stdout=result.stdout.decode(errors='replace'), stderr=result.stderr.decode(errors='replace')))
    print(f'{engine}: {"PASS" if passed else "FAIL"}', flush=True)
args.output.parent.mkdir(parents=True, exist_ok=True)
paths = [Path(__file__), *Path(__file__).parent.glob('*.rvn'),
         *Path(__file__).parent.glob('*.rvnproj'),
         *(ROOT / 'tools/testing/NeoCLR.Testing/NeoClr/Testing').glob('*.rvn'),
         bundle / 'manifest.json', application / 'app', application / 'app.dll']
args.output.write_text(json.dumps(dict(passed=all(row['passed'] for row in rows), executions=rows,
    hashes={str(p.resolve()): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}), indent=2) + '\n')
raise SystemExit(0 if all(row['passed'] for row in rows) else 1)

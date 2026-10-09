#!/usr/bin/env python3
"""Execute an already-built storage probe in native and interpreted modes."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[3]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--build', type=Path, required=True)
parser.add_argument('--bundle', type=Path, required=True)
parser.add_argument('--interpreter', type=Path, required=True)
args = parser.parse_args()
build, bundle, interpreter = (p.resolve() for p in (args.build, args.bundle, args.interpreter))
lib = bundle / 'lib'
catalog = json.loads((lib / 'bundle.json').read_text())
exe = build / ('app.exe' if platform.system() == 'Windows' else 'app')
commands = [
    ('native', [exe]),
    ('interpreter', [interpreter, 'run', build / 'app.dll', '--system',
                     lib / catalog['runtimeSeed'],
                     *[a for name in catalog['assemblyNames']
                       for a in ('--module', lib / (name + '.dll'))],
                     '--object-root', lib / 'System.Runtime.dll',
                     '--instructions', '100000000']),
]
report = {'passed': False, 'platform': platform.platform(), 'results': []}
for mode, command in commands:
    result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True,
                            text=True, timeout=120)
    row = dict(mode=mode, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr)
    report['results'].append(row)
    print(json.dumps(row), flush=True)
paths = [Path(__file__).resolve(), Path(__file__).parent / 'raven/Main.rvn',
         Path(__file__).parent / 'raven/Native.rvnproj', exe, build / 'app.dll',
         build / 'build.json', interpreter, bundle / 'manifest.json']
report['hashes'] = {str(p.relative_to(ROOT) if p.is_relative_to(ROOT) else p):
                    hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
report['passed'] = all(r['exitCode'] == 0 and not r['stderr'] and
                       r['stdout'] == 'set storage passed\n'
                       for r in report['results'])
(build / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
if not report['passed']:
    raise SystemExit('Storage probe failed; see validation.json')

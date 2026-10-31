"""Check the compiled primitive server with a real HTTP consumer in three modes."""
import argparse
import json
from pathlib import Path
import runpy

ROOT = Path(__file__).resolve().parents[3]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bundle', required=True, type=Path)
parser.add_argument('--build', required=True, type=Path)
parser.add_argument('--output', required=True, type=Path)
parser.add_argument('--runtime', required=True, type=Path)
parser.add_argument('--codegen', required=True, type=Path)
args = parser.parse_args()
bundle, build = args.bundle.resolve(), args.build.resolve()
manifest = json.loads((bundle / 'manifest.json').read_text())
lib = bundle / 'lib'
measure = runpy.run_path(str(ROOT / 'scripts/benchmark-neo-web.py'))['measure']
base = [str(args.runtime.resolve()), 'run', str(build / 'app.dll'), '--system', str(lib / 'System.runtime.neox'),
        '--object-root', str(lib / 'System.Runtime.dll')]
for module in manifest['modules']:
    base += ['--module', str(bundle / module)]
commands = {
    'native': [str(build / 'app')],
    'interpreter': base + ['--execution', 'interpreter', '--instructions', '100000000'],
    'jit': base + ['--execution', 'jit', '--jit-host', str(bundle / 'bin/neoclr-jit-host.dylib'),
                   '--jit-compiler', str(args.codegen.resolve())],
}
report = dict(passed=False, note='Correctness smoke, not a throughput comparison.', cases=[])
try:
    for mode, command in commands.items():
        result = measure(command, 3, 1, '/', b'42', None)
        report['cases'].append(dict(mode=mode, command=command, **result))
    report['passed'] = True
finally:
    args.output.write_text(json.dumps(report, indent=2) + '\n')
print('Primitive HTTP smoke: PASS')

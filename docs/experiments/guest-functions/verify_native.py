"""Qualify explicit function retention on the pinned macOS ARM64 development host."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
for name in ('bundle', 'compiler', 'backend', 'output'):
    parser.add_argument('--' + name, type=Path, required=True)
args = parser.parse_args()
bundle, compiler, backend, output = (getattr(args, n).resolve() for n in ('bundle', 'compiler', 'backend', 'output'))
output.mkdir(parents=True, exist_ok=False)
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
inputs = [Path(__file__), HERE / 'verify.py', HERE / 'Main.rvn', HERE / 'GuestFunctions.rvnproj',
          ROOT / 'runtime/raven/tests/framework/TestAttribute.rvn', compiler, backend,
          bundle / 'lib/System.Runtime.dll', bundle / 'bin/neoclr-jit-host.dylib',
          ROOT / 'docs/experiments/runtime-async/generic-cleanup/mac-shared-gc-validation.json']
report = dict(passed=False, platform=platform.platform(), commands=[], inputs={str(p): sha(p) for p in inputs})


def run(label, command, expected=0, stdout=None):
    result = subprocess.run(list(map(str, command)), cwd=ROOT, env=dict(os.environ),
                            capture_output=True, text=True, timeout=600)
    row = dict(label=label, command=result.args, exitCode=result.returncode, stderr=result.stderr)
    if len(result.stdout) < 4000:
        row['stdout'] = result.stdout
    else:
        row['stdoutSha256'] = hashlib.sha256(result.stdout.encode()).hexdigest()
    report['commands'].append(row)
    (output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    assert result.returncode == expected, result.stdout[-1000:] + result.stderr
    if stdout is not None:
        assert result.stdout == stdout, result.stdout
    return result


run('interpreter', [sys.executable, HERE / 'verify.py', '--bundle', bundle, '--compiler', compiler,
                   '--runner', bundle / 'bin/neoclr', '--execution', 'interpreter', '--output', output / 'interpreter'])
reference = json.loads((output / 'interpreter/report.json').read_text())
image = Path(reference['commands'][1]['command'][2])
report['imageSha256'] = sha(image)
# Reuse a qualified native toolchain/host recipe, replacing app and bundle paths.
recipe = json.loads(inputs[-1].read_text())
compile_command = next(x['command'] for x in recipe['commands'] if x['label'] == 'use-state-machine-native').copy()
compile_command[0], compile_command[2], compile_command[4] = str(backend), str(image), str(output / 'app.o')
compile_command = [str(bundle / x.split('/bundle/', 1)[1]) if '/bundle/' in x else x for x in compile_command]
inspection = run('inspect', [backend, '--inspect', image, '@entry', '--closed-world'] + compile_command[5:])
assemblies = json.loads(inspection.stdout)['assemblies']
identity = next(a['full_name'] for a in assemblies if a['name'] == 'GuestFunctions')
roots = dict(schemaVersion=5, types=[], moduleCatalogs=[identity], moduleFunctions=[
    dict(assembly=identity, name=name, customAttributes=True) for name in ('Example.Tests', 'Example.Empty')])
root_path = output / 'roots.json'
root_path.write_text(json.dumps(roots, indent=2) + '\n')
report['retention'] = roots
compile_command += ['--reflection-roots', str(root_path)]
run('aot-compile', compile_command)
link = next(x['command'] for x in recipe['commands'] if x['label'] == 'use-state-machine-link')
link = [str(output / 'app.o') if x.endswith('/use-state-machine.o') else str(output / 'app') if x.endswith('/use-state-machine') else str(ROOT / 'tools/native/console-host.c') if x.endswith('/benchmarks/native-web/http-host.c') else x for x in link]
for item in link:
    p = Path(item)
    if p.is_file() and p != output / 'app.o':
        report['inputs'][str(p)] = sha(p)
run('aot-link', link)
expected = 'Guest module function discovery passed\n'
run('aot-run', [output / 'app'], stdout=expected)
jit = compile_command.copy()
jit[4] = str(bundle / 'bin/neoclr-jit-host.dylib')
jit += ['--jit-run']
run('jit-run', jit, stdout=expected)
for label, config, message in [
    ('no-functions', dict(roots, moduleFunctions=[]), 'native module function metadata was not retained'),
    ('no-attributes', dict(roots, moduleFunctions=[dict(row, customAttributes=False) for row in roots['moduleFunctions']]), 'native module-member custom attribute metadata was not retained'),
    ('no-catalog', dict(roots, moduleCatalogs=[]), 'native assembly module catalog was not retained'),
]:
    path = output / (label + '.json')
    path.write_text(json.dumps(config, indent=2) + '\n')
    command = jit.copy()
    command[command.index('--reflection-roots') + 1] = str(path)
    result = run(label, command, expected=1, stdout='')
    assert message in result.stderr, result.stderr
assert all(sha(Path(p)) == digest for p, digest in report['inputs'].items())
assert sha(image) == report['imageSha256']
report.update(passed=True, scope='Interpreter, macOS ARM64 JIT/AOT module-function metadata discovery and method attribute data. Explicit negative retention cases. No function invocation or automatic test registration.')
(output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
print('Module function discovery: interpreter, JIT, AOT and retention faults PASS')

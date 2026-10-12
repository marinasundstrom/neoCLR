"""Qualify retained declaring-type metadata with the pinned macOS native host recipe."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
for name in ('bundle', 'compiler', 'backend', 'output'):
    parser.add_argument('--' + name, type=Path, required=True)
args = parser.parse_args()
bundle, compiler, backend, output = (getattr(args, n).resolve() for n in ('bundle', 'compiler', 'backend', 'output'))
output.mkdir(parents=True, exist_ok=False)
report = {'passed': False, 'commands': [], 'inputs': {}}
for path in [Path(__file__), HERE / 'Main.rvn', HERE / 'App.rvnproj', compiler, backend, bundle / 'lib/System.Runtime.dll', bundle / 'bin/neoclr-jit-host.dylib']:
    report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
for name in ('Main.rvn', 'App.rvnproj'):
    shutil.copyfile(HERE / name, output / name)


def run(command, expected=0, stdout=None):
    result = subprocess.run(list(map(str, command)), cwd=ROOT,
                            env=dict(os.environ, NeoClrBundleRoot=str(bundle)),
                            capture_output=True, text=True, timeout=600)
    row = dict(command=result.args, exitCode=result.returncode, stderr=result.stderr)
    if len(result.stdout) < 4000:
        row['stdout'] = result.stdout
    else:
        row['stdoutSha256'] = hashlib.sha256(result.stdout.encode()).hexdigest()
    report['commands'].append(row)
    (output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    assert result.returncode == expected, result.stderr
    if stdout is not None:
        assert result.stdout == stdout, result.stdout
    return result


run(['dotnet', compiler, 'neoclr', '--project', output / 'App.rvnproj'])
image = output / 'bin/neoclr/DeclaringType.dll'
expected = 'PASS declaring type\n'
run([bundle / 'bin/neoclr', 'run', image, '--execution', 'interpreter', '--system', bundle / 'lib/System.runtime.neox', '--module', bundle / 'lib/System.Runtime.dll', '--object-root', bundle / 'lib/System.Runtime.dll'], stdout=expected)
# Reuse the qualified compiler flags and host sources; this is a development harness,
# not a portable SDK publishing command. The recipe's paths identify its toolchain.
recipe = json.loads((ROOT / 'docs/experiments/runtime-async/generic-cleanup/mac-shared-gc-validation.json').read_text())
compile_command = next(x['command'] for x in recipe['commands'] if x['label'] == 'use-state-machine-native').copy()
compile_command[0], compile_command[2], compile_command[4] = str(backend), str(image), str(output / 'app.o')
inspection = run([backend, '--inspect', image, '@entry', '--closed-world'] + compile_command[5:])
types = json.loads(inspection.stdout)['types']
inner = next(t for t in types if t.get('origin', {}).get('name') == 'Inner')
owner = inner['declaring_type']
roots = {'schemaVersion': 1, 'types': [dict(definition=dict(owner, index=i), construct=False) for i in (owner['index'], types.index(inner))]}
root_path = output / 'roots.json'
root_path.write_text(json.dumps(roots, indent=2) + '\n')
compile_command += ['--reflection-roots', str(root_path)]
run(compile_command)
link = next(x['command'] for x in recipe['commands'] if x['label'] == 'use-state-machine-link')
link = [str(output / 'app.o') if x.endswith('/use-state-machine.o') else str(output / 'app') if x.endswith('/use-state-machine') else str(ROOT / 'tools/native/console-host.c') if x.endswith('/benchmarks/native-web/http-host.c') else x for x in link]
run(link)
run([output / 'app'], stdout=expected)
jit = compile_command.copy()
jit[4] = str(bundle / 'bin/neoclr-jit-host.dylib')
jit += ['--jit-run']
run(jit, stdout=expected)
# Missing roots must fault instead of fabricating None for the nested declaration.
missing = run(jit[:-3] + ['--jit-run'], expected=1)
assert 'native declaring-type metadata was not retained' in missing.stderr
report['passed'] = True
(output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
print('Declaring type: interpreter, AOT, JIT and missing-retention fault PASS')

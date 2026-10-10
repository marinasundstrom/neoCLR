"""Check flat guest module traversal and explicitly retained native ownership."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
for name in ('bundle', 'aot', 'runner', 'output'):
    parser.add_argument('--' + name, type=Path, required=True)
args = parser.parse_args()
bundle, aot, runner, output = (getattr(args, name).resolve() for name in ('bundle', 'aot', 'runner', 'output'))
output.mkdir(parents=True, exist_ok=False)
source = output / 'source'
source.mkdir()
inputs = [Path(__file__), *HERE.glob('*.rvn'), *HERE.glob('*.rvnproj')]
for path in inputs:
    if path.suffix != '.py':
        shutil.copyfile(path, source / path.name)
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
report = dict(passed=False, platform=platform.platform(), commands=[],
              inputs={str(path.relative_to(ROOT)): sha(path) for path in inputs},
              bundleManifestSha256=sha(bundle / 'manifest.json'), aotSha256=sha(aot), runnerSha256=sha(runner))
env = dict(os.environ, NeoClrBundleRoot=str(bundle))


def run(command, expected=0, fault=None):
    result = subprocess.run(list(map(str, command)), cwd=ROOT, env=env, capture_output=True, text=True, encoding='utf-8', timeout=600)
    record = dict(command=result.args, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr)
    report['commands'].append(record)
    (output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    if fault is None:
        assert result.returncode == expected, result.stdout + result.stderr
    else:
        assert result.returncode != 0 and fault in result.stdout + result.stderr, record
    return result


compiler = bundle / 'sdk/tools/rvnc/rvnc.dll'
lib = bundle / 'lib'
context = ['--system', lib / 'System.runtime.neox', '--module', lib / 'System.Runtime.dll', '--object-root', lib / 'System.Runtime.dll']
run(['dotnet', compiler, 'neoclr', '--project', source / 'Traversal.rvnproj'])
image = source / 'bin/neoclr/CoffeePackage.dll'
traversal = run([runner, 'run', image, *context, '--gc-stats'])
assert traversal.stdout == 'Logical module traversal passed\n' and 'live=0' in traversal.stderr, traversal
run(['dotnet', compiler, 'neoclr', '--project', source / 'Native.rvnproj'])
roots = output / 'roots.json'
run(['dotnet', 'run', '--project', ROOT / 'tools/metadata/NeoCLR.Metadata.Experimental.Tests', '-p:WarningLevel=0',
     '--', '--module-retention', image, roots])


def build(label, policy):
    destination = output / label
    run([sys.executable, ROOT / 'scripts/build-native-project.py', '--profile', 'windows-console' if os.name == 'nt' else 'console',
         '--project', source / 'Native.rvnproj', '--bundle', bundle, '--aot', aot, '--reflection-roots', policy, '--output', destination])
    return destination


native = build('retained', roots)
name = 'app.exe' if os.name == 'nt' else 'app'
# Run only the executable, without compiler/bundle files alongside it.
isolated = output / 'isolated'
isolated.mkdir()
shutil.copyfile(native / name, isolated / name)
shutil.copymode(native / name, isolated / name)
assert run([isolated / name]).stdout == 'Logical module ownership passed\n'
assert run([runner, 'run', native / 'app.dll', *context, '--gc-stats']).stdout == 'Logical module ownership passed\n'
policy = json.loads(roots.read_text())
policy['types'] = policy['types'][:1]
missing = output / 'missing-root.json'
missing.write_text(json.dumps(policy) + '\n')
negative = build('unretained', missing)
run([negative / name], fault='native logical module metadata was not retained')
assert all(sha(ROOT / path) == digest for path, digest in report['inputs'].items()), 'Fixture input changed'
report['passed'] = True
report['scope'] = 'Interpreter assembly/module/type/member/parameter traversal; AOT retained type-to-module ownership. AOT assembly traversal and Object.Equals dispatch are not covered.'
(output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
print('Guest logical module checks: PASS')

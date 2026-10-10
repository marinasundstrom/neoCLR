"""Compile and execute guest module-function metadata discovery (interpreter)."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import platform

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
for name in ('bundle', 'compiler', 'runner', 'output'):
    parser.add_argument('--' + name, type=Path, required=True)
args = parser.parse_args()
bundle, compiler, runner, output = (getattr(args, name).resolve() for name in ('bundle', 'compiler', 'runner', 'output'))
output.mkdir(parents=True, exist_ok=False)
inputs = [Path(__file__), HERE / 'Main.rvn', HERE / 'GuestFunctions.rvnproj', ROOT / 'runtime/raven/tests/framework/TestAttribute.rvn']
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
report = dict(passed=False, platform=platform.platform(), commands=[],
              inputs={str(p.relative_to(ROOT)): sha(p) for p in inputs},
              compilerSha256=sha(compiler), runnerSha256=sha(runner),
              runtimeSha256=sha(bundle / 'lib/System.Runtime.dll'))
for path in inputs[1:]:
    copied = output / 'source' / path.relative_to(ROOT)
    copied.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(path, copied)
project = output / 'source' / HERE.relative_to(ROOT) / 'GuestFunctions.rvnproj'
env = dict(os.environ, NeoClrBundleRoot=str(bundle))


def run(command):
    result = subprocess.run(list(map(str, command)), cwd=ROOT, env=env, capture_output=True, text=True, encoding='utf-8', timeout=600)
    report['commands'].append(dict(command=result.args, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
    (output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    assert result.returncode == 0, result.stdout + result.stderr
    return result


run(['dotnet', compiler, 'neoclr', '--project', project])
image = project.parent / 'bin/neoclr/GuestFunctions.dll'
result = run([runner, 'run', image, '--system', bundle / 'lib/System.runtime.neox',
              '--module', bundle / 'lib/System.Runtime.dll', '--object-root', bundle / 'lib/System.Runtime.dll', '--gc-stats'])
assert result.stdout == 'Guest module function discovery passed\n'
assert 'live=0' in result.stderr
assert all(sha(ROOT / path) == digest for path, digest in report['inputs'].items())
report.update(passed=True, imageSha256=sha(image), scope='Interpreter module function enumeration, absent declaring type, module/parameter metadata, exact TestAttribute identity and fixed/named descriptions. No test invocation, AOT retention or registration is claimed.')
(output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
print('Guest module function discovery: PASS')

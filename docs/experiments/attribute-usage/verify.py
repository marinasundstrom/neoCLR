"""Check runtime usage defaults and imported usage diagnostics with a matching bundle."""
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
for name in ('bundle', 'aot', 'runtime', 'output'):
    parser.add_argument('--' + name, type=Path, required=True)
args = parser.parse_args()
bundle, aot, runtime, output = (p.resolve() for p in (args.bundle, args.aot, args.runtime, args.output))
output.mkdir(parents=True, exist_ok=False)
report = dict(passed=False, host=platform.platform(), commands=[])


def run(command, expected=0, env=None):
    result = subprocess.run(list(map(str, command)), cwd=ROOT, env=env, capture_output=True, text=True, timeout=600)
    report['commands'].append(dict(command=result.args, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
    (output / 'commands.json').write_text(json.dumps(report, indent=2) + '\n')
    assert result.returncode == expected, result.stdout + result.stderr
    return result


build = output / 'native'
run([sys.executable, ROOT / 'scripts/build-native-project.py', '--profile', 'windows-console' if os.name == 'nt' else 'console',
     '--project', HERE / 'Native.rvnproj', '--bundle', bundle, '--aot', aot, '--output', build])
expected = 'Runtime attribute usage passed\n'
isolated = output / 'isolated'
isolated.mkdir()
exe = isolated / ('app.exe' if os.name == 'nt' else 'app')
shutil.copy2(build / exe.name, exe)
clean_env = {k: v for k, v in os.environ.items() if k.upper() in ('SYSTEMROOT', 'WINDIR', 'TEMP', 'TMP')}
assert run([exe], env=clean_env).stdout == expected
lib = bundle / 'lib'
catalog = json.loads((lib / 'bundle.json').read_text())
interpreted = run([runtime, 'run', build / 'app.dll', '--system', lib / catalog['runtimeSeed'],
                   *[arg for assembly in catalog['assemblyNames'] for arg in ('--module', lib / (assembly + '.dll'))],
                   '--object-root', lib / 'System.Runtime.dll', '--gc-stats'])
assert interpreted.stdout == expected and 'live=0' in interpreted.stderr
negative = output / 'invalid-target'
negative.mkdir()
shutil.copyfile(HERE / 'Native.rvnproj', negative / 'Native.rvnproj')
(negative / 'Main.rvn').write_text('import System.*\n[AttributeUsage(AttributeTargets.Method)]\nfunc Invalid() { }\nfunc Main() { }\n')
compiler = bundle / 'sdk/tools/rvnc/rvnc.dll'
rejected = run(['dotnet', compiler, 'neoclr', '--project', negative / 'Native.rvnproj'], expected=1,
               env=dict(os.environ, NeoClrBundleRoot=str(bundle)))
assert 'RAV0502' in rejected.stderr + rejected.stdout, rejected.stderr + rejected.stdout
assert not (negative / 'bin/neoclr/AttributeUsageConsumer.dll').exists()
inputs = [Path(__file__), HERE / 'Main.rvn', HERE / 'Native.rvnproj',
          ROOT / 'runtime/raven/src/System/AttributeTargets.rvn', ROOT / 'runtime/raven/src/System/AttributeUsageAttribute.rvn',
          lib / 'System.Runtime.dll', lib / 'System.runtime.neox', lib / 'Core.dll', lib / 'ownership.json',
          lib / 'build-evidence.json', bundle / 'manifest.json', build / 'app.dll', exe, aot, runtime, compiler,
          compiler.parent / 'Raven.CodeAnalysis.dll', compiler.parent / 'Raven.CodeAnalysis.NeoClr.dll']
report.update(passed=True, cases=['all-target-flag-values', 'flags-composition', 'usage-construction-defaults',
                                 'mutable-options', 'independent-instances', 'native-interpreter-parity',
                                 'interpreter-cleanup', 'imported-usage-target-rejection'],
              inputs={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs})
(output / 'validation.json').write_text(json.dumps({k: v for k, v in report.items() if k != 'commands'}, indent=2) + '\n')
print('PASS runtime AttributeUsage and imported target checks')

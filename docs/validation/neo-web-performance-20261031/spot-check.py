"""Reproduce interpreter/JIT spot checks using qualified bundle and native host recipe."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--bundle', required=True, type=Path)
p.add_argument('--build', required=True, type=Path)
p.add_argument('--host-recipe', required=True, type=Path)
p.add_argument('--output', required=True, type=Path)
a = p.parse_args()
bundle, build, out = a.bundle.resolve(), a.build.resolve(), a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
spec = importlib.util.spec_from_file_location('benchmark', ROOT / 'scripts/benchmark-neo-web.py')
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
recipe = json.loads(a.host_recipe.read_text())
assert recipe['passed']
command = next(row['command'].copy() for row in recipe['commands'] if any(str(arg).endswith('host.dylib') for arg in row['command']) and '-o' in row['command'])
paced = out / 'paced-host.dylib'
command[command.index('-o') + 1] = str(paced)
command.insert(1, '-DNEOCLR_GC_PACED')
built = subprocess.run(command, capture_output=True, text=True, check=True)
(out / 'host-build.log').write_text(built.stdout + built.stderr)
manifest = json.loads((bundle / 'manifest.json').read_text())
project = json.loads((build / 'build.json').read_text())
sha = lambda f: hashlib.sha256(f.read_bytes()).hexdigest()
report = dict(hostCommand=command, bundleManifestSha256=sha(bundle / 'manifest.json'),
              hostSha256={'stress': sha(bundle / 'bin/neoclr-jit-host.dylib'), 'paced': sha(paced)}, cases=[])
for mode, policy, handler in [('interpreter', 'interpreter', 'generic'),
                               ('interpreter', 'interpreter', 'cancellation'),
                               ('jit', 'stress', 'cancellation'), ('jit', 'paced', 'cancellation')]:
    base = [str(bundle / 'bin/neoclr'), 'run', str(build / 'app.dll'), '--execution', mode,
            '--system', str(bundle / 'lib/System.runtime.neox'), '--object-root', str(bundle / 'lib/System.Runtime.dll')]
    if mode == 'jit':
        base += ['--jit-compiler', str(bundle / 'bin/neoclr-codegen'), '--jit-host', str(paced if policy == 'paced' else bundle / 'bin/neoclr-jit-host.dylib'), '--reflection-roots', str(build / 'app.reflection.json')]
    else:
        base += ['--instructions', '1000000000']
    for module in manifest['modules']:
        base += ['--module', str(bundle / module)]
    for module in project.get('projectModules', []):
        base += ['--module', module]
    base += ['--', handler]
    row = dict(mode=mode, policy=policy, handler=handler, command=base)
    row.update(m.measure(base, 100 if policy == 'paced' else 30, 5, '/', b'42', None))
    report['cases'].append(row)
    (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(mode, policy, handler, row['requestsPerSecond'], flush=True)
report['passed'] = True
(out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')

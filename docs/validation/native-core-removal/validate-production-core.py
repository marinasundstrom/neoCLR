"""Qualify a native production core consumer on the existing macOS ARM64 host.

Supply a source-core bundle, a consumer returning 42 and a fresh output directory. Reuses the recorded intrinsic host link recipe.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
bundle = Path(sys.argv[1]).resolve()
image = Path(sys.argv[2]).resolve()
folder = Path(sys.argv[3]).resolve()
folder.mkdir(parents=True, exist_ok=False)
output = folder / 'results.json'
recipe = json.loads((ROOT / 'docs/validation/native-core-removal/intrinsic-results.json').read_text())
report = dict(passed=False, commands=[])
refs = ['--system', bundle / 'System.runtime.neox', '--object-root', bundle / 'System.Runtime.dll',
        '--module', bundle / 'System.Runtime.dll']
options = [*refs, '--compile-system', '--reference-arena', '--native-gc',
           '--native-stack-budget', '--bind-console-read-byte', '--bind-integer-text', '--bind-task-queue', '--bind-user-fault', '--bind-runtime-async']


def run(label, command, expected=0, stdin=None):
    command = list(map(str, command))
    result = subprocess.run(command, input=stdin, capture_output=True, text=True, cwd=ROOT, timeout=240)
    log = result.stdout + result.stderr
    (folder / (label + '.log')).write_text(log)
    report['commands'].append(dict(label=label, command=command, stdin=stdin,
                                   exitCode=result.returncode, output=log if not label.endswith('aot-build') else 'Local selection manifest',
                                   logSha256=hashlib.sha256(log.encode()).hexdigest()))
    output.write_text(json.dumps(report, indent=2) + '\n')
    if result.returncode != expected:
        raise RuntimeError(f'{label}: expected {expected}, got {result.returncode}\n{log[-3000:]}')
    return log


for name in ('Consumer',):
    sub = folder / (name + '-validated')
    sub.mkdir(exist_ok=True)
    run(name + '-interpreter', [ROOT / 'target/debug/neoclr', 'run', image, '--execution', 'interpreter', *refs], 42, '*')
    run(name + '-aot-build', [ROOT / 'target/debug/neoclr-codegen', '--closed-world', image, '@entry', sub / 'app.o', *options])
    for label in ('link-aot', 'link-jit'):
        cmd = next(c['command'] for c in recipe['commands'] if c['label'] == label)
        cmd = [str(sub / Path(x).name) if '/target/native-core-intrinsics-v2/' in x else x for x in cmd]
        cmd = [str(ROOT / 'docs/validation/native-core-removal/native-core-host.c') if x.endswith('/interface-variance/native-host.c') else x for x in cmd]
        cmd.insert(cmd.index('-o'), str(ROOT / 'docs/experiments/aot-console/console.c'))
        cmd.insert(cmd.index('-o'), str(ROOT / 'docs/experiments/aot-console/task-queue.c'))
        cmd.insert(cmd.index('-o'), str(ROOT / 'runtime/native/async/native-runtime.c'))
        cmd.insert(cmd.index('-o'), str(ROOT / 'docs/experiments/aot-console/socket-listener.c'))
        cmd.insert(1, '-Wno-deprecated-declarations')
        run(name + '-' + label, cmd)
    log = run(name + '-aot-run', [sub / 'app'], 42, '*')
    log = run(name + '-jit', [ROOT / 'target/debug/neoclr-codegen', '--jit-managed', '--closed-world', image, '@entry', sub / 'host.dylib', *options], stdin='*')
    result = json.loads(log.split('JIT_RESULT ', 1)[1])
    if result['guestExitCode'] != 42:
        raise RuntimeError('Unexpected JIT result: ' + log[-2000:])
report['passed'] = True
report['sha256'] = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in [Path(__file__), ROOT / 'docs/validation/native-core-removal/native-core-host.c', image, bundle / 'System.Runtime.dll', bundle / 'bundle.json', ROOT / 'target/debug/neoclr', ROOT / 'target/debug/neoclr-codegen', *ROOT.joinpath('runtime/native/async').glob('*.[ch]'), *ROOT.joinpath('docs/experiments/aot-console').glob('*.[ch]')]}
output.write_text(json.dumps(report, indent=2) + '\n')
print('PASS production consumer interpreter/JIT/AOT')

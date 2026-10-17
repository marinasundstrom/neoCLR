"""Qualify prebuilt native storage helper consumers on the existing macOS ARM64 host.

Run Raven NeoClrMetadataProbe with --native-source-core first, then supply its
output folder and an output JSON path. Assemble an empty System seed in that
folder before running. Reuses the recorded intrinsic host link recipe.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
folder = Path(sys.argv[1]).resolve()
output = Path(sys.argv[2]).resolve()
recipe = json.loads((ROOT / 'docs/validation/native-core-removal/intrinsic-results.json').read_text())
report = dict(passed=False, commands=[])
refs = ['--system', folder / 'System.neox', '--object-root', folder / 'SourceCore.dll',
        '--module', folder / 'SourceCore.dll']
options = [*refs, '--compile-system', '--reference-arena', '--native-gc',
           '--native-stack-budget', '--bind-console-read-byte']


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


for name in ('Consumer', 'ReadUninitialized', 'NegativeLength'):
    sub = folder / name
    sub.mkdir(exist_ok=True)
    image = folder / (name + '.dll')
    valid = name == 'Consumer'
    run(name + '-interpreter', [ROOT / 'target/debug/neoclr', 'run', image, '--execution', 'interpreter', *refs], 42 if valid else 1, '*')
    run(name + '-aot-build', [ROOT / 'target/debug/neoclr-codegen', '--closed-world', image, '@entry', sub / 'app.o', *options])
    for label in ('link-aot', 'link-jit'):
        cmd = next(c['command'] for c in recipe['commands'] if c['label'] == label)
        cmd = [str(sub / Path(x).name) if '/target/native-core-intrinsics-v2/' in x else x for x in cmd]
        cmd.insert(cmd.index('-o'), str(ROOT / 'docs/experiments/aot-console/console.c'))
        run(name + '-' + label, cmd)
    log = run(name + '-aot-run', [sub / 'app'], 42 if valid else 2, '*')
    if not valid and 'RuntimeError' not in log:
        raise RuntimeError('Missing AOT storage fault')
    log = run(name + '-jit', [ROOT / 'target/debug/neoclr-codegen', '--jit-managed', '--closed-world', image, '@entry', sub / 'host.dylib', *options], stdin='*')
    result = json.loads(log.split('JIT_RESULT ', 1)[1])
    if result['guestExitCode'] != (42 if valid else 2) or not valid and 'RuntimeError' not in log:
        raise RuntimeError('Unexpected JIT result: ' + log[-2000:])
report['passed'] = True
report['sha256'] = {str(f): hashlib.sha256(f.read_bytes()).hexdigest() for f in
                    [folder / 'SourceCore.dll', folder / 'Consumer.dll', folder / 'ReadUninitialized.dll', folder / 'NegativeLength.dll',
                     ROOT / 'target/debug/neoclr', ROOT / 'target/debug/neoclr-codegen']}
output.write_text(json.dumps(report, indent=2) + '\n')
print('PASS storage interpreter/JIT/AOT success and rejection')

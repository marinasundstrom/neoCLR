"""Generate and compile the candidate from copied runtime sources without benchmarks."""
import argparse, hashlib, json, os, shutil, subprocess, sys
from pathlib import Path

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--cc', default=os.environ.get('CC', 'clang'))
a = p.parse_args()
root = Path(__file__).resolve().parents[3]
out = a.output.resolve(); out.mkdir(parents=True, exist_ok=False)
native = out/'runtime/native'
r = dict(passed=False, commands=[], inputs={})

def run(command):
    command = list(map(str, command))
    x = subprocess.run(command, capture_output=True, text=True, timeout=120)
    r['commands'].append(dict(command=command, exitCode=x.returncode, stdout=x.stdout, stderr=x.stderr))
    x.check_returncode()

try:
    for source in (root/'runtime/native').rglob('*'):
        if source.is_file() and source.suffix in ['.c', '.h', '.py', '.json'] and '__pycache__' not in source.parts:
            target = native/source.relative_to(root/'runtime/native')
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target)
            r['inputs'][str(source)] = hashlib.sha256(source.read_bytes()).hexdigest()
    candidate = native/'gc/concurrent'
    gc = out/'native-gc.c'; provider = out/'text-arena.c'
    run([sys.executable, candidate/'generate.py', '--output', gc])
    run([sys.executable, candidate/'native-providers.py', '--output', provider])
    policy = json.loads((candidate/'policy.json').read_text())
    common = [a.cc, '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-pthread',
              *['-D'+value for value in policy['defines']],
              '-I', native/'gc', '-I', native/'memory', '-I', candidate]
    for source in [gc, provider, candidate/'gate.c']:
        run(common+['-c', source, '-o', out/(source.stem+'.o')])
    r['outputs'] = {p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in [gc,provider]}
    r['passed'] = True
finally:
    (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

import argparse
import hashlib
import json
from pathlib import Path
import subprocess

p = argparse.ArgumentParser()
p.add_argument('--output', type=Path, required=True)
p.add_argument('--gate-words', type=int, default=4096)
a = p.parse_args()
base = Path(__file__).resolve().parent
root = base.parents[2]
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
r = dict(passed=False, commands=[])

def run(cmd):
    cmd = list(map(str,cmd))
    result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
    r['commands'].append(dict(command=cmd, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
    result.check_returncode()

try:
    recipe = json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
    cc = recipe[:recipe.index('-std=c11')]
    for sanitize in [False,True]:
        exe = out/f'test-{sanitize}'
        cmd = cc + [f'-DGATE_WORDS={a.gate_words}','-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread',base/'gate.c',base/'test.c','-o',exe]
        if sanitize: cmd += ['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
        run(cmd)
        run([exe])
    r['inputs'] = {str(x):hashlib.sha256(x.read_bytes()).hexdigest() for x in [base/'gate.c',base/'gate.h',base/'test.c',Path(__file__)]}
    r['passed'] = True
finally:
    (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

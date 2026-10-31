"""Paired Neo.Web JIT comparison, retaining one app image and native host."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--baseline', required=True, type=Path)
p.add_argument('--candidate', required=True, type=Path)
p.add_argument('--recipe', required=True, type=Path, help='Previous spot-results.json with a paced JIT case')
p.add_argument('--output', required=True, type=Path)
p.add_argument('--repetitions', type=int, default=3)
p.add_argument('--requests', type=int, default=1000)
p.add_argument('--aot', type=Path, help='Optional prelinked AOT control with the same heap and GC policy')
a = p.parse_args()
if min(a.repetitions, a.requests) < 1:
    p.error('Positive repetitions and requests are required')
if os.environ.get('NEOCLR_JIT_LAZY', '0') != '0':
    p.error('This comparison requires eager JIT (NEOCLR_JIT_LAZY unset or 0)')
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
spec = importlib.util.spec_from_file_location('benchmark', ROOT / 'scripts/benchmark-neo-web.py')
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
recipe = json.loads(a.recipe.read_text())
base = next(r['command'] for r in recipe['cases'] if r['mode'] == 'jit' and r['policy'] == 'paced')
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
report = dict(sourceRevision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
              compilers={k: dict(path=str(v.resolve()), sha256=sha(v)) for k, v in [('baseline', a.baseline), ('candidate', a.candidate)]},
              appSha256=sha(base[2]), hostSha256=sha(base[base.index('--jit-host')+1]),
              scope='Eager JIT, unchanged paced host and image, alternating compiler order; fresh sequential loopback connections.', cases=[])
for repetition in range(a.repetitions):
    order = ['baseline', 'candidate'] if repetition % 2 == 0 else ['candidate', 'baseline']
    for variant in order:
        command = base.copy()
        command[command.index('--jit-compiler')+1] = report['compilers'][variant]['path']
        row = dict(variant=variant, repetition=repetition+1, handler=command[-1], command=command)
        row.update(m.measure(command, a.requests, 20, '/', b'42', None))
        report['cases'].append(row)
        (out/'results.json').write_text(json.dumps(report, indent=2)+'\n')
        print(f'{variant}/{repetition+1}: ready {row["readySeconds"]:.3f}s; {row["requestsPerSecond"]:.1f} req/s', flush=True)
if a.aot:
    report['aotSha256'] = sha(a.aot)
    report['aotCases'] = []
    for repetition in range(a.repetitions):
        command = [str(a.aot.resolve()), base[-1]]
        row = dict(repetition=repetition+1, command=command)
        row.update(m.measure(command, a.requests, 20, '/', b'42', None))
        report['aotCases'].append(row)
        (out/'results.json').write_text(json.dumps(report, indent=2)+'\n')
        print(f'aot/{repetition+1}: {row["requestsPerSecond"]:.1f} req/s', flush=True)
report['passed'] = True
(out/'results.json').write_text(json.dumps(report, indent=2)+'\n')

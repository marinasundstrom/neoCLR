#!/usr/bin/env python3
"""Sample a qualified native async benchmark with symbols and checked repeated invocations.

Reuses the benchmark's guest object and exact host link recipe. This is diagnostic
sampling, not a timing comparison; every invocation still verifies reclamation.
"""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--benchmark', type=Path, required=True)
p.add_argument('--case', choices=['completed-100', 'pending-100'], default='pending-100')
p.add_argument('--lowering', choices=['runtime', 'state-machine'], default='runtime')
p.add_argument('--seconds', type=int, default=3)
p.add_argument('--repetitions', type=int, default=200)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
if platform.system() != 'Darwin' or a.seconds < 1 or a.repetitions < 1:
    p.error('requires macOS and positive duration/repetitions')
r = json.loads(a.benchmark.read_text())
if not r['passed']:
    p.error('benchmark must have passed')
name = f'{a.case}-{a.lowering}'
command = next(x['command'].copy() for x in r['commands']
               if '-DNEOCLR_BENCH_AOT' in x['command']
               and Path(x['command'][x['command'].index('-o') + 1]).name == name)
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
command[command.index('-o') + 1] = str(out / name)
command += ['-g', '-DNEOCLR_GC_PROFILE', f'-DNEOCLR_BENCH_REPETITIONS={a.repetitions}']
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
inputs = {str(Path(v).resolve()): sha(v) for v in command if Path(v).is_file()}
for directory in (Path(__file__).resolve().parent, Path(__file__).resolve().parent.parent / 'aot-console'):
    for header in directory.glob('*.h'):
        inputs[str(header)] = sha(header)
inputs[str(Path(__file__).resolve())] = sha(__file__)
inputs[str(a.benchmark.resolve())] = sha(a.benchmark)
report = dict(passed=False, command=command, inputsSha256=inputs,
              platform=platform.platform(), case=a.case, lowering=a.lowering,
              repetitions=a.repetitions, samplingSeconds=a.seconds,
              caveat='Instrumented diagnostic process, not matched benchmark timing')
try:
    subprocess.run(command, check=True, capture_output=True, text=True, timeout=120)
    with (out / 'stdout').open('w') as stdout, (out / 'samples.jsonl').open('w') as stderr:
        process = subprocess.Popen([str(out / name)], stdout=stdout, stderr=stderr)
        try:
            sample_command = ['sample', str(process.pid), str(a.seconds), '1', '-file', str(out / 'stacks.txt')]
            report['sampleCommand'] = sample_command
            sampled = subprocess.run(sample_command, capture_output=True, text=True, timeout=a.seconds + 30)
            report['sampleDiagnostic'] = sampled.stdout + sampled.stderr
            sampled.check_returncode()
            report['exitCode'] = process.wait(timeout=180)
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()
    samples = [json.loads(line.removeprefix('NATIVE_SAMPLE '))
               for line in (out / 'samples.jsonl').read_text().splitlines()]
    if report['exitCode'] or len(samples) != a.repetitions or (out / 'stdout').read_text():
        raise RuntimeError('guest failed or incomplete invocation samples')
    if any(s['allocatedBlocks'] != s['reclaimedBlocks'] or s['reservedStackBytes'] for s in samples):
        raise RuntimeError('unreclaimed heap/stack resources')
    if any(sha(path) != expected for path, expected in inputs.items()):
        raise RuntimeError('input changed while profiling')
    report['passed'] = True
finally:
    (out / 'profile.json').write_text(json.dumps(report, indent=2) + '\n')
print(out / 'stacks.txt')

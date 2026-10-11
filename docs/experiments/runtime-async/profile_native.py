#!/usr/bin/env python3
"""Sample a qualified native async benchmark with symbols and checked repeated invocations.

Recompiles the benchmark guest and reuses its host link recipe. This is diagnostic
sampling, not a timing comparison; every invocation still verifies reclamation.
"""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import re
import shlex
import statistics
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--benchmark', type=Path, required=True)
p.add_argument('--case', choices=['completed-100', 'pending-100'], default='pending-100')
p.add_argument('--lowering', choices=['runtime', 'state-machine'], default='runtime')
p.add_argument('--seconds', type=int, default=3)
p.add_argument('--repetitions', type=int, default=200)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
if platform.system() != 'Darwin' or a.seconds < 1 or a.repetitions < 2:
    p.error('requires macOS, a positive duration and at least two repetitions')
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
# Recompile the guest and retain its selection report together: an old symbol
# inventory must never silently annotate a different generated object.
old_object = next(v for v in command if v.endswith('.o'))
compile_command = next(x['command'].copy() for x in r['commands']
                       if '--closed-world' in x['command'] and '--jit-managed' not in x['command']
                       and old_object in x['command'])
obj = out / 'guest.o'
compile_command[compile_command.index(old_object)] = str(obj)
command[command.index(old_object)] = str(obj)
inputs = {str(Path(v).resolve()): sha(v) for v in [*command, *compile_command] if Path(v).is_file()}
# Ask the C compiler for transitive local dependencies, including implementation
# sources included by the benchmark host. Do not run its linker for this step.
dependency_command = [command[0], '-MM', '-MT', 'profile']
for i, value in enumerate(command[1:], 1):
    if value in ('-isysroot', '-I', '-include'):
        dependency_command.extend([value, command[i + 1]])
    elif value.startswith(('-D', '-I', '-std=')) or value.endswith('.c'):
        dependency_command.append(value)
dependencies = subprocess.run(dependency_command, check=True, capture_output=True, text=True, timeout=60)
for line in dependencies.stdout.replace('\\\n', '').splitlines():
    for value in shlex.split(line.split(':', 1)[1]):
        inputs[str(Path(value).resolve())] = sha(value)
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
    compiled = subprocess.run(compile_command, check=True, capture_output=True, text=True, timeout=180)
    selection_path = out / 'selection.json'
    selection_path.write_text(compiled.stdout)
    selection = json.loads(compiled.stdout)
    def readable(name):
        # Diagnostic-only decoding of this experimental projection. Preserve any
        # unrecognized spelling; identity always remains the report's exact name.
        def decode(value):
            result = bytes.fromhex(value).decode('utf-8')
            if any(ord(c) < 32 or ord(c) == 127 for c in result):
                raise ValueError('control character')
            return result
        try:
            match = re.fullmatch(r'NeoMetadata_[0-9A-Fa-f]{64}\.(.*)', name)
            if not match: return name
            member = match[1]
            if member.startswith('F_'):
                function = decode(member[2:])
                if function.startswith('<NeoFunction>'):
                    namespace, method = function[len('<NeoFunction>'):].split('_', 1)
                    return '.'.join(x for x in (decode(namespace), decode(method)) if x)
                return function
            if not member.startswith('T_'): return name
            owner, method = member[2:].split('.', 1)
            namespace, owner = owner.split('_', 1)
            method = decode(method[2:]) if method.startswith('M_') else method
            return '.'.join(x for x in (decode(namespace), decode(owner), method) if x)
        except (ValueError, UnicodeError):
            return name
    symbols = {f"neoclr_value_{row['compiledIndex']}": dict(name=row['name'], display=readable(row['name']))
               for row in selection['functions']}
    mapping = dict(objectSha256=sha(obj), selectionSha256=sha(selection_path), symbols=symbols)
    (out / 'symbols.json').write_text(json.dumps(mapping, indent=2) + '\n')
    report.update(compileCommand=compile_command, compileStderr=compiled.stderr,
                  objectSha256=sha(obj), selectionSha256=sha(selection_path),
                  dependencyCommand=dependency_command)
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
    raw = (out / 'stacks.txt').read_text()
    hits = set()
    def annotate(match):
        symbol = match[0]
        if symbol not in symbols: return symbol
        hits.add(symbol)
        return symbol + ' [' + symbols[symbol]['display'] + ']'
    annotated = re.sub(r'\bneoclr_value_\d+\b', annotate, raw)
    (out / 'stacks-readable.txt').write_text(annotated)
    # The first invocation is retained in raw evidence but omitted from summaries.
    retained = samples[1:]
    if not retained: raise RuntimeError('need at least two invocations for a warmup-excluded summary')
    report['summary'] = dict(retainedInvocations=len(retained), discardedWarmups=1,
        medianSeconds=statistics.median(s['seconds'] for s in retained),
        medianGcFraction=statistics.median(s['gcNanoseconds'] / 1e9 / s['seconds'] for s in retained),
        medianGcAttempts=statistics.median(s['gcAttempts'] for s in retained),
        medianAllocatedBlocks=statistics.median(s['allocatedBlocks'] for s in retained),
        medianResumes=statistics.median(s['resumes'] for s in retained),
        annotatedGuestSymbols=len(hits))
    if sha(obj) != mapping['objectSha256'] or sha(selection_path) != mapping['selectionSha256']:
        raise RuntimeError('guest object or symbol selection changed while profiling')
    report['outputSha256'] = {name: sha(out / name) for name in
        ('stacks.txt', 'stacks-readable.txt', 'samples.jsonl', 'symbols.json')}
    report['passed'] = True
finally:
    (out / 'profile.json').write_text(json.dumps(report, indent=2) + '\n')
print(out / 'stacks-readable.txt')

#!/usr/bin/env python3
"""Replay qualified library assemblies through a relocated installed default JIT."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--bundle', type=Path, required=True)
p.add_argument('--library-report', type=Path, required=True)
p.add_argument('--completed-prefix-report', type=Path, help='Reuse only successful suites preceding a terminal loader failure')
p.add_argument('--async-stack-assembly', type=Path, help='Qualified AsyncStackTraceCases consumer (macOS)')
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
bundle = a.bundle.resolve()
manifest = json.loads((bundle / 'manifest.json').read_text())
for relative, digest in manifest['files'].items():
    path = (bundle / relative).resolve()
    if bundle not in path.parents or sha(path) != digest:
        raise ValueError('Installed file mismatch: ' + relative)
baseline = json.loads(a.library_report.read_text())
if not baseline['passed']:
    raise ValueError('The primary baseline must pass')
sources = [(a.library_report.resolve(), baseline)]
excluded = None
if a.completed_prefix_report:
    partial = json.loads(a.completed_prefix_report.read_text())
    failed = partial['cases'][-1]
    if partial['passed'] or not failed['exitCode'] or not failed['stderr'] or failed['stdout']:
        raise ValueError('Prefix reuse requires a final terminal loader failure with no guest output')
    excluded = failed['suite']
    completed = [row for row in partial['cases'][:-1] if row.get('mode') == 'jit' and row.get('repetition') == 1]
    if any(row['exitCode'] or row['stderr'] or ', failed: 0, skipped: 0\n' not in row['stdout'] for row in completed):
        raise ValueError('The prefix contains an unqualified suite')
    partial = dict(partial, cases=completed)
    sources.insert(0, (a.completed_prefix_report.resolve(), partial))
rows = []
assemblies = {}
for source, baseline in sources:
    for row in baseline['cases']:
        if row.get('mode') != 'jit' or row.get('repetition') != 1:
            continue
        if row['suite'] in assemblies:
            raise ValueError('Duplicate suite in input reports')
        assembly = source.parent / (row['suite'] + '-build/app.dll')
        evidence = next(phase for phase in baseline['phases']
                        if phase.get('suite') == row['suite'] and phase.get('phase') == 'source-compilation')
        if sha(assembly) != evidence['assemblySha256']:
            raise ValueError('Compiled assembly changed: ' + row['suite'])
        assemblies[row['suite']] = assembly
        rows.append(row)
if not rows:
    raise ValueError('No qualified JIT suites')
report = dict(passed=False, cases=[], manifestSha256=sha(bundle / 'manifest.json'),
              baselinesSha256={str(path): sha(path) for path, _ in sources},
              excludedFailedSuite=excluded, scriptSha256=sha(__file__),
              assembliesSha256={name: sha(path) for name, path in assemblies.items()})
a.output.parent.mkdir(parents=True, exist_ok=True)
try:
    with tempfile.TemporaryDirectory(prefix='neoCLR relocated JIT ') as directory:
        temporary = Path(directory)
        installed = temporary / 'installed bundle'
        shutil.copytree(bundle, installed)
        runtime = installed / manifest['runtime']
        def run(name, arguments):
            command = [str(runtime), *map(str, arguments)]
            result = subprocess.run(command, cwd=temporary, capture_output=True, text=True, timeout=600)
            row = dict(name=name, command=command, exitCode=result.returncode,
                       stdout=result.stdout, stderr=result.stderr)
            report['cases'].append(row)
            return row
        for row in rows:
            actual = run(row['suite'], ['run', assemblies[row['suite']]])
            if any(actual[key] != row[key] for key in ('exitCode', 'stdout', 'stderr')):
                raise RuntimeError('Installed default differs: ' + row['suite'] + ': ' + str(actual))
            print('PASS installed default ' + row['suite'], flush=True)
        interpreted = run('explicit-interpreter', ['run', ROOT / 'examples/hello.neoil', '--execution', 'interpreter'])
        if (interpreted['exitCode'], interpreted['stdout'], interpreted['stderr']) != (0, 'Hello, world!\n', ''):
            raise RuntimeError('Explicit interpreter control failed: ' + str(interpreted))
        if a.async_stack_assembly:
            report['asyncAssemblySha256'] = sha(a.async_stack_assembly)
            actual = run('runtime-await-stack', ['run', a.async_stack_assembly.resolve()])
            blocks = actual['stdout'].rstrip('\n').split('\n\n')
            if (actual['exitCode'] or actual['stderr'] or len(blocks) != 2 or blocks[0] != blocks[1]
                    or '$closure$0.Invoke' not in blocks[0] or 'NeoMetadata_' in blocks[0]):
                raise RuntimeError('Runtime await/GC stack snapshot failed: ' + str(actual))
        report['passed'] = True
finally:
    a.output.write_text(json.dumps(report, indent=2) + '\n')

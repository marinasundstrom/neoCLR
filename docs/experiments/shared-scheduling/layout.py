#!/usr/bin/env python3
"""Inventory native scheduler metadata; this is not total task/retained memory."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
here = Path(__file__).resolve().parent
root = here.parents[2]
native = root / 'runtime/native'
report = dict(passed=False, platform=platform.platform(), commands=[], layouts={},
              scope='Native fixed metadata only. Nested records overlap: do not sum columns. '
                    'Excludes managed captures/state machines, reachable graphs, TLS root tables, '
                    'heap capacity, allocator overhead, thread stacks and OS resources. '
                    'No context-switch timing or GC performance claim.')

def run(command):
    command = list(map(str, command))
    r = subprocess.run(command, capture_output=True, text=True, timeout=60)
    report['commands'].append(dict(command=command, exitCode=r.returncode,
                                   stdout=r.stdout, stderr=r.stderr))
    r.check_returncode()
    return r.stdout

try:
    cc = [run(['xcrun', '--find', 'clang']).strip(), '-isysroot',
          run(['xcrun', '--show-sdk-path']).strip(), '-std=c11', '-O2',
          '-Wall', '-Wextra', '-Werror', '-pthread', '-DNEOCLR_NATIVE_GC']
    for name, flags in (
        ('baseline', []),
        ('shared', ['-DNEOCLR_SHARED_READY_QUEUE']),
        ('shared-diagnostics', ['-DNEOCLR_SHARED_READY_QUEUE', '-DNEOCLR_SCHEDULER_DIAGNOSTICS']),
        ('shared-stealing', ['-DNEOCLR_SHARED_READY_QUEUE', '-DNEOCLR_WORKER_QUEUES']),
    ):
        binary = out / name
        run([*cc, *flags, '-I', native/'gc', '-I', native/'workers', here/'layout.c',
             native/'gc/native-gc.c', native/'gc/root-probe.c', '-o', binary])
        report['layouts'][name] = json.loads(run([binary]))
    files = [here/'layout.c', Path(__file__)]
    files += [f for base in (native, root/'docs/experiments/aot-console')
              for f in base.rglob('*') if f.suffix in ('.c', '.h', '.inc')]
    report['inputs'] = {str(f.relative_to(root)): hashlib.sha256(f.read_bytes()).hexdigest()
                        for f in files}
    report['passed'] = True
finally:
    (out/'report.json').write_text(json.dumps(report, indent=2)+'\n')
print(json.dumps(report['layouts'], indent=2))

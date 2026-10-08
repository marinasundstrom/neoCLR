#!/usr/bin/env python3
"""Compare a historical kernel against the working kernel with identical -O2 adapters.
Requires an existing --native-gc Route Run(Int32) object from verify_route_lifetime.py.
Route timings include process startup and output capture; graph timings exclude setup.
"""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import statistics
import subprocess
import time

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--baseline', default='9bb27be6')
p.add_argument('--route-object', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
base = Path(__file__).resolve().parent
root = base.parents[2]
a.output.mkdir(parents=True, exist_ok=False)
old = a.output / 'baseline.c'
old.write_bytes(subprocess.check_output(['git', 'show', f'{a.baseline}:docs/experiments/aot-console/native-gc.c'], cwd=root))
flags = ['clang', '-arch', 'arm64', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-I', str(base)]
report = {'baseline': subprocess.check_output(['git', 'rev-parse', a.baseline], cwd=root, text=True).strip(),
          'host': platform.platform(), 'compiler': subprocess.check_output(['clang', '--version'], text=True),
          'flags': flags, 'routeRequests': 1024, 'routeCapacity': 65536,
          'timing': 'graph: 100 collections of 1024 reverse-linked nodes, excludes construction; route: process wall time including startup and captured output',
          'inputs': {}, 'samplesSeconds': {}}
adapters = [base / 'route-lifetime-host.c', base.parent / 'aot-fault-details/render.c',
            base / 'console.c', base.parent / 'aot-scalar/console.c', base / 'text-arena.c', base / 'root-probe.c']
for f in [a.route_object, old, base / 'native-gc.c', base / 'native-gc-bench.c', *adapters, *base.glob('*.h')]:
    report['inputs'][str(f)] = hashlib.sha256(f.read_bytes()).hexdigest()
for name, kernel in [('baseline', old), ('worklist', base / 'native-gc.c')]:
    subprocess.run([*flags, str(base / 'native-gc-bench.c'), str(kernel), str(base / 'root-probe.c'), '-o', str(a.output / (name + '-graph'))], check=True)
    subprocess.run([*flags, '-DNEOCLR_ROOT_PROBES', '-DNEOCLR_NATIVE_GC', *map(str, adapters), str(kernel), str(a.route_object), '-o', str(a.output / (name + '-route'))], check=True)
for workload in ['graph', 'route']:
    samples = {'baseline': [], 'worklist': []}
    for round in range(9):  # first two pairs warm up; alternate ordering
        for name in (['baseline', 'worklist'] if round % 2 == 0 else ['worklist', 'baseline']):
            args = [str(a.output / (name + '-' + workload))]
            if workload == 'route':
                args += ['1024', '65536', 'audit']
            start = time.perf_counter()
            r = subprocess.run(args, check=True, capture_output=True, text=True)
            elapsed = time.perf_counter() - start
            if workload == 'graph':
                elapsed = float(r.stdout)
            else:
                assert r.stdout.strip() == '1024', r
                lines = r.stderr.splitlines()
                audit = {prefix: json.loads(next(x[len(prefix)+1:] for x in lines if x.startswith(prefix + ' '))) for prefix in ['NATIVE_GC', 'AOT_MEASURE']}
                assert audit['AOT_MEASURE']['used'] == 1648 and audit['AOT_MEASURE']['status'] == 0, audit
                key = name + 'RouteAudit'
                if key in report:
                    assert report[key] == audit
                report[key] = audit
            if round >= 2:
                samples[name].append(elapsed)
    report['samplesSeconds'][workload] = samples
    medians = {k: statistics.median(v) for k, v in samples.items()}
    report[workload] = {'medianSeconds': medians, 'baselineOverWorklist': medians['baseline'] / medians['worklist']}
assert report['baselineRouteAudit'] == report['worklistRouteAudit']
(a.output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({k: report[k] for k in ['graph', 'route']}, indent=2))

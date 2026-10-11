#!/usr/bin/env python3
"""Alternate already-built AOT cache variants; preserve samples and check teardown."""
import argparse
import hashlib
import json
from pathlib import Path
import statistics
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--without-cache', type=Path, required=True, help='benchmark_modes.py results.json')
p.add_argument('--with-cache', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
reports = {0: a.without_cache, 8: a.with_cache}
loaded = {limit: json.loads(path.read_text()) for limit, path in reports.items()}
for limit, data in loaded.items():
    if not data['passed'] or data['stackCacheLimit'] != limit:
        p.error('requires passing runs with cache bounds 0 and 8')
# Both experiments must use the same carrier, measurement host and input program.
for suffix in ['native-runtime.c', 'native-runtime.h', 'native-benchmark-host.c',
               'RuntimeAwaitBenchmark.rvn', 'RuntimeAsyncBridge.rvn']:
    hashes = [next(v for k, v in data['inputsSha256'].items() if k.endswith('/' + suffix))
              for data in loaded.values()]
    if len(set(hashes)) != 1:
        p.error('variant sources differ: ' + suffix)
if a.output.exists():
    p.error('output already exists')
report = dict(passed=False, rounds=8, discardedWarmupsPerProcess=1, samples=[], summaries=[],
              runnerSha256=sha(__file__), inputsSha256={str(path): sha(path) for path in reports.values()})
try:
    for case in ['completed-100', 'pending-100']:
        binaries = {limit: path.resolve().parent / (case + '-runtime') for limit, path in reports.items()}
        for limit, binary in binaries.items():
            if sha(binary) != loaded[limit]['inputsSha256'][str(binary)]:
                raise RuntimeError('binary differs from measured artifact')
            report['inputsSha256'][str(binary)] = sha(binary)
        for round_index in range(8):
            for limit in ([0, 8] if round_index % 2 == 0 else [8, 0]):
                command = [str(binaries[limit])]
                run = subprocess.run(command, capture_output=True, text=True, timeout=30)
                lines = run.stderr.splitlines()
                if run.returncode or run.stdout or len(lines) != 4 or any(not x.startswith('NATIVE_SAMPLE ') for x in lines):
                    raise RuntimeError('invalid benchmark output: ' + run.stderr)
                samples = [json.loads(x.removeprefix('NATIVE_SAMPLE ')) for x in lines]
                for sample in samples:
                    if (sample['allocatedBlocks'] != sample['reclaimedBlocks'] or sample['reservedStackBytes'] or
                            sample['stackMappings'] != sample['stackUnmappings']):
                        raise RuntimeError('invocation retained resources')
                report['samples'].append(dict(case=case, round=round_index, stackCacheLimit=limit,
                                               command=command, invocations=samples))
        summary = dict(case=case)
        for limit in [0, 8]:
            samples = [s['seconds'] for r in report['samples'] if r['case'] == case and r['stackCacheLimit'] == limit
                       for s in r['invocations'][1:]]
            summary[str(limit)] = dict(median=statistics.median(samples), minimum=min(samples), maximum=max(samples))
        summary['cachedOverUncached'] = summary['8']['median'] / summary['0']['median']
        report['summaries'].append(summary)
    for path, expected in report['inputsSha256'].items():
        if sha(path) != expected:
            raise RuntimeError('input changed during measurement')
    report['passed'] = True
finally:
    a.output.write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report['summaries'], indent=2))

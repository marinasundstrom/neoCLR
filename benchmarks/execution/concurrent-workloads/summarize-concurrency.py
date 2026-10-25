"""Summarize matched HTTP client concurrency without claiming GC pause distributions."""
import argparse
import hashlib
import json
from pathlib import Path
import statistics

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--measurements', type=Path, nargs='+', required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
report = {
    'scope': __doc__,
    'notes': [
        'Median trial values; latency percentiles are medians of trial percentiles, not pooled percentiles.',
        'Client latency includes connection setup, queueing, response validation and client scheduling.',
        'Server process CPU includes startup, warmup and cleanup; client CPU covers the measured request interval.',
        'Collection timing is collector instrumentation, not application pause time.',
        'Concurrent clients do not imply multiple server workers or qualified shared heaps.',
    ],
    'cohorts': [], 'inputs': {},
}
seen = set()
work = set()
modes = None
for path in args.measurements:
    data = json.loads(path.read_text())
    assert data['passed'] and not data.get('diagnosticOnly')
    key = (data['sample'], data['concurrency'])
    assert key not in seen
    seen.add(key)
    trials = data['trials']
    assert trials
    current_modes = {t['mode'] for t in trials}
    if modes is None: modes = current_modes
    assert modes == current_modes
    assert len({(t['requests'], t['warmup']) for t in trials}) == 1
    work.add((trials[0]['requests'], trials[0]['warmup']))
    assert len(work) == 1
    assert all(t['clientConcurrency'] == data['concurrency'] for t in trials)
    equal = len({(t['allocations'], t['gcProfile']['allocatedBytes']) for t in trials}) == 1
    assert equal or data.get('allowAllocationVariation')
    cohort = {'sample': key[0], 'concurrency': key[1], 'allocationEquality': equal, 'modes': {}}
    for mode in sorted(current_modes):
        rows = [t for t in trials if t['mode'] == mode]
        med = lambda values: statistics.median(values)
        result = {field: med(t[field] for t in rows) for field in
                  ['requestsPerSecond', 'processCpuSeconds', 'clientCpuSeconds', 'peakRssBytes', 'allocations']}
        result['trialCount'] = len(rows)
        result['allocationRange'] = [min(t['allocations'] for t in rows), max(t['allocations'] for t in rows)]
        result['allocatedBytesRange'] = [min(t['gcProfile']['allocatedBytes'] for t in rows), max(t['gcProfile']['allocatedBytes'] for t in rows)]
        result['clientLatencyMs'] = {q: med(t['clientLatencyMs'][q] for t in rows) for q in ['0.5', '0.95', '0.99', '1']}
        result['gcProfile'] = {field: med(t['gcProfile'][field] for t in rows) for field in
                               ['attempts', 'nanoseconds', 'maximumNanoseconds', 'allocatedBytes', 'peakLiveBytes', 'peakArenaUsed']}
        if mode != 'sync':
            result['candidateCounts'] = [med(t['candidateCounts'][i] for t in rows) for i in range(6)]
        cohort['modes'][mode] = result
    report['cohorts'].append(cohort)
    report['inputs'][str(path.resolve())] = hashlib.sha256(path.read_bytes()).hexdigest()
report['requests'], report['warmup'] = next(iter(work))
report['inputs'][str(Path(__file__).resolve())] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
args.output.write_text(json.dumps(report, indent=2) + '\n')

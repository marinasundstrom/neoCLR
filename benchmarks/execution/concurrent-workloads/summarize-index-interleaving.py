"""Validate balanced index-variant cohorts and summarize paired CPU ratios.

Ratios pair variants with the indexed live baseline in the same sample/round.
They describe observed runs, not independent confidence intervals.
"""
import argparse
import collections
import hashlib
import json
from pathlib import Path
import statistics

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--measurement', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
data = a.measurement.read_bytes()
r = json.loads(data)
assert r['passed'] and r['allocationEquality'] and not r.get('diagnosticOnly')
assert r['compareExactAllocationIndex'] and r['modeOrder'] == 'rotating forward/reverse cycles'
result = {'measurementSha256': hashlib.sha256(data).hexdigest(),
          'scope': __doc__, 'samples': {}}
for sample in sorted({t['sample'] for t in r['trials']}):
    rows = [t for t in r['trials'] if t['sample'] == sample]
    modes = {t['mode'] for t in rows}
    rounds = {t['round'] for t in rows}
    assert 'live' in modes and len(rounds) % (2 * len(modes)) == 0
    assert rounds == set(range(len(rounds)))
    assert all(t['passed'] and t['cpuSeconds'] > 0 for t in rows)
    assert len({(t['allocations'], t['allocatedBytes']) for t in rows}) == 1
    positions = collections.Counter()
    keyed = {}
    for rd in sorted(rounds):
        trial = [t for t in rows if t['round'] == rd]
        assert len(trial) == len(modes) and {t['mode'] for t in trial} == modes
        for position, t in enumerate(trial):
            positions[t['mode'], position] += 1
            keyed[t['mode'], rd] = t
    assert len(positions) == len(modes) ** 2
    assert set(positions.values()) == {len(rounds) // len(modes)}
    detail = {'rounds': len(rounds), 'positionCountPerMode': len(rounds) // len(modes), 'modes': {}}
    for mode in sorted(modes):
        trials = [keyed[mode, rd] for rd in sorted(rounds)]
        ratios = [keyed[mode, rd]['cpuSeconds'] / keyed['live', rd]['cpuSeconds'] for rd in sorted(rounds)]
        detail['modes'][mode] = {
            'cpuMedianSeconds': statistics.median(t['cpuSeconds'] for t in trials),
            'cpuPairedRatioMedian': statistics.median(ratios),
            'cpuPairedRatioQuartiles': statistics.quantiles(ratios, n=4, method='inclusive'),
            'allocations': trials[0]['allocations'], 'allocatedBytes': trials[0]['allocatedBytes'],
            'lookupCounts': sorted({tuple(t['LOOKUPS']) for t in trials})}
    result['samples'][sample] = detail
a.output.write_text(json.dumps(result, indent=2) + '\n')

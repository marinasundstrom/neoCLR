"""Validate and pool owner-operation histograms; quantiles are bucket intervals."""
import argparse
import hashlib
import json
from pathlib import Path

NAMES = ['collect-call', 'begin', 'finish', 'cancel', 'guarded-write', 'outermost', 'outermost-gc']

def parse(stderr):
    lines = stderr.splitlines() if isinstance(stderr, str) else stderr
    rows = {}
    for line in lines:
        if not line.startswith('OWNER_PAUSE '):
            continue
        v = list(map(int, line.split()[1:]))
        assert len(v) == 69 and all(n >= 0 for n in v)
        kind, count, total, maximum, nonzero, *bins = v
        assert kind in range(7) and kind not in rows
        assert sum(bins) == count and nonzero <= count
        assert (count == 0 and total == maximum == 0) or (count > 0 and maximum <= total <= count * maximum)
        low = sum(n * (0 if b == 0 else 1 << b) for b, n in enumerate(bins))
        high = sum(n * ((1 << (b + 1)) - 1) for b, n in enumerate(bins))
        assert low <= total <= high
        if count:
            last = max(b for b, n in enumerate(bins) if n)
            assert (0 if last == 0 else 1 << last) <= maximum < 1 << (last + 1)
        rows[kind] = dict(count=count, totalNs=total, maximumNs=maximum, nonzeroResults=nonzero, bins=bins)
    assert set(rows) == set(range(7)) and rows[5]['count'] > 0
    assert rows[6]['count'] <= rows[5]['count'] and rows[6]['totalNs'] <= rows[5]['totalNs']
    assert rows[5]['totalNs'] <= sum(rows[k]['totalNs'] for k in range(5))
    return rows

def quantile(bins, numerator):
    count = sum(bins)
    if not count:
        return None
    rank = max(1, (count * numerator + 99) // 100)
    accumulated = 0
    for b, n in enumerate(bins):
        accumulated += n
        if accumulated >= rank:
            return dict(lowerNs=0 if b == 0 else 1 << b, upperExclusiveNs=1 << (b + 1))
    raise AssertionError('missing quantile')

if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--inputs', type=Path, nargs='+', required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    result = dict(passed=False, scope=__doc__, groups={}, inputs={})
    for path in a.inputs:
        data = json.loads(path.read_text())
        assert data['passed'] and data.get('diagnosticOnly')
        for trial in data['trials']:
            assert trial['exitCode'] == 0
            key = trial.get('sample', data.get('sample')) + '/' + trial['mode']
            group = result['groups'].setdefault(key, dict(trials=0, operations={}))
            rows = parse(trial['stderr'])
            if 'candidateCounts' in trial:
                counts = trial['candidateCounts']
                assert all(rows[k]['nonzeroResults'] == 0 for k in [1, 2])
                assert [rows[k]['count'] for k in [1, 2, 3]] == counts[:3]
            group['trials'] += 1
            for kind, row in rows.items():
                dest = group['operations'].setdefault(NAMES[kind], dict(count=0, totalNs=0, maximumNs=0, nonzeroResults=0, bins=[0]*64))
                for field in ['count', 'totalNs', 'nonzeroResults']:
                    dest[field] += row[field]
                dest['maximumNs'] = max(dest['maximumNs'], row['maximumNs'])
                dest['bins'] = [x+y for x,y in zip(dest['bins'],row['bins'])]
        result['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
    for group in result['groups'].values():
        for row in group['operations'].values():
            row['quantileBounds'] = {str(p):quantile(row['bins'],p) for p in [50,95,99]}
    result['passed'] = True
    a.output.write_text(json.dumps(result,indent=2)+'\n')

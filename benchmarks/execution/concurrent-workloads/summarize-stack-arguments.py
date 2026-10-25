"""Validate matched code-only comparisons and summarize retained stack-elision trials."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import statistics

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--root', type=Path, required=True)
p.add_argument('--prefix', default='stack-elision')
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
result = dict(passed=False, cohorts={}, inputs={})

def normalized(command):
    c = command.copy()
    c[c.index('-o') + 1] = '<executable>'
    return ['<managed-object>' if x.endswith('.o') else x for x in c]

for cohort, suffix in [('ordinary', 'ordinary-measure'), ('plain', 'http-plain'), ('json', 'http-json')]:
    filename = a.prefix + '-' + suffix
    path = a.root / filename / 'results.json'
    data = json.loads(path.read_text())
    assert data['passed'] and not data.get('diagnosticOnly')
    trials = data['trials']
    ordinary = cohort == 'ordinary'
    assert trials
    if ordinary:
        assert all(t['passed'] for t in trials)
    else:
        for t in trials:
            assert t['exitCode'] == 0 and t['gcProfile']['failures'] == 0
            done = next(line.split() for line in t['stdout'] if line.startswith('DONE '))
            assert int(done[2]) == t['requests'] + t['warmup'] and int(done[3]) == 0
            if t['mode'] != 'sync':
                counts = t['candidateCounts']
                assert counts[0] == counts[1] + counts[2] and counts[5] == 0
                assert t['indexMetadata'][1] == 0 and t['storeCounts'][4] == 0
                assert counts == [0] * 6 if t['mode'].startswith('guarded-sync') else counts[1] > 0
    samples = sorted({t['sample'] for t in trials}) if ordinary else [cohort]
    groups = {}
    for sample in samples:
        rows = [t for t in trials if t.get('sample', cohort) == sample]
        modes = sorted({t['mode'] for t in rows})
        assert len(modes) == 5
        if ordinary:
            assert data['allocationEquality']
            ranks = {m: Counter() for m in modes}
            for round_id in sorted({t['round'] for t in rows}):
                ordered = [t for t in rows if t['round'] == round_id]
                assert len(ordered) == 5 and {t['mode'] for t in ordered} == set(modes)
                for rank, t in enumerate(ordered): ranks[t['mode']][rank] += 1
            assert all(set(counts) == set(range(5)) and set(counts.values()) == {2} for counts in ranks.values())
        groups[sample] = {}
        for mode in modes:
            selected = [t for t in rows if t['mode'] == mode]
            fields = ['cpuSeconds', 'wallSeconds', 'peakLiveBytes', 'peakArenaUsed'] if ordinary else ['requestsPerSecond', 'processCpuSeconds', 'peakRssBytes']
            groups[sample][mode] = {f: statistics.median(t[f] for t in selected) for f in fields}
            groups[sample][mode]['allocationsRange'] = [min(t['allocations'] for t in selected), max(t['allocations'] for t in selected)]
            allocated = [t['allocatedBytes'] if ordinary else t['gcProfile']['allocatedBytes'] for t in selected]
            groups[sample][mode]['allocatedBytesRange'] = [min(allocated), max(allocated)]
        for prefix in ['live', 'guarded-sync']:
            new = prefix + ('-hybrid-inline' if ordinary else '-inline-index')
            old = prefix + ('-previous-hybrid-inline' if ordinary else '-previous-inline-index')
            def command(mode):
                suffix = sample + '-' + mode if ordinary else mode
                return next(c['command'] for c in data['commands'] if '-o' in c['command'] and Path(c['command'][c['command'].index('-o') + 1]).name == suffix)
            assert normalized(command(new)) == normalized(command(old)), 'Native controls differ beyond generated object'
            key = 'round' if ordinary else 'pair'
            old_rows = {t[key]: t for t in rows if t['mode'] == old}
            new_rows = {t[key]: t for t in rows if t['mode'] == new}
            assert len(old_rows) == len(new_rows) and old_rows.keys() == new_rows.keys()
            cpu = 'cpuSeconds' if ordinary else 'processCpuSeconds'
            metric = 'wallSeconds' if ordinary else 'requestsPerSecond'
            groups[sample][new]['pairedCpuRatio'] = statistics.median(new_rows[i][cpu] / old_rows[i][cpu] for i in old_rows)
            groups[sample][new]['pairedTimeOrThroughputRatio'] = statistics.median(new_rows[i][metric] / old_rows[i][metric] for i in old_rows)
    result['cohorts'][cohort] = dict(trials=len(trials), samples=groups, allocationEquality=data['allocationEquality'])
    if not ordinary:
        result['cohorts'][cohort]['checkedResponses'] = sum(t['requests'] + t['warmup'] for t in trials)
    result['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
result['passed'] = True
a.output.write_text(json.dumps(result, indent=2) + '\n')

#!/usr/bin/env python3
"""Normalize comparable work counters without mixing invocation and request units."""
import argparse
import hashlib
import json
from pathlib import Path

p = argparse.ArgumentParser(description=__doc__)
for name in ['plain-small', 'plain-large', 'json-small', 'json-large', 'corpus']:
    p.add_argument('--'+name, type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
metrics = ['transientPublications', 'boundaries', 'allocations', 'payloadBytes', 'collections']
inputs = {}
reports = {}
for name in ['plain_small', 'plain_large', 'json_small', 'json_large', 'corpus']:
    path = getattr(a, name).resolve()
    data = path.read_bytes()
    report = json.loads(data)
    assert report['passed'] or (name == 'corpus' and report['cases'] and report['commands'][-1]['exitCode'] != 0), path
    inputs[str(path)] = hashlib.sha256(data).hexdigest()
    reports[name] = report
# Match native runtime hashes across every report that records them. Diagnostic
# generated source and compiler identities are retained in the underlying reports.
identities = {}
for report in reports.values():
    for path, digest in report['inputs'].items():
        if '/runtime/native/' in path:
            assert identities.get(path, digest) == digest, path
            identities[path] = digest
for leaf in ['diagnostic-probe.c', 'diagnostic-gc.c']:
    hashes = {digest for report in reports.values() for path, digest in report['inputs'].items() if path.endswith('/'+leaf)}
    assert len(hashes) == 1, (leaf, hashes)
http = {}
for name in ['plain', 'json']:
    small, large = reports[name+'_small'], reports[name+'_large']
    assert small['paced'] and large['paced']
    assert small['json'] == large['json'] == (name == 'json')
    added = len(large['responses']) - len(small['responses'])
    assert added > 0
    assert small['inputs'].get(next(p for p in small['inputs'] if p.endswith('.o'))) == large['inputs'].get(next(p for p in large['inputs'] if p.endswith('.o')))
    http[name] = dict(unit='additional checked HTTP response', additionalResponses=added,
                      counts={k:(large[k]-small[k])/added for k in metrics})
    assert all(v >= 0 for v in http[name]['counts'].values())
ordinary = [dict(sample=c['sample'], unit='complete sample invocation including setup/teardown',
                 counts={k:c[k]/c['invocations'] for k in metrics}, objectBytes=c['objectBytes'])
            for c in reports['corpus']['cases']]
result = dict(passed=True, inputs=inputs, http=http, ordinary=ordinary,
              corpusComplete=reports['corpus']['passed'],
              excluded=[] if reports['corpus']['passed'] else [dict(sample='json-dom-stream', reason=reports['corpus']['commands'][-1]['stderr'])],
              limits=['Counter diagnostics use native sanitizers, one owner and paced GC; not capacity measurements.',
                      'HTTP finite differences subtract fixed setup/teardown; pacing and scheduling introduce rounding/interleaving effects.',
                      'Different ordinary programs do different amounts of work; invocation counts are not normalized application operations.',
                      'Publication counts identify metadata-processing frequency, not metadata-load bytes, resolution cache misses or CPU attribution.'])
a.output.parent.mkdir(parents=True, exist_ok=True)
a.output.write_text(json.dumps(result, indent=2)+'\n')
print(json.dumps(dict(http=http, ordinary=ordinary), indent=2))

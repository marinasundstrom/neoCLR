#!/usr/bin/env python3
"""Print paired-cohort medians; keep readiness separate from warm execution."""
import argparse
import gzip
import json
from pathlib import Path
from statistics import median


def read(path):
    data = Path(path).read_bytes()
    result = json.loads(gzip.decompress(data) if str(path).endswith('.gz') else data)
    if not result['passed']:
        raise ValueError('Incomplete experiment')
    return result


p = argparse.ArgumentParser(description=__doc__)
p.add_argument('ordinary')
p.add_argument('http')
a = p.parse_args()
ordinary, http = read(a.ordinary), read(a.http)
summary = []
for sample in ['tasks', 'collections']:
    for mode, field in [('interpreter', 'wallSeconds'), ('jit', 'selectionAndVerificationSeconds')]:
        medians = {}
        for variant in ['baseline', 'candidate']:
            rows = [x for x in ordinary['runs'] if x['sample'] == sample
                    and x['mode'] == mode and x['variant'] == variant]
            assert len(rows) == 3
            medians[variant] = median(x[field] if mode == 'interpreter' else x['jit'][field] for x in rows)
        summary.append(dict(sample=sample, mode=mode, field=field, seconds=medians,
                            reductionPercent=100*(1-medians['candidate']/medians['baseline'])))
medians = {variant: median(x['readySeconds'] for x in http['trials'] if x['variant'] == variant)
           for variant in ['baseline', 'candidate']}
summary.append(dict(sample='http', mode='jit', field='readySeconds', seconds=medians,
                    reductionPercent=100*(1-medians['candidate']/medians['baseline'])))
print(json.dumps(summary, indent=2))

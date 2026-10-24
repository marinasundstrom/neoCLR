#!/usr/bin/env python3
"""Check lookup-counter invariants and summarize uninstrumented paired capacity."""
import json
import statistics
import sys


def counters(report):
    if not report.get('passed'):raise ValueError('incomplete counters')
    rows=[]
    for row in report['trials']:
        if row['exitCode'] or len(row['counters'])!=len(report['fields']):raise ValueError('invalid trial')
        c=dict(zip(report['fields'],row['counters']))
        if any(v<0 for v in c.values()):raise ValueError('negative counter')
        if c['base_cache_hits']+c['base_walks']>c['base_calls']:raise ValueError('base count mismatch')
        if not c['newly_marked']<=c['payload_matches']<=c['mark_candidates']<=c['index_queries']:raise ValueError('mark count mismatch')
        if c['index_probes']<c['index_queries']-c['index_outside']:raise ValueError('index count mismatch')
        rows.append(dict(mode=row['mode'],count=row.get('count'),sample=row.get('sample',report.get('sample')),counters=c))
    return rows


def capacity(report):
    if not report.get('passed'):raise ValueError('incomplete capacity')
    groups={}
    for row in report['trials']:
        if row['exitCode']:raise ValueError('failed server')
        groups.setdefault(row['label'],[]).append(row)
    if len(groups)!=4 or any(len(rows)!=2 for rows in groups.values()):raise ValueError('incomplete cohort')
    return {label:dict(requestsPerSecond=statistics.mean(x['trial']['requestsPerSecond'] for x in rows),
                       cpuMicrosecondsPerResponse=statistics.mean(x['cpuMicrosecondsPerResponse'] for x in rows))
            for label,rows in groups.items()}


if __name__=='__main__':
    report=json.load(open(sys.argv[1]))
    print(json.dumps(counters(report) if 'fields' in report else capacity(report),indent=2))

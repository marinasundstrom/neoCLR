#!/usr/bin/env python3
"""Summarize paired capacity without mixing worker and entry-host CPU windows."""
import argparse
import gzip
import json
import statistics
from pathlib import Path


def read(path):
    path=Path(path)
    return json.loads(gzip.decompress(path.read_bytes()) if path.suffix=='.gz' else path.read_bytes())


def pair(rows, key, baseline, candidate):
    result={}
    for name in [baseline,candidate]:
        selected=[r for r in rows if r[key]==name]
        if len(selected)!=2:
            raise ValueError('expected exactly two trials per configuration')
        rates=[r.get('trial',r)['requestsPerSecond'] for r in selected]
        cpus=[r['cpuMicrosecondsPerResponse'] for r in selected]
        if min(rates)<=0 or min(cpus)<=0:
            raise ValueError('nonpositive capacity or CPU')
        if any(r.get('exitCode',0)!=0 for r in selected):
            raise ValueError('failed execution')
        result[name]=dict(requestsPerSecond=statistics.mean(rates),range=[min(rates),max(rates)],
                          cpuMicrosecondsPerResponse=statistics.mean(cpus),cpuRange=[min(cpus),max(cpus)],
                          p99Milliseconds=[r.get('trial',r)['latencyMilliseconds']['p99'] for r in selected])
    b=result[baseline];c=result[candidate]
    result['throughputGainPercent']=(c['requestsPerSecond']/b['requestsPerSecond']-1)*100
    result['cpuReductionPercent']=(1-c['cpuMicrosecondsPerResponse']/b['cpuMicrosecondsPerResponse'])*100
    return result


def summarize(aot,jit):
    if not aot['passed'] or not jit['passed'] or jit['profile']:
        raise ValueError('failed or profiled cohort')
    if len(aot['trials'])!=8 or len(jit['trials'])!=4:
        raise ValueError('incomplete cohort')
    return dict(aotOneOwner=pair(aot['trials'],'name','baseline-w1','candidate-w1'),
                aotFourOwners=pair(aot['trials'],'name','baseline-w4','candidate-w4'),
                jitOneOwner=pair(jit['trials'],'label','baseline','candidate'))


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('aot',type=Path);p.add_argument('jit',type=Path);p.add_argument('output',type=Path)
    a=p.parse_args();a.output.write_text(json.dumps(summarize(read(a.aot),read(a.jit)),indent=2)+'\n')

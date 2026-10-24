#!/usr/bin/env python3
"""Keep startup, capacity, and wall-sample denominators separate."""
import argparse
from collections import Counter
import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import statistics


def read(path):
    path=Path(path)
    return json.loads(gzip.decompress(path.read_bytes()) if path.suffix=='.gz' else path.read_bytes())


def startup(report):
    if not report['passed']:
        raise ValueError('failed startup cohort')
    result=[]
    for sample in ['tasks','collections']:
        for mode in ['interpreter','jit','aot']:
            rows=[r for r in report['runs'] if r['sample']==sample and r['mode']==mode]
            if len(rows)!=3 or any(r['exitCode'] for r in rows):
                raise ValueError('incomplete startup cohort')
            row=dict(sample=sample,mode=mode,
                     medianWallSeconds=statistics.median(r['wallSeconds'] for r in rows),
                     wallRange=[min(r['wallSeconds'] for r in rows),max(r['wallSeconds'] for r in rows)],
                     medianCpuSeconds=statistics.median(r['cpuSeconds'] for r in rows))
            if mode=='jit':
                if any(r['jit']['guestExitCode'] for r in rows):
                    raise ValueError('guest failure')
                row['phases']={k:statistics.median(r['jit'][k] for r in rows) for k in [
                    'metadataLoadSeconds','selectionAndVerificationSeconds','compileAndFinalizeSeconds','invocationSeconds']}
            result.append(row)
    return result


def peers(report):
    if not report['passed']:
        raise ValueError('failed capacity cohort')
    result={}
    for name in ['neoclr-1','neoclr-4','go-1','go-4','kestrel-1','kestrel-4','node-1']:
        rows=[r for r in report['trials'] if r['name']==name]
        if len(rows)!=2:
            raise ValueError('incomplete capacity cohort')
        cpus=[]
        for r in rows:
            responses=sum(t['completed']+t['drained'] for t in [r,r['warmup']])
            if responses<=0 or r['processCpuSeconds']<0:
                raise ValueError('invalid CPU denominator')
            cpus.append(r['processCpuSeconds']*1e6/responses)
        rates=[r['requestsPerSecond'] for r in rows]
        result[name]=dict(requestsPerSecond=statistics.mean(rates),range=[min(rates),max(rates)],
                          cpuMicrosecondsPerResponse=statistics.mean(cpus),
                          p99Milliseconds=[r['latencyMilliseconds']['p99'] for r in rows])
    return result


def category(name):
    if name.startswith('neoclr_value_') or '[neoclr_value_' in name:
        return 'managed generated code'
    if name=='_tlv_get_addr':
        return 'TLS resolution'
    if name in {'stack_roots','neoclr_gc_enter_owned_v1','neoclr_gc_enter_v1','neoclr_gc_leave_v1',
                'neoclr_gc_stack_map_v1','neoclr_gc_transient_owned_v1','neoclr_gc_transient_v1',
                'neoclr_gc_published_boundary_v1','neoclr_gc_boundary_v1','validate_storage','observe_fault'}:
        return 'root publication and frame bookkeeping'
    if name in {'collect_indexed','collect_conditional','neoclr_gc_collect_conditional_v1','mark_frames','mark','build_index','index_find'}:
        return 'collector'
    if name in {'validated_bases','neoclr_gc_allocate_v1','neoclr_gc_allocation_info_v1','reserve_values','allocate_object'}:
        return 'allocation and object admission'
    if name=='neoclr_native_stack_check_v1':
        return 'native stack checks'
    if name in {'__accept','accept','__recvfrom','__sendto','send','close','__close','socket_poll','poll','__poll','kevent','__kevent'}:
        return 'socket and poll leaves'
    return 'other (retained separately)'


def groups(counts):
    result=Counter()
    if any(n<0 for n in counts.values()) or not sum(counts.values()):
        raise ValueError('invalid samples')
    for name,count in counts.items():
        result[category(name)]+=count
    total=sum(counts.values())
    return dict(total=total,scope='Exclusive wall-clock self samples; not CPU shares or call counts.',
                groups={k:dict(samples=v,percent=100*v/total) for k,v in result.most_common()})


def lookup(report):
    if not report['passed']:
        raise ValueError('failed lookup cohort')
    result={}
    for mode in ['aot','jit']:
        rows=[r for r in report['trials'] if r['mode']==mode]
        if len(rows)!=2 or sorted(r['count'] for r in rows)!=[4,12]:
            raise ValueError('incomplete lookup cohort')
        small,large=sorted(rows,key=lambda r:r['count'])
        if any(r['exitCode'] or len(r['counters'])!=len(report['fields']) for r in rows):
            raise ValueError('invalid lookup row')
        result[mode]={k:(b-a)/8 for k,a,b in zip(report['fields'],small['counters'],large['counters'])}
    return result


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--directory',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True)
    a=p.parse_args()
    d=a.directory
    r=dict(startup=startup(read(d/'startup/results.json')),peers=peers(read(d/'peers/comparison.json')))
    root=Path(__file__).resolve().parents[3]
    spec=importlib.util.spec_from_file_location('symbols',root/'benchmarks/execution/jit-layout/symbolize.py')
    symbols=importlib.util.module_from_spec(spec);spec.loader.exec_module(symbols)
    r['startupProfiles']={mode:symbols.self_samples((d/'startup'/f'{mode}.sample.txt').read_text()) for mode in ['interpreter','jit']}
    native=read(d/'worker-analysis/summary.json')
    r['workerProfile']=groups({x['symbol']:x['count'] for x in native['workerSelfSamples']})
    r['workerProfile']['topLeaves']=native['workerSelfSamples'][:25]
    r['workerProfile']['coordinatorSamples']=sum(t['samples'] for t in native['threads'] if t['role']=='coordinator')
    r['workerProfile']['source']='worker-analysis/summary.json'
    r['lookupPerAdditionalResponse']={label:lookup(read(d/(label+'-counts/results.json'))) for label in ['plain','json']}
    gc=read(d/'gc-timing/profile.json')
    if not gc['passed'] or gc['gc']['failures']:
        raise ValueError('failed GC timing cohort')
    r['gcTiming']=dict(perResponse=gc['perResponse'],totals=gc['gc'])
    paths=[d/'startup/results.json',d/'peers/comparison.json',d/'worker-analysis/summary.json',
           d/'plain-counts/results.json',d/'json-counts/results.json',d/'gc-timing/profile.json',Path(__file__)]
    r['inputs']={str(x):hashlib.sha256(x.read_bytes()).hexdigest() for x in paths}
    a.output.write_text(json.dumps(r,indent=2)+'\n')

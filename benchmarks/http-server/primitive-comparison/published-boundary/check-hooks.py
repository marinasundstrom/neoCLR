#!/usr/bin/env python3
"""Check that only ordinary paced publication reuses the current-head proof."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=Path, required=True)
a=p.parse_args()
a.output.mkdir(parents=True, exist_ok=False)
root=Path(__file__).resolve().parents[4]
source=root/'runtime/native/gc/root-probe.c'
report=dict(passed=False, checks=[], inputs={str(source):hashlib.sha256(source.read_bytes()).hexdigest()})
try:
    for mode, flags, expected in [
        ('paced', ['-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PACED'], 'neoclr_gc_published_boundary_v1'),
        ('instrumented', ['-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PACED','-DNEOCLR_GC_SITE_PROFILE'], 'neoclr_gc_boundary_v1'),
        ('forced', ['-DNEOCLR_NATIVE_GC'], None),
        ('diagnostic', [], None)]:
        obj=a.output/(mode+'.o')
        cmd=['xcrun','--sdk','macosx','clang','-std=c11','-O2','-Wall','-Wextra','-Werror',*flags,'-c',str(source),'-o',str(obj)]
        result=subprocess.run(cmd,capture_output=True,text=True,timeout=60)
        result.check_returncode()
        symbols=subprocess.run(['nm','-u',str(obj)],capture_output=True,text=True,check=True).stdout
        for symbol in ['neoclr_gc_boundary_v1','neoclr_gc_published_boundary_v1']:
            assert (symbol in symbols)==(symbol==expected), (mode,symbol,symbols)
        report['checks'].append(dict(mode=mode,command=cmd,exitCode=result.returncode,stderr=result.stderr,undefinedSymbols=symbols))
    report['passed']=True
finally:
    (a.output/'validation.json').write_text(json.dumps(report,indent=2)+'\n')

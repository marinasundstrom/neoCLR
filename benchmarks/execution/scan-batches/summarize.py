#!/usr/bin/env python3
"""Summarize both rejected AOT variants using the established paired arithmetic."""
import argparse,json,runpy
from pathlib import Path
root=Path(__file__).resolve().parents[3]
helpers=runpy.run_path(str(root/'benchmarks/execution/owned-root-chain/summarize.py'))
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('initial',type=Path);p.add_argument('adjusted',type=Path);p.add_argument('output',type=Path);a=p.parse_args()
r={}
for name,path in [('initial',a.initial),('adjusted',a.adjusted)]:
 report=helpers['read'](path)
 if not report['passed'] or len(report['trials'])!=8:raise ValueError('incomplete or failed cohort')
 r[name]={f'aot{owners}Owner':helpers['pair'](report['trials'],'name',f'baseline-w{owners}',f'candidate-w{owners}') for owners in [1,4]}
a.output.write_text(json.dumps(r,indent=2)+'\n')

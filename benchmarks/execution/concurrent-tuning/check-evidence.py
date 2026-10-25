"""Validate tuning report policy selection and successful lifetime accounting."""
import argparse
import json
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('reports',type=Path,nargs='+');a=p.parse_args()
for path in a.reports:
 r=json.loads(path.read_text());assert r['passed']
 for command in r['commands']:
  cmd=command['command']
  if '-o' not in cmd:continue
  mode=Path(cmd[cmd.index('-o')+1]).name
  if mode not in r.get('budgetsByMode',{}):continue
  policy=r['budgetsByMode'][mode]
  for key,value in [('ALLOCATION',policy['allocations']),('PUBLICATION',policy['publications'])]:
   assert [x for x in cmd if x.startswith('-DNEOCLR_GC_'+key+'_BUDGET=')]==[f'-DNEOCLR_GC_{key}_BUDGET={value}']
  heap=[x for x in cmd if x.startswith('-DNEOCLR_HTTP_HEAP_BYTES=')]
  assert heap==[f'-DNEOCLR_HTTP_HEAP_BYTES={1048576 if mode=="sync" else r["liveHeapBytes"]}']
 for t in r['trials']:
  assert t['exitCode']==0 and t['gcProfile']['failures']==0
  if t['mode']=='sync':continue
  c=t['candidateCounts'];assert c[0]==c[1]+c[2] and c[5]==0
  assert t['indexMetadata'][1]==0 and t['nativeScopeCounts'][4]==0
 print('PASS',path)

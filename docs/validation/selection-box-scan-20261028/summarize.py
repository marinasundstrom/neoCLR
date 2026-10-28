from pathlib import Path
import json,statistics,hashlib
work=Path('/Users/robert/Projects/neoclr/target/selection-box-scan-20261028')
p=work/'paired-http/results.json';data=json.loads(p.read_text());assert data['passed']
r={'scope':'Three alternating baseline/hoisted-box-scan process pairs per eager/lazy and sync/concurrent mode. Local JSON pipeline; production-default backend. Ranges describe observed variation, not confidence intervals.','rows':[],'inputs':{str(p):hashlib.sha256(p.read_bytes()).hexdigest()}}
def stats(xs):return {'median':statistics.median(xs),'min':min(xs),'max':max(xs),'values':xs}
for mode in ['eager-sync-jit','lazy-sync-jit','eager-live-jit','lazy-live-jit']:
 trials={variant:sorted([t for t in data['trials'] if t['mode']==variant+'-'+mode],key=lambda t:t['round']) for variant in ['baseline','indexed']}
 assert all(len(x)==3 for x in trials.values())
 row={'mode':mode,'metrics':{}}
 for name in ['readySeconds','firstResponseSeconds','readyThroughFirstResponseSeconds','wholeProcessCpuSeconds','requestsPerSecond','selectionAndVerificationSeconds','compileAndFinalizeSeconds']:
  values={v:[t[name] if name in t else t['jit'][name] for t in ts] for v,ts in trials.items()}
  row['metrics'][name]={v:stats(xs) for v,xs in values.items()}
  row['metrics'][name]['pairedChangePercent']=stats([100*(b/a-1) for a,b in zip(values['baseline'],values['indexed'])])
 for variant,ts in trials.items():
  assert all(t['passed'] and t['exitCode']==0 and t['jit']['guestExitCode']==0 for t in ts)
  if mode.startswith('lazy'):
   assert all(t['jit']['lazyJit']['uncompiledMethods']==811 for t in ts)
 r['rows'].append(row)
r['checkedResponses']=sum(t['requests']+t['warmup'] for t in data['trials'])
(work/'summary.json').write_text(json.dumps(r,indent=2)+'\n')
print('| Mode | Ready before → after | Selection/verification before → after | Warm req/s before → after |')
print('| --- | ---: | ---: | ---: |')
for row in r['rows']:
 vals=[]
 for key in ['readySeconds','selectionAndVerificationSeconds','requestsPerSecond']:
  m=row['metrics'][key];digits=0 if key=='requestsPerSecond' else 3
  vals.append(f"{m['baseline']['median']:,.{digits}f} → {m['indexed']['median']:,.{digits}f}")
 print('| '+row['mode']+' | '+' | '.join(vals)+' |')

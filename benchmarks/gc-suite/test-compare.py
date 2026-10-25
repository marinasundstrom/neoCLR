"""Reject mismatched sample definitions and diagnostic/performance comparisons."""
import copy,importlib.util
from pathlib import Path
spec=importlib.util.spec_from_file_location('compare',Path(__file__).with_name('compare.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
a=dict(passed=True,schemaVersion=1,kind='benchmark',suite='all',sampleDefinitionSha256='same',system='same',hardware={'cpu':'same','logicalCpus':8},requests=4096,policies={'sync':{'budget':256},'concurrent':{'budget':1024}},rows=[dict(sample='steady',collector=c,cpuSeconds=2,unitsPerSecond=10,allocations=100,peakArenaBytes=200,unit='items') for c in ['sync','concurrent']])
b=copy.deepcopy(a);b['rows'][0]['cpuSeconds']=1;b['rows'][1]['cpuSeconds']=3;b['policies']['concurrent']['budget']=2048
r=m.compare(a,b);assert [x['cpuRatio'] for x in r['rows']]==[.5,1.5];assert [x['policyChanged'] for x in r['rows']]==[False,True]
for key,value in [('kind','profile'),('sampleDefinitionSha256','other'),('system','other'),('hardware',{'cpu':'other'}),('requests',8192),('passed',False)]:
 bad=copy.deepcopy(b);bad[key]=value
 try:m.compare(a,bad)
 except AssertionError:pass
 else:raise AssertionError(key)
bad=copy.deepcopy(b);bad['rows'].pop()
try:m.compare(a,bad)
except AssertionError:pass
else:raise AssertionError('missing collector accepted')
print('PASS same-collector ratios, explicit policy changes, profile/sample/host/work mismatch rejection')

"""Map failed live selections to the matching retained ordinary selection metadata."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import re
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--baseline',type=Path,required=True)
p.add_argument('--current',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args()
def read(path):
 data=path.read_bytes()
 return json.loads(gzip.decompress(data) if path.suffix=='.gz' else data)
baseline=read(a.baseline);current=read(a.current)
r={'scope':'Compiler admission only; not execution or performance evidence.', 'samples':[],
   'inputs':{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [a.baseline,a.current,Path(__file__)]}}
for item in current['commands']:
 if item['mode']!='live':continue
 old=next(c for c in baseline['commands'] if c['sample']==item['sample'] and c['mode']=='sync')
 assert old['exitCode']==0
 left=old['selectionCommand'].copy();right=item['selectionCommand'].copy()
 left[4]=right[4]='<output>'
 assert left==right, 'Selection arguments differ'
 for path,digest in baseline['inputs'].items():
  if Path(path).suffix in ['.dll','.neox','.json']:
   assert current['inputs'].get(path)==digest, 'Selection input changed: '+path
 text=old['stdout'];report,_=json.JSONDecoder().raw_decode(text[text.index('{'):])
 rows={f['compiledIndex']:f for f in report['functions']}
 rejected=[]
 for index,name in re.findall(r'function (\d+) \(([^)]+)\)',item['stderr']):
  row=rows.get(int(index))
  if row is None:
   rejected.append({'compiledIndex':int(index),'name':name,'mapping':'Absent from ordinary selection metadata; name from live diagnostic.'})
  else:
   assert row.get('compiledName',row['name'])==name
   rejected.append(row)
 categories={}
 for row in rejected:
  index=row['compiledIndex'];matches=[]
  for key,field in [('interfaceDispatch','contractCompiledIndex'),('classVirtualDispatch','thunkCompiledIndex'),('int32Boxes','helperCompiledIndex'),('booleanBoxes','helperCompiledIndex'),('scalarBoxQueries','helperCompiledIndex')]:
   if any(x.get(field)==index for x in report.get(key,[])):matches.append(key)
  row['selectionCategories']=matches
  category='+'.join(matches) or 'other'
  categories[category]=categories.get(category,0)+1
 r['samples'].append({'sample':item['sample'],'exitCode':item.get('exitCode'),'categories':categories, 'rejectedBodies':rejected,'stderr':item['stderr']})
a.output.write_text(json.dumps(r,indent=2)+'\n')

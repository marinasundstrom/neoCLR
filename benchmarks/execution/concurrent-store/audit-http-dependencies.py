"""Compare retained HTTP object imports with current live provider admission.

This is dependency evidence, not compilation, execution or a provider safety proof.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();root=Path(__file__).resolve().parents[3]
a.output.mkdir(parents=True,exist_ok=False)
provider_file=root/'runtime/codegen/src/mutation_admission.rs'
source=provider_file.read_text().split('pub(crate) fn root_providers',1)[1]
providers=set(re.findall(r'"((?:neoclr|experiment)_[a-z0-9_]+)"',source))
replacements={
 'neoclr_static_storage_rooted_v1':'experiment_gc_static_storage_rooted_v1',
 'neoclr_string_slice_utf8_rooted_v1':'experiment_gc_string_slice_utf8_rooted_v1',
 'neoclr_gc_entry_check_v1':'experiment_gc_prepare_entry_v1',
}
legacy={'neoclr_entry_callback_read_v1','neoclr_entry_tasks_begin_v1','neoclr_entry_tasks_end_v1','neoclr_entry_tasks_poll_v1'}
r={'scope':__doc__,'samples':{},'inputs':{}}
def record(path):
 r['inputs'][str(path.relative_to(root))]=hashlib.sha256(path.read_bytes()).hexdigest()
record(provider_file)
record(root/'runtime/codegen/src/values.rs')
record(root/'runtime/codegen/src/fault_details.rs')
record(Path(__file__).resolve())
for name,report in [('plain','codegen.json'),('json','json-codegen.json')]:
 obj=root/f'target/state-integration/final-objects/{name}.o'
 meta=root/f'target/root-map/{report}'
 selection=json.loads(meta.read_text()) if meta.exists() else {};bindings=selection.get('nativeBindings',[])
 cmd=['nm','-u',str(obj)];x=subprocess.run(cmd,capture_output=True,text=True,check=True)
 imports=sorted({line.split()[-1].removeprefix('_') for line in x.stdout.splitlines() if line.strip()})
 async_selected='neoclr_runtime_async_pump_v1' in imports
 rows=[]
 for symbol in imports:
  replacement=replacements.get(symbol)
  if symbol in providers:status='exact provider name admitted; not whole-program qualification'
  elif replacement in providers:status='live compiler substitutes admitted provider'
  elif symbol in legacy and async_selected:status='unused legacy pump import omitted by live compiler'
  else:status='not admitted'
  rows.append(dict(symbol=symbol,status=status,**({'replacement':replacement} if replacement else {})))
 missing=[x['symbol'] for x in rows if x['status']=='not admitted']
 r['samples'][name]=dict(object=str(obj.relative_to(root)),selection=str(meta.relative_to(root)) if meta.exists() else None,command=cmd,
  selectedFunctions=len(selection['functions']) if selection else None,selectedTypes=len(selection['types']) if selection else None,imports=rows,
  missing=missing,entryBindings=[b for b in bindings if b.get('implementation')=='entry-task-drain-v1'],
  remainingBindings=[b for b in bindings if b.get('symbol') in missing or b.get('symbol','').replace('_v1','_rooted_v1') in missing])
 record(obj)
 if meta.exists():record(meta)
(a.output/'results.json').write_text(json.dumps(r,indent=2)+'\n')
for name,sample in r['samples'].items():
 print(name, sample['selectedFunctions'],'functions;',len(sample['imports']),'imports;',len(sample['missing']),'not admitted')
 print('\n'.join(sample['missing']))

#!/usr/bin/env python3
"""Compare optimized unprofiled collector functions, ignoring comments and local-label names."""
import argparse,json,subprocess,re,hashlib
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
root=Path(__file__).resolve().parents[3];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
old=out/'baseline.c';old.write_bytes(subprocess.check_output(['git','show','da0509682:runtime/native/gc/native-gc.c']))
recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command'];cc=recipe[:recipe.index('-std=c11')]
r=dict(passed=False,commands=[]);functions={}
try:
 for label,source in [('baseline',old),('candidate',root/'runtime/native/gc/native-gc.c')]:
  assembly=out/(label+'.s');cmd=cc+['-std=c11','-O2','-S','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PACED','-I',str(root/'runtime/native/gc'),str(source),'-o',str(assembly)]
  x=subprocess.run(cmd,capture_output=True,text=True);r['commands'].append(dict(command=cmd,exitCode=x.returncode,stderr=x.stderr));x.check_returncode()
  functions[label]={}
  pools={name:'POOL_'+hashlib.sha256('\n'.join(line.split(';',1)[0].strip() for line in data.splitlines() if line.split(';',1)[0].strip()).encode()).hexdigest()
         for name,data in re.findall(r'^(lCPI\d+_\d+):\n(.*?)\s*\.section',assembly.read_text(),re.M|re.S)}
  for name,body in re.findall(r'^_(\w+):[^\n]*\n(.*?\.cfi_endproc)',assembly.read_text(),re.M|re.S):
   body="\n".join(line.split(";",1)[0].rstrip() for line in body.splitlines() if line.split(";",1)[0].strip())
   body=re.sub(r'lCPI\d+_\d+',lambda m:pools[m[0]],body)
   labels={}
   def local(m):return labels.setdefault(m[0],f'LOCAL_{len(labels)}')
   functions[label][name]=re.sub(r'\bL[A-Za-z_]\w*\b',local,body)
 r['changed']=[name for name,body in functions['baseline'].items() if functions['candidate'].get(name)!=body]
 r['compared']=len(functions['baseline']);r['added']=sorted(set(functions['candidate'])-set(functions['baseline']))
 assert r['compared']>20 and not r['changed'],r
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

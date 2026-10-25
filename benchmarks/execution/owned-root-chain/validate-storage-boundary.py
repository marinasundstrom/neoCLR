"""Compare original and split validators at small/large and invalid-tail boundaries."""
import argparse
import hashlib
import json
from pathlib import Path
import signal
import subprocess
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args()
root=Path(__file__).resolve().parents[3]
base=Path(__file__).resolve().parent
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,commands=[],inputs={})
def run(command,expected=0):
 command=list(map(str,command))
 x=subprocess.run(command,capture_output=True,text=True,timeout=30)
 r['commands'].append(dict(command=command,exitCode=x.returncode,expected=expected,stdout=x.stdout,stderr=x.stderr))
 assert x.returncode==expected and 'runtime error:' not in x.stderr,r['commands'][-1]
try:
 for split in [False,True]:
  for sanitized in [False,True]:
   exe=out/f'check-{split}-{sanitized}'
   flags=['clang','-std=c11','-O2','-Wall','-Wextra','-Werror','-DNEOCLR_NATIVE_GC']
   if split:flags+=['-DNEOCLR_GC_SPLIT_STORAGE_VALIDATION']
   if sanitized:flags+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
   run(flags+[base/'storage-boundary-test.c',root/'runtime/native/gc/native-gc.c','-o',exe])
   for count in [0,1,4,5,64]:
    run([exe,count,0,0])
    if count:
     for index in sorted({0,count-1}):
      for fault in [1,2,3,4]:run([exe,count,index,fault],0 if fault==4 else -signal.SIGABRT)
 for path in [Path(__file__),base/'storage-boundary-test.c',*(root/'runtime/native/gc').glob('*.[ch]')]:
  r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
print('PASS original/split storage boundaries and invalid tails')

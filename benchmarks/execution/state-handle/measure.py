import argparse,hashlib,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--source',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
root=Path(__file__).resolve().parents[3];out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command'];cc=recipe[:recipe.index('-std=c11')]
r=dict(passed=False,commands=[],inputs={},scope='Same prototype binary, per-frame state capture; empty-heap hook CPU, not HTTP capacity')
def run(cmd):
 cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=90);r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
try:
 exe=out/'cost';flags=cc+['-O2','-std=c11','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PACED','-I',root/'runtime/native/gc','-I',root/'runtime/native/memory']
 run(flags+[Path(__file__).with_name('cost.c'),a.source.resolve(),root/'runtime/native/gc/native-gc.c','-o',exe])
 for mode in ['0','1','1','0']:run([exe,mode])
 run(flags+['-S',a.source.resolve(),'-o',out/'prototype.s'])
 for path in [Path(__file__),Path(__file__).with_name('cost.c'),a.source.resolve(),exe,out/'prototype.s',*(root/'runtime/native/gc').glob('*.[ch]')]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

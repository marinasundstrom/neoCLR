"""Qualify explicit native mutators with the live marker; no compiled shared-heap admission."""
import argparse,hashlib,json,signal,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);a=p.parse_args()
root=Path(__file__).resolve().parents[3];base=Path(__file__).resolve().parent
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False);r=dict(passed=False,commands=[],inputs={})
def run(cmd):
 cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=120);r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
try:
 source=out/'native-gc.c';live=base.parent/'concurrent-live';gate=base.parent/'concurrent-mutation-gate'
 run(['python3',live/'generate.py','--output',source])
 recipe=json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command'];cc=recipe[:recipe.index('-std=c11')]
 for sanitized in [False,True]:
  exe=out/f'test-{sanitized}'
  cmd=cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PROFILE','-DNEOCLR_GATE_WORKERS','-DNEOCLR_SHADOW_TEST','-DGATE_WORDS=64','-I',gate,'-I',live,'-I',root/'runtime/native/gc',gate/'gate.c',root/'runtime/native/gc/root-probe.c',root/'runtime/native/gc/participant-roots.c',source,base/'test.c','-o',exe]
  if sanitized:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
  run(cmd);run([exe])
 # Zero logged values without changing capacity/overflow behavior. Each variant
 # must reach the final object-liveness assertion, not merely fail setup.
 ordinary=[str(x) for x in cmd if str(x) not in ['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']]
 original='memcpy(&g->words[g->count++], (unsigned char *)payload + at, 8);'
 prefix,body=(gate/'gate.c').read_text().split('static int write_locked(',1)
 assert body.count(original)==2
 r['negative']=[]
 for index,name in enumerate(['deletion','insertion']):
  pieces=body.split(original);pieces[index+1]='g->words[g->count++] = 0;'+pieces[index+1]
  broken=pieces[0]+('' if index==0 else original)+pieces[1]+('' if index==1 else original)+pieces[2]
  negative_gate=out/f'{name}-gate.c';negative_gate.write_text(prefix+'static int write_locked('+broken)
  exe=out/f'negative-{name}'
  command=[str(negative_gate) if x==str(gate/'gate.c') else x for x in ordinary]
  command[command.index('-o')+1]=str(exe);run(command)
  x=subprocess.run([str(exe)],capture_output=True,text=True,timeout=60)
  expected=next(i for i,line in enumerate((base/'test.c').read_text().splitlines(),1) if 'allocation_info_v1(&context,objects[i][j]' in line)
  r['negative'].append(dict(kind=name,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr,expectedLivenessLine=expected))
  assert x.returncode==-signal.SIGABRT and x.stderr.strip()==f'line {expected}'
 for path in [source,live/'generate.py',Path(__file__),base/'test.c',*live.glob('*.[h]'),*gate.glob('*.[ch]'),* (root/'runtime/native/gc').glob('*.[ch]')]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

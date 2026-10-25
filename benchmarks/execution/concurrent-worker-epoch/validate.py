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
  persistent_exe=out/f'persistent-{sanitized}'
  persistent_cmd=[persistent_exe if x==exe else x for x in cmd]+['-DNEOCLR_PERSISTENT_MUTATORS']
  run(persistent_cmd);run([persistent_exe])
  allocation_exe=out/f'allocation-{sanitized}'
  allocation_cmd=[allocation_exe if x==exe else base/'allocation.c' if x==base/'test.c' else x for x in cmd]+['-DNEOCLR_GC_ALLOCATION_DISPATCH']
  run(allocation_cmd);run([allocation_exe])
  array_exe=out/f'arrays-{sanitized}'
  array_cmd=[array_exe if x==exe else base/'arrays.c' if x==base/'test.c' else x for x in cmd]
  run(array_cmd);run([array_exe])
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
  persistent_exe=out/f'negative-persistent-{name}'
  persistent_cmd=command+['-DNEOCLR_PERSISTENT_MUTATORS']
  persistent_cmd[persistent_cmd.index('-o')+1]=str(persistent_exe);run(persistent_cmd)
  x=subprocess.run([str(persistent_exe)],capture_output=True,text=True,timeout=60)
  r['negative'].append(dict(kind='persistent-'+name,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr,expectedLivenessLine=expected))
  assert x.returncode==-signal.SIGABRT and x.stderr.strip()==f'line {expected}'
  array_exe=out/f'negative-arrays-{name}'
  array_cmd=[str(base/'arrays.c') if x==str(base/'test.c') else x for x in command]
  array_cmd[array_cmd.index('-o')+1]=str(array_exe);run(array_cmd)
  for shape in range(3):
   x=subprocess.run([str(array_exe),str(shape)],capture_output=True,text=True,timeout=60)
   child='old_child' if name=='deletion' else 'new_child'
   expected=next(i for i,line in enumerate((base/'arrays.c').read_text().splitlines(),1) if f'allocation_info_v1(&context,{child},' in line)
   r['negative'].append(dict(kind='arrays-'+name,shape=shape,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr,expectedLivenessLine=expected))
   assert x.returncode==-signal.SIGABRT and x.stderr.strip()==f'line {expected}'
 # Bypassing dispatch must fail successful allocation from the empty worker
 # arena, demonstrating that the test uses the owner service rather than a copy.
 broken_source=out/'no-allocation-dispatch.c'
 text=source.read_text();needle='if (neoclr_gc_allocation_dispatch_hook)'
 assert text.count(needle)==1
 broken_source.write_text(text.replace(needle,'if (0 && neoclr_gc_allocation_dispatch_hook)'))
 exe=out/'negative-allocation'
 command=[str(broken_source) if str(x)==str(source) else str(x) for x in allocation_cmd if str(x) not in ['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']]
 command[command.index('-o')+1]=str(exe);run(command)
 x=subprocess.run([str(exe)],capture_output=True,text=True,timeout=60)
 expected=next(i for i,line in enumerate((base/'allocation.c').read_text().splitlines(),1) if 'CHECK(!neoclr_gc_allocate_v1(&s->context.text' in line)
 r['negative'].append(dict(kind='allocation-dispatch',exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr,expectedLine=expected))
 assert x.returncode==-signal.SIGABRT and 1<=len(x.stderr.splitlines())<=4 and set(x.stderr.splitlines())=={f'line {expected}'}
 # Pacing counters must belong to the requesting worker, not the owner service.
 broken_counts=out/'no-requester-count.c'
 needle='        gc_state.statistics.allocations++;'
 assert source.read_text().count(needle)==1
 broken_counts.write_text(source.read_text().replace(needle,'        /* deliberately missing requester count */'))
 exe=out/'negative-allocation-count'
 command=[str(broken_counts) if str(x)==str(source) else str(x) for x in allocation_cmd if str(x) not in ['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']]
 command[command.index('-o')+1]=str(exe);run(command)
 x=subprocess.run([str(exe)],capture_output=True,text=True,timeout=60)
 expected=next(i for i,line in enumerate((base/'allocation.c').read_text().splitlines(),1) if 'CHECK(neoclr_gc_statistics_v1().allocations==2)' in line)
 r['negative'].append(dict(kind='requester-accounting',exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr,expectedLine=expected))
 assert x.returncode==-signal.SIGABRT and 1<=len(x.stderr.splitlines())<=4 and set(x.stderr.splitlines())=={f'line {expected}'}
 for path in [source,live/'generate.py',Path(__file__),base/'test.c',base/'arrays.c',base/'allocation.c',*live.glob('*.[h]'),*gate.glob('*.[ch]'),* (root/'runtime/native/gc').glob('*.[ch]')]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

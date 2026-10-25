"""Relink qualified HTTP native hosts with owner-local transfer timestamps.
Diagnostic only: clock costs remain included; never use this build for timings.
"""
import argparse,hashlib,json,re,subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--scheduler',action='store_true',help='Split acquisition into poll, idle GC and root delivery; implies --phases')
p.add_argument('--phases',action='store_true',help='Also time root/resume and scheduler intervals')
p.add_argument('--build',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();root=Path(__file__).resolve().parents[3];base=Path(__file__).resolve().parent
a.phases = a.phases or a.scheduler
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
prior=json.loads((a.build/'build.json').read_text());assert prior['passed']
r=dict(passed=False,fiberSwitchProfile=True,fiberPhaseProfile=a.phases,schedulerPhaseProfile=a.scheduler,diagnosticOnly=__doc__,commands=[],recipes={},inputs={})
def run(cmd,expected=0):
 cmd=list(map(str,cmd));x=subprocess.run(cmd,capture_output=True,text=True,timeout=120)
 r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));assert x.returncode==expected,x.stderr
 return x
try:
 original=root/'runtime/native/async/native-runtime.c';s=original.read_text()
 s=re.sub(r'#include "(.*?)"',lambda m:'#include "'+str(original.parent/m[1])+'"' if (original.parent/m[1]).exists() else m[0],s)
 s=s.replace('#include <string.h>','#include <string.h>\n#include "'+str(base/'fiber-clock.h')+'"')
 for call in ['if (swapcontext(&f->machine, f->caller)) abort();','if (swapcontext(&caller, &f->machine)) abort();']:
  assert s.count(call)==1;s=s.replace(call,'switch_begin();\n    '+call+'\n    switch_arrive();')
 assert s.count('static void body(void) {')==1
 s=s.replace('static void body(void) {','static void body(void) {\n    switch_arrive();')
 assert s.count('    memset(&state, 0, sizeof(state));')==1
 s=s.replace('    memset(&state, 0, sizeof(state));','    switch_dump(statistics.resumes);\n    memset(&state, 0, sizeof(state));')
 if a.phases:
  s=s.replace('#include "'+str(base/'fiber-clock.h')+'"','#include "'+str(base/'fiber-clock.h')+'"\n#include "'+str(base/'fiber-phases.h')+'"')
  start=s.index('static int resume(fiber *f) {');end=s.index('int32_t neoclr_runtime_async_create_v1',start)
  body=s[start:end]
  body=body.replace('    if (f->roots.expired)', '    phase_stamp prepare = phase_begin();\n    if (f->roots.expired)',1)
  body=body.replace('    switch_begin();','    phase_end(0, prepare);\n    switch_begin();',1)
  body=body.replace('    state.current = parent;','    phase_stamp finish = phase_begin();\n    state.current = parent;',1)
  split=body.index('    phase_stamp finish')
  body=body[:split]+body[split:].replace('return status;', 'phase_end(1, finish); return status;')
  s=s[:start]+body+s[end:]
  s=s.replace('static void yield(void) {','static void yield(void) {\n    phase_stamp stamp = phase_begin();',1)
  s=s.replace('    f->parked = 1;','    phase_end(2, stamp);\n    f->parked = 1;',1)
  call='int progress = neoclr_scheduler_acquire_v1(c, &delivery, NULL);'
  assert s.count(call)==1
  s=s.replace(call,'phase_stamp acquire = phase_begin();\n        '+call+'\n        phase_end(3, acquire);')
  call='if (!ready) neoclr_scheduler_wait_v1(c);';assert s.count(call)==1
  s=s.replace(call,'if (!ready) { phase_stamp wait = phase_begin(); neoclr_scheduler_wait_v1(c); phase_end(4, wait); }')
  s=s.replace('    switch_dump(statistics.resumes);','    phase_dump();\n    switch_dump(statistics.resumes);',1)
 scheduler_original=root/'docs/experiments/aot-console/socket-listener.c'
 scheduler_source=out/'socket-listener.c'
 if a.scheduler:
  s=s.replace('    switch_dump(statistics.resumes);', '#ifdef NEOCLR_DIAGNOSTIC_SCHEDULER\n    extern void neoclr_diagnostic_scheduler_dump(void);\n    neoclr_diagnostic_scheduler_dump();\n#endif\n    switch_dump(statistics.resumes);',1)
  socket=scheduler_original.read_text()
  socket=re.sub(r'#include "(.*?)"',lambda m:'#include "'+str(scheduler_original.parent/m[1])+'"' if (scheduler_original.parent/m[1]).exists() else m[0],socket)
  start=socket.index('int32_t neoclr_scheduler_acquire_v1(');end=socket.index('int32_t neoclr_scheduler_complete_v1(',start)
  body=socket[start:end]
  needle='    int32_t progress = neoclr_scheduler_step_v1(context, &source, boundary);';assert body.count(needle)==1
  body=body.replace(needle,'    scheduler_stamp poll = scheduler_begin();\n'+needle+'\n    scheduler_end(0, poll);')
  needle='        if ((progress == NEOCLR_SCHEDULER_IDLE || progress == NEOCLR_SCHEDULER_PENDING) &&\n            neoclr_gc_idle_v1(context, boundary)) return -3;';assert body.count(needle)==1
  body=body.replace(needle,'        if (progress == NEOCLR_SCHEDULER_IDLE || progress == NEOCLR_SCHEDULER_PENDING) {\n            scheduler_stamp idle = scheduler_begin();\n            int status = neoclr_gc_idle_v1(context, boundary);\n            scheduler_end(1, idle);\n            if (status) return -3;\n        }')
  body=body.replace('    neoclr_socket_scope *scope = find_scope(context);','    scheduler_stamp roots = scheduler_begin();\n    neoclr_socket_scope *scope = find_scope(context);',1)
  body=body.replace('    return NEOCLR_SCHEDULER_READY;','    scheduler_end(2, roots);\n    return NEOCLR_SCHEDULER_READY;',1)
  socket=socket[:start]+'#include "'+str(base/'scheduler-clock.h')+'"\n'+body+socket[end:]
  scheduler_source.write_text(socket)
 source=out/'native-runtime.c';source.write_text(s)
 validation=json.loads((root/'docs/experiments/runtime-async/owned-native-body-validation.json').read_text())
 test=next(c['command'].copy() for c in validation['commands'] if any(v.endswith('native-owned-body-test.c') for v in c['command']))
 test=[str(source) if v.endswith('/native-runtime.c') else v for v in test]
 test[test.index('-o')+1]=str(out/'positive');run(test)
 positive=run([out/'positive']);assert 'FIBER_SWITCH ' in positive.stderr
 negative=out/'negative-runtime.c'
 needle='static void body(void) {\n    switch_arrive();';assert s.count(needle)==1
 negative.write_text(s.replace(needle,'static void body(void) {'))
 test=[str(negative) if v==str(source) else v for v in test];test[test.index('-o')+1]=str(out/'negative')
 run(test);run([out/'negative'],expected=-6)
 for mode in ['sync','live']:
  recipe=next(c['command'].copy() for c in prior['commands'] if '-o' in c['command'] and c['command'][c['command'].index('-o')+1]==str(a.build.resolve()/mode))
  assert str(original) in recipe
  recipe=[str(source) if v==str(original) else v for v in recipe]
  if a.scheduler:
   assert str(scheduler_original) in recipe
   recipe=[str(scheduler_source) if v==str(scheduler_original) else v for v in recipe]+['-DNEOCLR_DIAGNOSTIC_SCHEDULER']
  recipe[recipe.index('-o')+1]=str(out/mode);run(recipe)
  r['recipes'][mode+'-aot']=dict(execution='aot',collector=mode,command=[str(out/mode)],env={})
  shared=[v for v in recipe if not v.endswith(('/main.c','/'+mode+'.o'))]+['-dynamiclib']
  host=out/(mode+'.dylib');shared[shared.index('-o')+1]=str(host);run(shared)
  old=prior['recipes'][mode+'-jit'];selection=json.loads(Path(old['env']['NEOCLR_LIVE_SELECTION_COMMAND']).read_text())
  previous_host=str(a.build.resolve()/(mode+'.dylib'));assert previous_host in selection
  selection=[str(host) if v==previous_host else v for v in selection]
  path=out/(mode+'-selection.json');path.write_text(json.dumps(selection)+'\n')
  r['recipes'][mode+'-jit']=dict(old,env=dict(old['env'],NEOCLR_LIVE_SELECTION_COMMAND=str(path)))
 for c in r['commands']:
  for value in c['command']:
   path=Path(value)
   if path.is_file():r['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
 for path in [Path(__file__),base/'fiber-clock.h',base/'fiber-phases.h',base/'scheduler-clock.h',scheduler_original,original,source,a.build/'build.json',*out.glob('*.dylib'),out/'sync',out/'live',*out.glob('*selection.json')]:
  r['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'build.json').write_text(json.dumps(r,indent=2)+'\n')

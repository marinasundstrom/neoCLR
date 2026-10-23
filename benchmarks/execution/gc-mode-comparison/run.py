#!/usr/bin/env python3
"""Compare admitted sample behavior across interpreter/JIT/AOT and native GC policies."""
import argparse
import gzip
import hashlib
import json
import os
from pathlib import Path
import resource
import subprocess
import time

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args(); out = a.output.resolve(); out.mkdir(parents=True, exist_ok=False)
root = Path(__file__).resolve().parents[3]
r = dict(passed=False, commands=[], inputs={}, cases=[], scope='Small sample admission and policy experiment; not HTTP capacity or equal GC representations.')

def record(path):
    path = Path(path).resolve(); r['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()

def run(command, label, timeout=180):
    command = list(map(str, command)); before = resource.getrusage(resource.RUSAGE_CHILDREN); start=time.perf_counter()
    result = subprocess.run(command, capture_output=True, text=True, timeout=timeout, cwd=root)
    wall = time.perf_counter()-start; after=resource.getrusage(resource.RUSAGE_CHILDREN)
    row=dict(label=label, command=command, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr,
             wallSeconds=wall, processCpuSeconds=after.ru_utime+after.ru_stime-before.ru_utime-before.ru_stime)
    r['commands'].append(row)
    (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
    return row

try:
    runtime=root/'target/release/neoclr'; compiler=root/'target/release/neoclr-codegen'
    build=run(['cargo','build','--release','--bin','neoclr'],'runtime-build'); assert build['exitCode']==0,build
    recipe=json.loads((root/'target/primitive-comparison-build/build.json').read_text())
    native=next(c['command'] for c in recipe['commands'] if c.get('phase')=='native-compilation')
    link=next(c['command'] for c in recipe['commands'] if c.get('phase')=='host-compile-link')
    host=root/'docs/experiments/jit-managed/host.c'
    base=[str(host) if x.endswith('/benchmarks/native-web/http-host.c') else x for x in link if x != native[4] and not x.startswith('-DNEOCLR_HTTP_WORKERS=')]
    gc=root/'runtime/native/gc/native-gc.c'; text=gc.read_text()
    assert text.count('gc_state.statistics.allocations++;')==1 and text.count('gc_state.statistics.collections++;')==1
    text='#include <stdatomic.h>\n#include <stdio.h>\nstatic _Atomic unsigned long long mode_allocs, mode_collections;\n'+text
    text=text.replace('gc_state.statistics.allocations++;','gc_state.statistics.allocations++; atomic_fetch_add(&mode_allocs,1);')
    text=text.replace('gc_state.statistics.collections++;','gc_state.statistics.collections++; atomic_fetch_add(&mode_collections,1);')
    text+='\n__attribute__((destructor)) static void mode_counts(void) { fprintf(stderr,"MODEGC %llu %llu\\n",atomic_load(&mode_allocs),atomic_load(&mode_collections)); }\n'
    diagnostic=out/'diagnostic-gc.c';diagnostic.write_text(text)
    wrapper=out/'main.c';wrapper.write_text('''#include <stdint.h>
#include <stdio.h>
#include <time.h>
extern int32_t neoclr_entry_v4(int32_t, int32_t *, void *);
extern int32_t neoclr_jit_run_v1(const void *, int32_t, int32_t, const char **);
int main(int argc, const char **argv) {
 struct timespec a,b; clock_gettime(CLOCK_MONOTONIC,&a);
 int status=neoclr_jit_run_v1((const void *)neoclr_entry_v4,0,argc,argv);
 clock_gettime(CLOCK_MONOTONIC,&b);
 fprintf(stderr,"MODETIME %.9f\\n",(b.tv_sec-a.tv_sec)+(b.tv_nsec-a.tv_nsec)/1e9);
 return status;
}
''')
    hosts={}
    def link_command(policy, counted, output):
        cmd=base.copy();cmd[cmd.index('-o')+1]=str(output)
        cmd+=['-I',str(root/'runtime/native/gc')]
        if policy=='paced':cmd+=['-DNEOCLR_GC_PACED']
        if counted:cmd=[str(diagnostic) if x.endswith('/docs/experiments/aot-console/native-gc.c') else x for x in cmd]
        return cmd
    for policy in ['forced','paced']:
        for counted in [False,True]:
            lib=out/f'{policy}-{counted}.dylib';cmd=link_command(policy,counted,lib)+['-dynamiclib']
            row=run(cmd,'host-build'); assert row['exitCode']==0,row
            hosts[(policy,counted)]=lib;record(lib)
    corpus=json.loads(gzip.decompress((root/'benchmarks/http-server/primitive-comparison/transient-handoff/corpus.json.gz').read_bytes()))
    for sample in corpus['cases']:
        name=sample['sample'];cmd=next(c['command'].copy() for c in corpus['commands'] if '--closed-world' in c['command'] and f'/{name}/bin/' in c['command'][2])
        expected=sample['stdout'][:len(sample['stdout'])//2]
        obj=out/(name+'.o');cmd[0]=str(compiler);cmd[4]=str(obj)
        row=run(cmd,'aot-compile'); assert row['exitCode']==0,row['stderr'];row['stdoutSha256']=hashlib.sha256(row['stdout'].encode()).hexdigest();row['stdout']='(metadata omitted; hash retained)'
        record(obj);record(cmd[2]);case=dict(sample=name,expected=expected,runs=[]);r['cases'].append(case)
        context=cmd[5:cmd.index('--compile-system')]
        for counted in [False,True]:
            for repetition in range(1 if counted else 3):
                row=run([runtime,'run',cmd[2],'--execution','interpreter',*context,*(['--gc-stats'] if counted else [])],'interpreter')
                row.update(mode='interpreter',diagnostic=counted,repetition=repetition,outputMatches=row['stdout']==expected)
                case['runs'].append(row)
        for policy in ['forced','paced']:
            for counted in [False,True]:
                exe=out/f'{name}-{policy}-{counted}';row=run(link_command(policy,counted,exe)+[str(wrapper),str(obj)],'aot-link');assert row['exitCode']==0,row
                record(exe)
                for repetition in range(1 if counted else 3):
                    jit=[str(compiler),'--jit-managed','--closed-world',cmd[2],cmd[3],str(hosts[(policy,counted)]),*cmd[5:]]
                    for mode,command in [('jit',jit),('aot',[exe])]:
                        row=run(command,mode);row.update(mode=mode,policy=policy,diagnostic=counted,repetition=repetition)
                        if mode=='jit' and row['exitCode']==0:
                            line=next(x for x in row['stderr'].splitlines() if x.startswith('JIT_RESULT '))
                            row['jit']=json.loads(line[len('JIT_RESULT '):]);row['outputMatches']=row['stdout']==expected and row['jit']['guestExitCode']==0
                        else:row['outputMatches']=row['stdout']==expected
                        case['runs'].append(row)
    for path in [runtime,compiler,host,wrapper,diagnostic,Path(__file__),*(root/'runtime/native').rglob('*.[ch]')]:record(path)
    r['passed']=all(x['exitCode']==0 and x['outputMatches'] for c in r['cases'] for x in c['runs'])
finally:
    (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps(dict(passed=r['passed'],cases=len(r['cases']))))

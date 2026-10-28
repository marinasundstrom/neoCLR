#!/usr/bin/env python3
"""Qualify bounded completion wakeups and real owner resumption with both collectors."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
out = a.output.resolve(); out.mkdir(parents=True, exist_ok=False)
here = Path(__file__).resolve().parent; root = here.parents[2]
native = root / 'runtime/native'; legacy = root / 'docs/experiments/aot-console'; live = native / 'gc/concurrent'
report = dict(passed=False, platform=platform.platform(), commands=[], samples=[])


def run(command):
    command = list(map(str, command))
    r = subprocess.run(command, capture_output=True, text=True, timeout=60, cwd=out)
    report['commands'].append(dict(command=command, exitCode=r.returncode, stdout=r.stdout, stderr=r.stderr))
    r.check_returncode(); return r.stdout


try:
    cc = [run(['xcrun','--find','clang']).strip(), '-isysroot', run(['xcrun','--show-sdk-path']).strip()]
    flags = ['-std=c11','-O2','-Wall','-Wextra','-Werror','-Wno-deprecated-declarations',
             '-pthread','-Wl,-dead_strip','-DNEOCLR_WORKER_DELIVERY',
             '-I',native/'async','-I',native/'gc','-I',native/'workers','-I',legacy]
    run(['python3',live/'generate.py','--output',out/'live-gc.c'])
    policy = json.loads((live/'policy.json').read_text())
    for mode in ('sync','concurrent'):
        defs = ['-DNEOCLR_NATIVE_GC'] if mode=='sync' else ['-D'+d for d in policy['defines']]+['-DTRACE_LIVE','-I',live]
        gc = [native/'gc/native-gc.c'] if mode=='sync' else [out/'live-gc.c',live/'gate.c']
        common = [native/'workers/native-workers.c',native/'gc/root-probe.c',*gc]
        for variant,extra in [('optimized',[]),('ubsan',['-fsanitize=undefined','-fno-sanitize-recover=all'])]:
            test = out/f'{mode}-{variant}-test'
            run([*cc,*flags,*defs,*extra,here/'test.c',*common,'-o',test]); run([test])
            bench = out/f'{mode}-{variant}'
            run([*cc,*flags,*defs,*extra,here/'benchmark.c',native/'async/native-runtime.c',
                 legacy/'socket-listener.c',legacy/'native-stack.c',legacy/'text-arena.c',*common,'-o',bench])
            for wake in (0,1): run([bench,wake,8])
            chain = out/f'{mode}-{variant}-chain'
            run([*cc,*flags,*defs,*extra,here/'chain.c',native/'async/native-runtime.c',
                 legacy/'socket-listener.c',legacy/'native-stack.c',legacy/'text-arena.c',*common,'-o',chain])
            run([chain])
        trace = out/f'{mode}-traced'
        run([*cc,*flags,*defs,'-DDELIVERY_TRACE','-DNEOCLR_ASYNC_TRACE',here/'benchmark.c',
             native/'async/native-runtime.c',legacy/'socket-listener.c',legacy/'native-stack.c',
             legacy/'text-arena.c',*common,'-o',trace])
        run([trace,1,16])
        raw=out/f'{mode}-trace.json'; (out/'trace.json').rename(raw)
        events=json.loads(raw.read_text())
        assert not events['dropped']
        assert sum(e[4]==5 for e in events['events'])==16*9
        run(['python3',here.parent/'execution-trace/async-export.py',raw,out/f'{mode}.trace.json'])
    # Five interleaved pairs per collector; identical bounded work and forced root checks.
    for pair in range(5):
        for mode in (('sync','concurrent') if pair%2==0 else ('concurrent','sync')):
            for wake in ((0,1) if pair%2==0 else (1,0)):
                sample=json.loads(run([out/f'{mode}-optimized',wake,256]))
                sample.update(mode=mode,pair=pair);report['samples'].append(sample)
    paths = [*here.glob('*.c'),Path(__file__),out/'live-gc.c',legacy/'socket-listener.c',legacy/'socket-listener.h']
    paths += [f for f in native.rglob('*') if f.suffix in ('.c','.h','.json','.py')]
    report['inputs'] = {str(f):hashlib.sha256(f.read_bytes()).hexdigest() for f in paths}
    report['passed'] = True
finally:
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print('PASS worker wake, shared capture roots and owner async/TCP delivery under both collectors')

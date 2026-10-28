#!/usr/bin/env python3
"""Private generated callback qualification; does not admit HTTP owner heaps."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--selection', type=Path, required=True, help='Existing closed-world bootstrap codegen command JSON')
p.add_argument('--runner', type=Path, required=True, help='Matching neoclr-codegen library test executable')
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
out = a.output.resolve(); out.mkdir(parents=True, exist_ok=False)
here = Path(__file__).resolve().parent; root = here.parents[2]
native = root/'runtime/native'; live = native/'gc/concurrent'; legacy = root/'docs/experiments/aot-console'
report = dict(passed=False, commands=[], inputs={})

def record(path):
    path = Path(path).resolve()
    report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()

def run(command, env=None):
    command = list(map(str, command))
    r = subprocess.run(command, capture_output=True, text=True, timeout=60, env=env)
    text = r.stdout
    if len(text)>32000:
        log = out/f'command-{len(report["commands"])}.log'; log.write_text(text); record(log)
        text = f'Output retained at {log}'
    report['commands'].append(dict(command=command, exitCode=r.returncode, stdout=text, stderr=r.stderr,
                                   environment={k:v for k,v in (env or {}).items() if k.startswith('NEOCLR_LIVE_SELECTION')}))
    r.check_returncode(); return r.stdout

try:
    selection = json.loads(a.selection.read_text())
    assert '--closed-world' in selection and '--native-host-bootstrap' in selection
    record(a.selection); record(a.runner)
    for item in selection[1:]:
        if Path(item).is_file(): record(item)
    cc = [run(['xcrun','--find','clang']).strip(),'-isysroot',run(['xcrun','--show-sdk-path']).strip()]
    run(['python3',live/'generate.py','--output',out/'live-gc.c'])
    run(['python3',live/'native-providers.py','--output',out/'text-arena.c'])
    policy = json.loads((live/'policy.json').read_text())
    for mode in ('sync','concurrent'):
        command = selection.copy(); command[4] = str(out/f'{mode}.o')
        recipe = out/f'{mode}-selection.json'; recipe.write_text(json.dumps(command,indent=2)+'\n'); record(recipe)
        env = dict(os.environ, NEOCLR_LIVE_SELECTION_COMMAND=str(recipe), NEOCLR_LIVE_SELECTION_MODE=mode)
        run([a.runner.resolve(),'mutation_store::tests::full_http_selection_compiles_with_live_gc','--exact','--ignored','--nocapture'],env)
        record(out/f'{mode}.o')
        defs = (['NEOCLR_NATIVE_GC','NEOCLR_GC_PROFILE','NEOCLR_GC_PACED',
                 'NEOCLR_GC_ALLOCATION_BUDGET=1024','NEOCLR_GC_PUBLICATION_BUDGET=16384'] if mode=='sync'
                else policy['defines']+['DELIVERY_CONCURRENT'])
        gc = [native/'gc/native-gc.c'] if mode=='sync' else [out/'live-gc.c',live/'gate.c']
        provider = native/'memory/text-arena.c' if mode=='sync' else out/'text-arena.c'
        common = [*cc,'-std=c11','-O2','-Wall','-Wextra','-Werror','-Wno-deprecated-declarations',
                  '-pthread','-Wl,-dead_strip','-DNEOCLR_WORKER_DELIVERY','-DNEOCLR_EXCLUSIVE_HEAP_ADOPTION',
                  '-DNEOCLR_WORKER_OWNED_CALLBACKS',*['-D'+d for d in defs],
                  '-I',native/'gc','-I',live,'-I',native/'memory','-I',native/'async','-I',native/'workers','-I',legacy,
                  here/'generated-chain.c',out/f'{mode}.o',native/'async/native-runtime.c',legacy/'socket-listener.c',
                  legacy/'native-stack.c',provider,native/'workers/native-workers.c',native/'gc/root-probe.c',*gc]
        for variant, flags in [('optimized',[]),('ubsan',['-fsanitize=undefined','-fno-sanitize-recover=all'])]:
            binary = out/f'{mode}-{variant}'
            run([*common,*flags,'-o',binary]); run([binary])
            shared = [here.parent/'native-worker-pool/shared-host.c' if x==here/'generated-chain.c' else x for x in common]
            binary = out/f'{mode}-{variant}-shared'
            run([*shared,*flags,'-o',binary]); run([binary])
        if mode=='concurrent':
            rejected = [out/'sync.o' if x==out/'concurrent.o' else native/'memory/text-arena.c' if x==provider else x for x in common]
            binary = out/'rejected-image'
            run([*rejected,'-DNEOCLR_EXPECT_IMAGE_REJECTION','-o',binary]); run([binary])
    for path in [Path(__file__),here.parent/'native-worker-pool/shared-host.c',here/'generated-chain.c',here/'Generated.rvn',out/'live-gc.c',out/'text-arena.c',
                 *list((root/'runtime/codegen/src').glob('*.rs')),
                 *[f for f in native.rglob('*') if f.suffix in ('.c','.h','.inc','.json','.py')],
                 legacy/'socket-listener.c',legacy/'native-stack.c']:
        record(path)
    report['passed'] = True
finally:
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print('PASS owned generated callback roots, active marker epochs and image admission')

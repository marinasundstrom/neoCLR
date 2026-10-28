#!/usr/bin/env python3
"""Build matched queue-control/candidate HTTP JIT hosts from one source revision."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import runpy
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--bundle', type=Path, required=True)
p.add_argument('--runner', type=Path, required=True)
p.add_argument('--runtime', type=Path, required=True, help='Metadata reader matching the compiler output')
p.add_argument('--host-recipes', type=Path, required=True)
p.add_argument('--selection', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--trace', action='store_true', help='Separate diagnostic builds, never headline timings')
a = p.parse_args()
here = Path(__file__).resolve().parent
root = here.parents[2]
native = root/'runtime/native'
live = native/'gc/concurrent'
out = a.output.resolve(); out.mkdir(parents=True, exist_ok=False)
bundle = a.bundle.resolve()
report = dict(passed=False, recipes={}, inputs={}, commands=[],
              workload='shared-capture-task-run-http',
              scope='Matched lazy JIT HTTP queues; owner affinity unchanged. '
                    'Both collectors use 1 MiB heaps and 1024/16384 pacing budgets.')
if a.trace: report['diagnosticOnly'] = 'Bounded owner async/GC tracing enabled; includes instrumentation overhead'

def record(path):
    path = Path(path).resolve()
    report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()

def run(command):
    command = list(map(str, command))
    r = subprocess.run(command, capture_output=True, text=True, timeout=120)
    report['commands'].append(dict(command=command, exitCode=r.returncode,
                                  stdout=r.stdout, stderr=r.stderr))
    r.check_returncode(); return r.stdout

try:
    for path in (a.host_recipes, a.selection, a.runner, a.runtime, Path(__file__)):
        record(path)
    run(['python3', live/'generate.py', '--output', out/'live-gc.c'])
    run(['python3', live/'native-providers.py', '--output', out/'text-arena.c'])
    guest = here/'bin/neoclr/SharedSchedulingHttp.dll'
    assert guest.is_file(), 'Compile Http.rvnproj with the supplied bundle first'
    roots = runpy.run_path(str(bundle/'tools/run-native-sample.py'))['reflection_roots'](
        a.runtime.resolve(), guest, ['Person'], run)
    seed = json.loads(a.selection.read_text())
    recipes = [x['command'] for x in json.loads(a.host_recipes.read_text())
               if '-dynamiclib' in x.get('command', [])]
    assert len(recipes) == 2
    for mode, command in zip(('sync', 'live'), recipes):
        assert ('-DWORKLOAD_LIVE' in command) == (mode == 'live')
        for policy in ('baseline', 'shared'):
            name = f'{policy}-{mode}'
            host = out/f'{name}.dylib'
            c = []
            for item in command:
                path = Path(item)
                if path.name == 'owned-host.c': item = str(here/'jit-host.c')
                elif mode == 'live' and path.name == 'native-gc.c': item = str(out/'live-gc.c')
                elif mode == 'live' and path.name == 'text-arena.c': item = str(out/'text-arena.c')
                elif item.startswith('-DNEOCLR_GC_ALLOCATION_BUDGET='): item = '-DNEOCLR_GC_ALLOCATION_BUDGET=1024'
                elif item.startswith('-DNEOCLR_GC_PUBLICATION_BUDGET='): item = '-DNEOCLR_GC_PUBLICATION_BUDGET=16384'
                c.append(item)
            c[c.index('-o')+1] = str(host)
            if policy == 'shared': c.append('-DNEOCLR_SHARED_READY_QUEUE')
            if a.trace: c += ['-DNEOCLR_ASYNC_TRACE', '-DNEOCLR_EXECUTION_TRACE',
                             '-I', str(native/'trace'), str(native/'trace/session.c')]
            run(c)
            for item in c:
                if Path(item).is_file(): record(item)
            selection = seed.copy()
            selection[3] = str(guest); selection[5] = str(host)
            i = selection.index('--reflection-roots'); del selection[i:i+2]
            selection += list(map(str, roots))
            for i, item in enumerate(selection):
                if i and selection[i-1] in ('--system', '--module', '--object-root'):
                    selection[i] = str(bundle/'lib'/Path(item).name)
            # This consumer uses only the four base libraries.
            for i in reversed(range(1, len(selection))):
                if selection[i].endswith('/System.Data.PostgreSql.dll'):
                    del selection[i-1:i+1]
            path = out/f'{name}-selection.json'
            path.write_text(json.dumps(selection, indent=2)+'\n'); record(path)
            for item in selection[1:]:
                if Path(item).is_file(): record(item)
            report['recipes'][name] = dict(execution='jit', collector='live' if mode=='live' else 'mark-sweep',
                command=[str(a.runner.resolve()), 'mutation_store::tests::full_http_selection_compiles_with_live_gc',
                         '--exact', '--ignored', '--nocapture'],
                env=dict(NEOCLR_LIVE_SELECTION_COMMAND=str(path), NEOCLR_LIVE_SELECTION_MODE=mode,
                         NEOCLR_JIT_LAZY='1', NEOCLR_LIVE_STACK_POLICY='qualified'))
            if a.trace: report['recipes'][name]['env']['NEOCLR_HTTP_TRACE'] = '1'
    for directory in (native, root/'runtime/codegen/src', root/'docs/experiments/aot-console'):
        for path in directory.rglob('*'):
            if path.suffix in ('.c', '.h', '.inc', '.rs', '.py', '.json'): record(path)
    for path in (here/'Http.rvn', here/'Http.rvnproj', here/'jit-host.c'): record(path)
    report['passed'] = True
finally:
    (out/'build.json').write_text(json.dumps(report, indent=2)+'\n')
print('PASS matched HTTP hosts prepared; execution qualification remains required')

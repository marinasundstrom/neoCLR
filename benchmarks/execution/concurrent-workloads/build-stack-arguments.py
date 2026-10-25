"""Recompile retained ordinary live selections with test-only stack guard removal."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--runner', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
root = Path(__file__).resolve().parents[3]
source = a.build.resolve()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
original = json.loads((source / 'results.json').read_text())
assert original['passed']
r = dict(passed=False, scope=__doc__, iterations=original['iterations'],
         optimizationLevel=original['optimizationLevel'], cases=[], inputs={})
try:
    for old in original['cases']:
        sample = old['sample']
        dest = out / sample
        dest.mkdir()
        shutil.copyfile(source / sample / 'sync.o', dest / 'sync.o')
        live = next(x for x in old['native'] if x['mode'] == 'live')
        command = live['selectionCommand'].copy()
        assert Path(command[4]) == source / sample / 'live.o'
        command[4] = str(dest / 'live.o')
        recipe = dest / 'command.json'
        recipe.write_text(json.dumps(command, indent=2) + '\n')
        audit = dest / 'audit.json'
        env = dict(os.environ, NEOCLR_TEST_OPT_LEVEL=r['optimizationLevel'],
                   NEOCLR_LIVE_SELECTION_COMMAND=str(recipe), NEOCLR_LIVE_SELECTION_MODE='live',
                   NEOCLR_LIVE_ELIDE_STACK_ARGUMENTS='1', NEOCLR_LIVE_STACK_ARGUMENT_AUDIT=str(audit))
        invoke = [str(a.runner.resolve()), 'mutation_store::tests::full_http_selection_compiles_with_live_gc', '--exact', '--ignored', '--nocapture']
        x = subprocess.run(invoke, cwd=root, env=env, capture_output=True, text=True, timeout=300)
        case = dict(sample=sample, expectedPerIteration=old['expectedPerIteration'],
                    native=[dict(mode='sync', exitCode=0, reusedFrom=str(source / sample / 'sync.o')),
                            dict(mode='live', command=invoke, selectionCommand=command,
                                 exitCode=x.returncode, stdout=x.stdout, stderr=x.stderr)])
        r['cases'].append(case)
        x.check_returncode()
        rows = json.loads(audit.read_text())['functions']
        count = sum(f['candidateStores'] for f in rows)
        assert f'live stack-argument elision: {count} stores' in x.stderr
        case['removedStores'] = count
        for path in [dest / 'sync.o', dest / 'live.o', source / sample / 'live.o', audit]:
            r['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
    for path in [a.runner.resolve(), source / 'results.json', Path(__file__)]:
        r['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
    r['passed'] = True
finally:
    (out / 'results.json').write_text(json.dumps(r, indent=2) + '\n')

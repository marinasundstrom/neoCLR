"""Audit stack arguments, or explicitly test bounded stack-store guard removal."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--runner', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--elide', action='store_true', help='Enable test-only stack-store guard removal')
a = p.parse_args()
root = Path(__file__).resolve().parents[3]
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
build = a.build.resolve()
source = json.loads((build / 'results.json').read_text())
assert source['passed']
runner = a.runner.resolve()
r = {'passed': False, 'scope': __doc__, 'elide': a.elide, 'commands': [], 'inputs': {}}
try:
    for sample in ['plain', 'json']:
        original = next(c for c in source['commands'] if c['sample'] == sample and c['mode'] == 'live')
        command = original['selectionCommand'].copy()
        assert Path(command[4]).resolve() == build / (sample + '-live.o')
        command[4] = str(out / (sample + '.o'))
        path = out / (sample + '-command.json')
        path.write_text(json.dumps(command, indent=2) + '\n')
        audit = out / (sample + '-audit.json')
        invoke = [str(runner), 'mutation_store::tests::full_http_selection_compiles_with_live_gc', '--exact', '--ignored', '--nocapture']
        env = dict(os.environ, NEOCLR_TEST_OPT_LEVEL=source['optimizationLevel'],
                   NEOCLR_LIVE_SELECTION_COMMAND=str(path), NEOCLR_LIVE_SELECTION_MODE='live',
                   NEOCLR_LIVE_STACK_ARGUMENT_AUDIT=str(audit))
        env.pop('NEOCLR_LIVE_ELIDE_STACK_ARGUMENTS', None)
        if a.elide:
            env['NEOCLR_LIVE_ELIDE_STACK_ARGUMENTS'] = '1'
        x = subprocess.run(invoke, cwd=root, env=env, capture_output=True, text=True, timeout=300)
        row = {'sample': sample, 'command': invoke, 'selectionCommand': command,
               'auditPath': str(audit), 'exitCode': x.returncode, 'stdout': x.stdout, 'stderr': x.stderr}
        r['commands'].append(row)
        x.check_returncode()
        report = json.loads(audit.read_text())
        rows = report['functions']
        assert rows and len({f['id'] for f in rows}) == len(rows)
        assert all(0 <= f['candidateStores'] <= f['guardedStores'] for f in rows)
        assert all(not f['stackParameters'] for f in rows if not f['local'] or f['addressTaken'] or not f['callSites'])
        row['objectMatchesRetained'] = Path(command[4]).read_bytes() == (build / (sample + '-live.o')).read_bytes()
        if a.elide:
            expected = sum(f['candidateStores'] for f in rows)
            assert expected > 0
            assert f'live stack-argument elision: {expected} stores' in x.stderr
            assert not row['objectMatchesRetained']
        else:
            assert row['objectMatchesRetained']
        for item in [Path(command[4]), audit, build / (sample + '-live.o')]:
            r['inputs'][str(item)] = hashlib.sha256(item.read_bytes()).hexdigest()
        (out / 'results.json').write_text(json.dumps(r, indent=2) + '\n')
    for item in [build / 'results.json', runner, Path(__file__), root / 'runtime/codegen/src/mutation_stack_audit.rs', root / 'runtime/codegen/src/mutation_store.rs', root / 'runtime/codegen/src/values.rs']:
        r['inputs'][str(item)] = hashlib.sha256(item.read_bytes()).hexdigest()
    r['passed'] = True
finally:
    (out / 'results.json').write_text(json.dumps(r, indent=2) + '\n')

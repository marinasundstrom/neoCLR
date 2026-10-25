"""Execute retained standalone selections through managed JIT with/without stack guard removal.

Correctness/collection/cleanup qualification only: includes JIT compilation, not timing evidence.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--elide-forwarded-stack-arguments',action='store_true')
p.add_argument('--expected-audit-root',type=Path,help='Matching forwarding audit directory for exact transformed-store counts')
p.add_argument('--borrow-stores',action='store_true')
p.add_argument('--build', type=Path, required=True)
p.add_argument('--runner', type=Path, required=True)
p.add_argument('--host', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
assert not a.elide_forwarded_stack_arguments or a.expected_audit_root is not None
root = Path(__file__).resolve().parents[3]
build = json.loads((a.build / 'results.json').read_text())
assert build['passed']
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
r = dict(passed=False, forwardedElision=a.elide_forwarded_stack_arguments, borrowedStores=a.borrow_stores, scope=__doc__, trials=[], inputs={}, iterations=build['iterations'])
try:
    for case in build['cases']:
        if a.elide_forwarded_stack_arguments:
            audit_path = a.expected_audit_root / (case['sample'] + '-audit.json')
            audit = json.loads(audit_path.read_text())
            assert audit['forwarding']
            expected_removed = sum(f['candidateStores'] for f in audit['functions'])
            r['inputs'][str(audit_path.resolve())] = hashlib.sha256(audit_path.read_bytes()).hexdigest()
        else:
            expected_removed = case['removedStores']
        original = next(x['selectionCommand'] for x in case['native'] if x['mode'] == 'live')
        assert original[1] == '--closed-world'
        command = [original[0], '--jit-managed', *original[1:4], str(a.host.resolve()), *original[5:]]
        recipe = out / (case['sample'] + '-command.json')
        recipe.write_text(json.dumps(command, indent=2) + '\n')
        for enabled in [False, True]:
            env = dict(os.environ, NEOCLR_TEST_OPT_LEVEL=build['optimizationLevel'],
                       NEOCLR_LIVE_SELECTION_COMMAND=str(recipe), NEOCLR_LIVE_SELECTION_MODE='live')
            env.pop('NEOCLR_LIVE_BORROW_STORES', None)
            if a.borrow_stores: env['NEOCLR_LIVE_BORROW_STORES']='1'
            env.pop('NEOCLR_LIVE_ELIDE_FORWARDED_STACK_ARGUMENTS', None)
            if enabled and a.elide_forwarded_stack_arguments:
                env['NEOCLR_LIVE_ELIDE_FORWARDED_STACK_ARGUMENTS'] = '1'
            env.pop('NEOCLR_LIVE_ELIDE_STACK_ARGUMENTS', None)
            if enabled:
                env['NEOCLR_LIVE_ELIDE_STACK_ARGUMENTS'] = '1'
            launch = [str(a.runner.resolve()), 'mutation_store::tests::full_http_selection_compiles_with_live_gc', '--exact', '--ignored', '--nocapture']
            x = subprocess.run(launch, cwd=root, env=env, capture_output=True, text=True, timeout=300)
            row = dict(sample=case['sample'], elision=enabled, command=launch, selectionCommand=command,
                       exitCode=x.returncode, stdout=x.stdout, stderr=x.stderr)
            r['trials'].append(row)
            x.check_returncode()
            expected = case['expectedPerIteration']
            assert expected * build['iterations'] in x.stdout
            assert x.stdout.count(expected) == build['iterations']
            jit = json.loads(next(s[len('JIT_RESULT '):] for s in x.stderr.splitlines() if s.startswith('JIT_RESULT ')))
            assert jit['guestExitCode'] == 0
            row['jitResult'] = jit
            def numbers(prefix):
                return list(map(int, next(s.split()[1:] for s in x.stderr.splitlines() if s.startswith(prefix + ' '))))
            counts = numbers('CANDIDATE')
            assert len(counts) == 6 and counts[0] == counts[1] + counts[2] and counts[1] > 0 and counts[5] == 0
            metadata = numbers('INDEX_METADATA')
            assert len(metadata) == 3 and metadata[0] > 0 and metadata[1] == 0
            removed = [int(s.split(':')[1].split()[0]) for s in x.stderr.splitlines() if s.startswith('live stack-argument elision:')]
            assert removed == ([expected_removed] if enabled else [])
            row.update(candidateCounts=counts, indexMetadata=metadata, removedStores=removed, outputMatches=True)
        for path in [Path(x) for x in original if Path(x).is_file()]:
            r['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
    for path in [a.build / 'results.json', a.runner, a.host, Path(__file__)]:
        r['inputs'][str(path.resolve())] = hashlib.sha256(path.read_bytes()).hexdigest()
    r['passed'] = True
finally:
    (out / 'results.json').write_text(json.dumps(r, indent=2) + '\n')

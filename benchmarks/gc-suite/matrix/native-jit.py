"""Benchmark unchanged standard samples in JIT using a completed AOT cohort's hosts.

Invocation CPU/wall excludes compiler/startup; compilation stages remain separate.
Requires qualified retained compiler/recipes, as documented by the standard suite.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import statistics
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--aot', type=Path, required=True)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--rounds', type=int, default=3)
p.add_argument('--runner', type=Path, default=Path('target/release/deps/neoclr_codegen-12f33f26d2ff4df0'))
a = p.parse_args()
assert a.rounds > 0
root = Path(__file__).resolve().parents[3]
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
build_path = a.build.resolve() / 'results.json'
aot_path = a.aot.resolve() / 'standalone/results.json'
build = json.loads(build_path.read_text())
aot = json.loads(aot_path.read_text())
assert build['passed'] and aot['passed'] and build['stableByrefs']
assert build['suite'] == 'allocation-patterns' and build['iterations'] == 1
spec = json.loads((root / 'benchmarks/gc-suite/samples.json').read_text())
for case in spec['cases']:
    path = root / 'benchmarks/gc-suite' / case['source']
    assert build['inputs'][str(path)] == hashlib.sha256(path.read_bytes()).hexdigest()
r = dict(passed=False,scope=__doc__,rounds=a.rounds,commands=[],trials=[],rows=[],inputs={})
flags = ['NEOCLR_LIVE_ELIDE_STACK_ARGUMENTS', 'NEOCLR_LIVE_FORWARD_STACK_ARGUMENTS',
         'NEOCLR_LIVE_ELIDE_FORWARDED_STACK_ARGUMENTS', 'NEOCLR_LIVE_SSA_BYREF_ARGUMENTS']

def run(command, env=None):
    command = list(map(str, command))
    x = subprocess.run(command, cwd=root, env=env, capture_output=True, text=True, timeout=300)
    row = dict(command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr)
    r['commands'].append(row)
    x.check_returncode()
    return row

try:
    hosts = {}
    policies = {'sync':'sync', 'live':'live-budget-1024-16384-hybrid-inline'}
    for mode, policy in policies.items():
        recipe = next(c['command'].copy() for c in aot['commands'] if '-o' in c['command']
                      and c['command'][c['command'].index('-o') + 1].endswith('compute-' + policy))
        obj = next(n['selectionCommand'][4] for n in build['cases'][0]['native'] if n['mode'] == mode)
        assert obj in recipe
        if mode == 'live':
            assert '-DNEOCLR_LIVE_NO_STORE_CACHE' in recipe
            assert '-DNEOCLR_GC_ALLOCATION_BUDGET=1024' in recipe
            assert '-DNEOCLR_GC_PUBLICATION_BUDGET=16384' in recipe
        command = [v for v in recipe if v != obj and not v.endswith('/main.c')] + ['-dynamiclib']
        host = out / (mode + '.dylib')
        command[command.index('-o') + 1] = str(host)
        run(command)
        hosts[mode] = host
    for repetition in range(a.rounds):
        for case in build['cases']:
            for mode in (['sync','live'] if repetition % 2 == 0 else ['live','sync']):
                original = next(n['selectionCommand'] for n in case['native'] if n['mode'] == mode)
                command = [original[0], '--jit-managed', *original[1:4], str(hosts[mode]), *original[5:]]
                selection = out / (case['sample'] + '-' + mode + '.json')
                selection.write_text(json.dumps(command,indent=2) + '\n')
                env = dict(os.environ, NEOCLR_TEST_OPT_LEVEL=build['optimizationLevel'],
                           NEOCLR_LIVE_SELECTION_MODE=mode, NEOCLR_LIVE_SELECTION_COMMAND=str(selection))
                for flag in flags:
                    env.pop(flag,None)
                    if mode == 'live':
                        env[flag] = '1'
                print(repetition,case['sample'],mode,flush=True)
                result = run([a.runner.resolve(), 'mutation_store::tests::full_http_selection_compiles_with_live_gc',
                              '--exact','--ignored','--nocapture'], env)
                assert case['expectedPerIteration'] in result['stdout']
                lines = result['stderr'].splitlines()
                profile = next(l.split()[1:] for l in lines if l.startswith('PROFILE '))
                jit = json.loads(next(l[len('JIT_RESULT '):] for l in lines if l.startswith('JIT_RESULT ')))
                assert jit['guestExitCode'] == 0
                row = dict(sample=case['sample'],execution='jit',collector=mode,round=repetition,
                           cpuSeconds=float(profile[0]),wallSeconds=float(profile[1]),jit=jit,
                           allocations=int(profile[2]),allocatedBytes=int(profile[3]),collections=int(profile[4]),
                           peakLiveBytes=int(profile[5]),peakArenaBytes=int(profile[6]))
                if mode == 'live':
                    counts = next(list(map(int,l.split()[1:])) for l in lines if l.startswith('CANDIDATE '))
                    assert counts[0] == counts[1] + counts[2] and counts[5] == 0
                    row['candidateCounts'] = counts
                r['trials'].append(row)
                (out/'progress.json').write_text(json.dumps(r,indent=2)+'\n')
    for case in build['cases']:
        for mode in policies:
            trials = [t for t in r['trials'] if t['sample']==case['sample'] and t['collector']==mode]
            aot_trials = [t for t in aot['trials'] if t['sample']==case['sample'] and t['mode']==policies[mode]]
            assert len(trials)==a.rounds and aot_trials
            assert {(t['allocations'],t['allocatedBytes']) for t in trials} == {(t['allocations'],t['allocatedBytes']) for t in aot_trials}
            r['rows'].append(dict(sample=case['sample'],collector=mode,
                                 jitCpuSeconds=statistics.median(t['cpuSeconds'] for t in trials),
                                 aotCpuSeconds=statistics.median(t['cpuSeconds'] for t in aot_trials),
                                 jitWallSeconds=statistics.median(t['wallSeconds'] for t in trials)))
    for path in [Path(__file__), build_path, aot_path, a.runner.resolve(), *hosts.values()]:
        r['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
    for case in build['cases']:
        for native in case['native']:
            for value in native['selectionCommand']:
                path = Path(value)
                if path.is_file():
                    r['inputs'][str(path.resolve())] = hashlib.sha256(path.read_bytes()).hexdigest()
    r['passed'] = True
finally:
    (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

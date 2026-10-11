#!/usr/bin/env python3
"""Same Raven source, interpreter, and references; vary only async lowering.

Private POC host excludes metadata load/verification and reports fresh-invocation
wall time and object counts. It is not a JIT/AOT comparison or framework feature.
"""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import statistics
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--recipe', type=Path, required=True, help='Passing runtime-await validation.json')
p.add_argument('--host', type=Path, required=True, help='Release runtime_async_interpreter example')
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
r = json.loads(a.recipe.read_text())
if not r['passed']:
    p.error('recipe must have passed')
output = a.output.resolve()
output.mkdir(parents=True, exist_ok=False)
here = Path(__file__).resolve().parent
host = a.host.resolve()
base = r['commands'][0]['command']
provider = here / 'RuntimeAsyncBridge.rvn'
bundle = Path(base[base.index('--core-reference') + 1]).parent.parent
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
report = dict(passed=False, platform=platform.platform(), measurement='fresh interpreter invocation; load and verification excluded',
              repetitions=4, discardedWarmupsPerProcess=1, rounds=2, host=str(host), hostSha256=sha(host),
              recipeSha256=sha(a.recipe), providerSha256=sha(provider), builds=[], runs=[], summaries=[])
try:
    for pending in [False, True]:
        for count in [10, 100]:
            case = f'{"pending" if pending else "completed"}-{count}'
            source = output / (case + '.rvn')
            source.write_text((here / 'RuntimeAwaitBenchmark.rvn').read_text()
                              .replace('let pending = true', 'let pending = ' + str(pending).lower())
                              .replace('let operations = 100', f'let operations = {count}'))
            assemblies = {}
            for mode in ['state-machine', 'runtime']:
                assembly = output / (case + '-' + mode + '.dll')
                command = base[:base.index('-o')] + ['-o', str(assembly), str(source), str(provider)]
                if mode == 'state-machine':
                    flag = command.index('--runtime-async-poc')
                    del command[flag:flag + 2]
                result = subprocess.run(command, capture_output=True, text=True, timeout=120)
                report['builds'].append(dict(case=case, mode=mode, command=command, exitCode=result.returncode,
                                             stdout=result.stdout, stderr=result.stderr, sourceSha256=sha(source)))
                if result.returncode:
                    raise RuntimeError(result.stderr)
                assemblies[mode] = assembly
            for round_index in range(2):
                modes = ['state-machine', 'runtime'] if round_index == 0 else ['runtime', 'state-machine']
                for mode in modes:
                    command = [str(host), str(assemblies[mode]), str(bundle), '4']
                    result = subprocess.run(command, capture_output=True, text=True, timeout=120)
                    if result.returncode or result.stderr:
                        raise RuntimeError(result.stderr)
                    data = json.loads(result.stdout)
                    if len(data['runs']) != 4 or any(run['liveObjects'] != 0 for run in data['runs']):
                        raise RuntimeError('missing samples or retained objects')
                    report['runs'].append(dict(case=case, mode=mode, round=round_index, command=command,
                                               assemblySha256=sha(assemblies[mode]), **data))
            summary = dict(case=case, operations=count)
            for mode in ['state-machine', 'runtime']:
                observations = [sample for run in report['runs'] if run['case'] == case and run['mode'] == mode
                                for sample in run['runs'][1:]]
                summary[mode] = {metric: statistics.median(sample[metric] for sample in observations)
                                 for metric in ['seconds', 'allocatedObjects', 'peakObjects', 'reclaimedObjects', 'liveObjects']}
            summary['runtimeOverStateMachine'] = summary['runtime']['seconds'] / summary['state-machine']['seconds']
            report['summaries'].append(summary)
    report['passed'] = True
finally:
    (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report['summaries'], indent=2))

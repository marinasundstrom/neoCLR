#!/usr/bin/env python3
"""Matched async lowerings across interpreter/JIT/AOT; private macOS ARM64 POC."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import statistics
import subprocess
import time

HERE = Path(__file__).resolve().parent
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--recipe', type=Path, required=True)
p.add_argument('--qualification', type=Path, default=HERE / 'native-validation.json')
p.add_argument('--host', type=Path, required=True, help='runtime_async_interpreter release host')
p.add_argument('--output', type=Path, required=True)
p.add_argument('--engines', nargs='+', choices=['interpreter', 'jit', 'aot'], default=['interpreter', 'jit', 'aot'])
p.add_argument('--counts', nargs='+', type=int, choices=[10, 100], default=[10, 100])
p.add_argument('--rounds', type=int, default=2)
p.add_argument('--stack-cache-limit', type=int, choices=range(129), default=8)
a = p.parse_args()
if a.rounds < 2 or a.rounds % 2 or len(set(a.engines)) != len(a.engines) or len(set(a.counts)) != len(a.counts):
    p.error('use an even number of rounds >= 2 and unique engines/counts')
if platform.system() != 'Darwin' or platform.machine() != 'arm64':
    p.error('requires macOS ARM64 carrier')
r = json.loads(a.recipe.read_text())
q = json.loads(a.qualification.read_text())
if not r['passed'] or not q['passed']:
    p.error('recipes must have passed')
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
base = r['commands'][0]['command']
bundle = Path(base[base.index('--core-reference') + 1]).parent.parent
commands = [item['command'] for item in q['commands']]
link = next(c.copy() for c in commands if '-dynamiclib' in c)
link.append(f'-DNEOCLR_ASYNC_STACK_CACHE_LIMIT={a.stack_cache_limit}')
compile_native = next(c.copy() for c in commands if '--closed-world' in c and '--jit-run' not in c)
report = dict(passed=False, platform=platform.platform(),
              processor=subprocess.check_output(['sysctl', '-n', 'machdep.cpu.brand_string'], text=True).strip(),
              repetitions=4, rounds=a.rounds, engines=a.engines, counts=a.counts, stackCacheLimit=a.stack_cache_limit,
              measurement='fresh invocation including heap/scope teardown; load and compilation excluded',
              nativeAllocationUnit='GC blocks, excluding retained native stacks; not interpreter object counts',
              discardedWarmupsPerProcess=1, inputsSha256={}, commands=[], runs=[], summaries=[])

def run(command):
    started = time.perf_counter()
    result = subprocess.run(command, capture_output=True, text=True, timeout=180)
    elapsed = time.perf_counter() - started
    index = len(report['commands'])
    log = out / f'command-{index}.stdout'
    log.write_text(result.stdout)
    report['commands'].append(dict(command=command, seconds=elapsed, exitCode=result.returncode,
                                   stdoutFile=str(log), stdoutSha256=sha(log), stderr=result.stderr))
    if result.returncode:
        raise RuntimeError(result.stderr or result.stdout)
    return result, elapsed

try:
    # Pin current compiler, runtime helpers and references, rather than assuming
    # old qualification artifacts describe the binaries measured today.
    for value in [*q['inputsSha256'], *base, *link, *compile_native, str(a.host), str(a.recipe), str(a.qualification),
                  str(HERE / 'RuntimeAsyncBridge.rvn'), str(HERE / 'RuntimeAwaitBenchmark.rvn'),
                  str(HERE / 'native-benchmark-host.c'), __file__]:
        path = Path(value)
        if path.is_file():
            report['inputsSha256'][str(path.resolve())] = sha(path)
    host = out / 'host.dylib'
    link = [str(HERE / 'native-benchmark-host.c') if x.endswith('/jit-managed/host.c') else x for x in link]
    link[link.index('-o') + 1] = str(host)
    run(link)
    for pending in [False, True]:
        for count in a.counts:
            case = f'{"pending" if pending else "completed"}-{count}'
            source = out / (case + '.rvn')
            source.write_text((HERE / 'RuntimeAwaitBenchmark.rvn').read_text()
                              .replace('let pending = true', 'let pending = ' + str(pending).lower())
                              .replace('let operations = 100', f'let operations = {count}'))
            invocations = {}
            for lowering in ['state-machine', 'runtime']:
                assembly = out / f'{case}-{lowering}.dll'
                command = base[:base.index('-o')] + ['-o', str(assembly), str(source), str(HERE / 'RuntimeAsyncBridge.rvn')]
                if lowering == 'state-machine':
                    flag = command.index('--runtime-async-poc')
                    del command[flag:flag + 2]
                run(command)
                obj = assembly.with_suffix('.o')
                native = compile_native.copy()
                native[2], native[4] = str(assembly), str(obj)
                run(native)
                executable = assembly.with_suffix('')
                aot = [x for x in link if x != '-dynamiclib'] + ['-DNEOCLR_BENCH_AOT', str(obj)]
                aot[aot.index('-o') + 1] = str(executable)
                run(aot)
                jit = [native[0], '--jit-managed', *native[1:]]
                jit[5] = str(host)
                invocations[lowering] = {'interpreter': [str(a.host.resolve()), str(assembly), str(bundle), '4'],
                                         'jit': jit, 'aot': [str(executable)]}
                for artifact in [source, assembly, obj, executable, host]:
                    report['inputsSha256'][str(artifact)] = sha(artifact)
            for round_index in range(a.rounds):
                lowerings = ['state-machine', 'runtime'][::1 if round_index % 2 == 0 else -1]
                engines = a.engines[::1 if round_index % 2 == 0 else -1]
                for engine in engines:
                    for lowering in lowerings:
                        result, wall = run(invocations[lowering][engine])
                        phases = {}
                        if engine == 'interpreter':
                            data = json.loads(result.stdout)
                            samples = data['runs']
                            phases['prepareSeconds'] = data['prepareSeconds']
                            if result.stderr or any(s['liveObjects'] or s['stdout'] for s in samples):
                                raise RuntimeError('interpreter retained objects or diagnostics')
                        else:
                            lines = result.stderr.splitlines()
                            samples = [json.loads(x.removeprefix('NATIVE_SAMPLE ')) for x in lines if x.startswith('NATIVE_SAMPLE ')]
                            for line in lines:
                                if line.startswith('JIT_RESULT '):
                                    phases = json.loads(line.removeprefix('JIT_RESULT '))
                                    if phases['guestExitCode']:
                                        raise RuntimeError('JIT guest failed')
                                elif not line.startswith('NATIVE_SAMPLE '):
                                    raise RuntimeError('unexpected native diagnostic: ' + line)
                            if result.stdout or (engine == 'jit' and not phases):
                                raise RuntimeError('unexpected output/missing JIT metrics')
                        if len(samples) != 4:
                            raise RuntimeError('expected four fresh invocations')
                        report['runs'].append(dict(case=case, engine=engine, lowering=lowering,
                                                   round=round_index, processSeconds=wall, phases=phases, samples=samples))
            for engine in a.engines:
                summary = dict(case=case, engine=engine)
                for lowering in ['state-machine', 'runtime']:
                    samples = [s['seconds'] for run_data in report['runs']
                               if (run_data['case'], run_data['engine'], run_data['lowering']) == (case, engine, lowering)
                               for s in run_data['samples'][1:]]
                    summary[lowering] = dict(median=statistics.median(samples), minimum=min(samples), maximum=max(samples))
                summary['runtimeOverStateMachine'] = summary['runtime']['median'] / summary['state-machine']['median']
                report['summaries'].append(summary)
            print(case, flush=True)
    for path, expected in report['inputsSha256'].items():
        if sha(path) != expected:
            raise RuntimeError('input changed during measurement: ' + path)
    report['passed'] = True
finally:
    (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report['summaries'], indent=2))

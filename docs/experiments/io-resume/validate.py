#!/usr/bin/env python3
"""Measure real owner-local TCP completion/resume; no compiler or HTTP claims."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
here = Path(__file__).resolve().parent
root = here.parents[2]
native = root / 'runtime/native'
legacy = root / 'docs/experiments/aot-console'
live = native / 'gc/concurrent'
report = dict(passed=False, platform=platform.platform(), commands=[], samples=[])


def run(command):
    command = list(map(str, command))
    result = subprocess.run(command, capture_output=True, text=True, timeout=60, cwd=out)
    report['commands'].append(dict(command=command, exitCode=result.returncode,
                                   stdout=result.stdout, stderr=result.stderr))
    result.check_returncode()
    return result.stdout


try:
    cc = [subprocess.check_output(['xcrun', '--find', 'clang'], text=True).strip(),
          '-isysroot', subprocess.check_output(['xcrun', '--show-sdk-path'], text=True).strip()]
    flags = ['-std=c11', '-O2', '-Wall', '-Wextra', '-Werror',
             '-Wno-deprecated-declarations', '-pthread', '-Wl,-dead_strip',
             '-I', native / 'async', '-I', native / 'gc', '-I', legacy]
    run(['python3', live / 'generate.py', '--output', out / 'live-gc.c'])
    policy = json.loads((live / 'policy.json').read_text())
    common = [here / 'probe.c', native / 'async/native-runtime.c',
              native / 'gc/root-probe.c', legacy / 'native-stack.c',
              legacy / 'socket-listener.c', legacy / 'text-arena.c']
    for mode in ('sync', 'concurrent'):
        defs = (['-DNEOCLR_NATIVE_GC'] if mode == 'sync' else
                ['-D' + value for value in policy['defines']] + ['-DTRACE_LIVE', '-I', live])
        gc = [native / 'gc/native-gc.c'] if mode == 'sync' else [out / 'live-gc.c', live / 'gate.c']
        for variant, extra in [('optimized', []), ('ubsan', ['-fsanitize=undefined', '-fno-sanitize-recover=all'])]:
            binary = out / f'{mode}-{variant}'
            run([*cc, *flags, *defs, *extra, *common, *gc, '-o', binary])
            for shape in range(4):
                run([binary, shape, 16])
    for mode in ('sync', 'concurrent'):
        defs = (['-DNEOCLR_NATIVE_GC'] if mode == 'sync' else
                ['-D' + value for value in policy['defines']] + ['-DTRACE_LIVE', '-I', live])
        gc = [native / 'gc/native-gc.c'] if mode == 'sync' else [out / 'live-gc.c', live / 'gate.c']
        binary = out / f'{mode}-traced'
        run([*cc, *flags, *defs, '-DIO_TRACE', '-DNEOCLR_ASYNC_TRACE', *common, *gc, '-o', binary])
        run([binary, 2, 16])
        raw = out / f'{mode}-trace.json'
        (out / 'trace.json').rename(raw)
        run(['python3', here.parent / 'execution-trace/async-export.py', raw,
             out / f'{mode}.trace.json'])
    # Fixed baseline screen: three alternating pairs, 256 TCP completions per cell.
    # Shape 0 idle, 1 sixteen CPU callbacks, 2 plus allocations, 3 one long CPU callback.
    for shape in range(4):
        for pair in range(3):
            for mode in (('sync', 'concurrent') if pair % 2 == 0 else ('concurrent', 'sync')):
                sample = json.loads(run([out / f'{mode}-optimized', shape, 256]))
                sample.update(mode=mode, pair=pair)
                report['samples'].append(sample)
    paths = [*common, Path(__file__), out / 'live-gc.c']
    paths += [p for p in (native / 'gc').rglob('*') if p.suffix in ('.c', '.h', '.py', '.json')]
    paths += list((native / 'async').glob('*.h')) + list(legacy.glob('socket-*.h'))
    report['inputs'] = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
    report['passed'] = True
finally:
    (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
print('PASS real TCP completion/resume with both collectors')

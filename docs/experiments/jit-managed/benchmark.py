#!/usr/bin/env python3
"""Compare fresh-process execution of the qualified Index/Range test assembly."""
import argparse
import hashlib
import json
import platform
from pathlib import Path
import statistics
import subprocess
import time


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--qualification', type=Path, required=True)
    parser.add_argument('--build', type=Path, required=True)
    parser.add_argument('--runtime', type=Path, default=Path('target/release/neoclr'))
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--repetitions', type=int, default=9)
    args = parser.parse_args()
    if args.repetitions < 3:
        parser.error('Use at least three repetitions')
    qualified = json.loads(args.qualification.read_text())
    build = args.build.resolve()
    if not qualified['passed'] or sha(build / 'app.dll') != qualified['appSha256']:
        raise ValueError('Requires the unchanged qualified application')
    jit = qualified['jitCommand']
    host = jit[jit.index('--closed-world') + 3]
    if sha(jit[0]) != qualified['jitSha256'] or sha(host) != qualified['hostSha256'] or sha(args.runtime) != qualified['interpreterSha256']:
        raise ValueError('Qualified tools/host changed')
    build_report = json.loads((build / 'build.json').read_text())
    if sha(build / 'app') != build_report['artifacts']['app']:
        raise ValueError('Qualified AOT executable changed')
    interpreter = [str(args.runtime.resolve()), 'run', str(build / 'app.dll')]
    for flag in ['--system', '--module', '--object-root']:
        for i, item in enumerate(jit):
            if item == flag:
                interpreter += [flag, jit[i + 1]]
    interpreter += ['--instructions', '100000000']
    commands = {'aot': [str(build / 'app')], 'interpreter': interpreter, 'jit': jit}
    expected = next(r for r in qualified['cases'] if r['label'] == 'suite' and r['mode'] == 'aot')
    result = dict(passed=False, platform=platform.platform(), machine=platform.machine(),
                  qualificationSha256=sha(args.qualification), appSha256=sha(build / 'app.dll'),
                  aotSha256=sha(build / 'app'), jitSha256=sha(jit[0]),
                  interpreterSha256=sha(args.runtime), hostSha256=sha(host),
                  commands=commands, warmups=[], samples=[], summary={})
    args.output.parent.mkdir(parents=True, exist_ok=True)
    if args.output.exists():
        raise ValueError('Use a fresh output file')

    def run(mode, round_index):
        started = time.perf_counter()
        process = subprocess.run(commands[mode], capture_output=True, text=True, timeout=180)
        sample = dict(mode=mode, round=round_index, processSeconds=time.perf_counter() - started)
        stderr = process.stderr
        if mode == 'jit':
            lines = stderr.splitlines(keepends=True)
            summaries = [line for line in lines if line.startswith('JIT_RESULT ')]
            if len(summaries) != 1:
                raise RuntimeError('Missing JIT result: ' + stderr)
            sample['jit'] = json.loads(summaries[0].removeprefix('JIT_RESULT '))
            if sample['jit']['guestExitCode'] != 0:
                raise RuntimeError('JIT guest failed')
            stderr = ''.join(line for line in lines if not line.startswith('JIT_RESULT '))
        if process.returncode != 0 or process.stdout != expected['stdout'] or stderr != expected['stderr']:
            raise RuntimeError('Runner parity failed: ' + mode + '\n' + process.stdout + stderr)
        sample['stdoutSha256'] = hashlib.sha256(process.stdout.encode()).hexdigest()
        return sample

    try:
        modes = list(commands)
        for mode in modes:
            result['warmups'].append(run(mode, -1))
        for index in range(args.repetitions):
            order = modes[index % 3:] + modes[:index % 3]
            for mode in order:
                result['samples'].append(run(mode, index))
            print(f'Completed round {index + 1}/{args.repetitions}', flush=True)
        for mode in modes:
            values = [s['processSeconds'] for s in result['samples'] if s['mode'] == mode]
            result['summary'][mode] = dict(medianSeconds=statistics.median(values), minSeconds=min(values), maxSeconds=max(values))
        for field in ['compileAndFinalizeSeconds', 'invocationSeconds']:
            values = [s['jit'][field] for s in result['samples'] if s['mode'] == 'jit']
            result['summary'][field] = dict(medianSeconds=statistics.median(values), minSeconds=min(values), maxSeconds=max(values))
        result['passed'] = True
    finally:
        args.output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result['summary'], indent=2))


if __name__ == '__main__':
    main()

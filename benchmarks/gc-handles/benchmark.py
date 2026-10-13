#!/usr/bin/env python3
"""Matched owned-handle admission matrix; heap construction excluded."""
import argparse
import hashlib
import json
import platform
import statistics
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / 'docs/experiments/aot-console'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--baseline', required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--rounds', type=int, default=5)
    args = parser.parse_args()
    if args.rounds < 1:
        parser.error("rounds must be positive")
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, platform=platform.platform(), machine=platform.machine(),
                  baseline=args.baseline, commands=[], cases=[])

    def run(command):
        command = list(map(str, command))
        result = subprocess.run(command, capture_output=True, text=True, timeout=180, cwd=out)
        report['commands'].append(dict(command=command, exitCode=result.returncode))
        if result.returncode:
            raise RuntimeError(result.stdout + result.stderr)
        return result.stdout

    try:
        if platform.system() == 'Windows':
            cc = ['cl', '/nologo', '/W4', '/WX', '/O2', '/std:c11', '/experimental:c11atomics',
                  '/MT', '/DNEOCLR_NATIVE_GC', '/I' + str(BASE)]
        else:
            cc = ['clang', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-DNEOCLR_NATIVE_GC', '-I', str(BASE)]
            if platform.system() == 'Darwin':
                cc[0] = run(['xcrun', '--sdk', 'macosx', '--find', 'clang']).strip()
                cc += ['-isysroot', run(['xcrun', '--sdk', 'macosx', '--show-sdk-path']).strip()]
        report['revision'] = run(['git', '-C', ROOT, 'rev-parse', 'HEAD']).strip()
        baseline_path = 'runtime/native/gc/native-gc.c'
        if subprocess.run(['git', '-C', ROOT, 'cat-file', '-e', args.baseline + ':' + baseline_path],
                          stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode:
            baseline_path = 'docs/experiments/aot-console/native-gc.c'
        sources = {'baseline': run(['git', '-C', ROOT, 'show', args.baseline + ':' + baseline_path]),
                   'candidate': (ROOT / 'runtime/native/gc/native-gc.c').read_text()}
        report['sourceHashes'] = {k: hashlib.sha256(v.encode()).hexdigest() for k, v in sources.items()}
        report['sourceHashes']['workload'] = hashlib.sha256(Path(__file__).with_name('workload.c').read_bytes()).hexdigest()
        report['inputs'] = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
                            for p in [Path(__file__).resolve(), *sorted((ROOT / 'runtime/native').rglob('*.[ch]')),
                                      BASE / 'root-probe.c', *BASE.glob('*.h')]}
        report['compiler'] = run([cc[0], '--version']) if platform.system() != 'Windows' else 'MSVC: see build commands'
        executables = {}
        for name, source in sources.items():
            src = out / (name + '.c'); src.write_text(source)
            exe = out / (name + ('.exe' if platform.system() == 'Windows' else ''))
            command = cc + [Path(__file__).with_name('workload.c').resolve(), src, BASE / 'root-probe.c']
            command += ['/Fe:' + str(exe)] if platform.system() == 'Windows' else ['-o', exe]
            run(command); executables[name] = exe
        for nodes in [16, 256, 4096]:
            for position, owner_position in [(0, -1), (nodes - 1, -1), (0, 0), (0, nodes - 1),
                                              (nodes // 2, nodes - 1), (nodes - 1, nodes - 1)]:
                iterations = max(10000, 20000000 // nodes)
                case = dict(nodes=nodes, position=position, ownerPosition=owner_position, iterations=iterations, samples=[], warmup=[])
                report['cases'].append(case)
                for name in sources:
                    case['warmup'].append(dict(mode=name, **json.loads(run([executables[name], nodes, iterations, position, owner_position]))))
                for round_index in range(args.rounds):
                    order = ['baseline', 'candidate'] if round_index % 2 == 0 else ['candidate', 'baseline']
                    for name in order:
                        case['samples'].append(dict(round=round_index, mode=name, **json.loads(run([executables[name], nodes, iterations, position, owner_position]))))
                medians = {name: statistics.median(s['seconds'] for s in case['samples'] if s['mode'] == name) for name in sources}
                case['medians'] = medians
                case['speedup'] = medians['baseline'] / medians['candidate']
                print(nodes, position, owner_position, case['speedup'], flush=True)
        report['passed'] = True
    except Exception as error:
        report['error'] = str(error)
        raise
    finally:
        (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()

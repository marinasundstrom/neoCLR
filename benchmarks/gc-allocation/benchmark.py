#!/usr/bin/env python3
"""Matched allocation-only matrix; full validation and first-fit preserved."""
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
        executables = {}
        for name, source in sources.items():
            for diagnostic in [False, True]:
                label = name + ('-diagnostic' if diagnostic else '')
                if diagnostic:
                    begin = source.index('int32_t neoclr_gc_allocate_v1(')
                    end = source.index('/* Opt-in admission', begin)
                    allocation = source[begin:end]
                    marker = '        block *b = (void *)(a->data + at);'
                    if allocation.count(marker) != 1:
                        raise ValueError('allocator traversal hook moved')
                    allocation = allocation.replace(marker, '        lookup_steps++;\n' + marker)
                    source_copy = source[:begin] + allocation + source[end:]
                    # Baseline separately validates all headers; candidate validates
                    # inside the allocation walk already counted above.
                    if '!blocks_valid(a)' in allocation:
                        source_copy = source_copy.replace('        if (!block_valid(a, at)) return 0;',
                            '        lookup_steps++;\n        if (!block_valid(a, at)) return 0;')
                    source_copy = '#include <stdint.h>\nuint64_t lookup_steps;\n' + source_copy
                else:
                    source_copy = source
                src = out / (label + '.c')
                src.write_text(source_copy)
                exe = out / (label + ('.exe' if platform.system() == 'Windows' else ''))
                command = cc + ([('/D' if platform.system() == 'Windows' else '-D') + 'LOOKUP_DIAGNOSTICS'] if diagnostic else [])
                command += [Path(__file__).with_name('workload.c').resolve(), src, BASE / 'root-probe.c']
                command += ['/Fe:' + str(exe)] if platform.system() == 'Windows' else ['-o', exe]
                run(command)
                executables[label] = exe
        matrix = [('objects', 64, 200), ('objects', 1024, 8), ('objects', 4096, 2),
                  ('strings', 1024, 8), ('buffers', 256, 8), ('holes', 1024, 4)]
        for shape, nodes, collections in matrix:
            case = dict(shape=shape, nodes=nodes, allocationBatches=collections, samples=[], warmup=[], diagnostics={})
            report['cases'].append(case)
            for name in sources:
                case['warmup'].append(dict(mode=name, **json.loads(run([executables[name], nodes, collections, shape]))))
            for round_index in range(args.rounds):
                order = ['baseline', 'candidate'] if round_index % 2 == 0 else ['candidate', 'baseline']
                for name in order:
                    case['samples'].append(dict(round=round_index, mode=name, **json.loads(run([executables[name], nodes, collections, shape]))))
            for name in sources:
                case['diagnostics'][name] = json.loads(run([executables[name + '-diagnostic'], nodes, collections, shape]))
            medians = {name: statistics.median(s['seconds'] for s in case['samples'] if s['mode'] == name) for name in sources}
            case['medians'] = medians
            case['speedup'] = medians['baseline'] / medians['candidate']
            print(shape, nodes, medians, case['speedup'], flush=True)
            (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
        report['passed'] = True
    except Exception as error:
        report['error'] = str(error)
        raise
    finally:
        (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()

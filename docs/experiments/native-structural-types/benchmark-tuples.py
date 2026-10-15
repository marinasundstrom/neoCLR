"""Compare native structural tuple and nominal record call/copy costs on macOS ARM64.

Measures already-compiled execution, separately records emission time and retains
raw samples and artifact hashes. Neither workload requires a heap allocator.
"""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import statistics
import subprocess
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--codegen', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--iterations', type=int, default=5_000_000)
    parser.add_argument('--elements', type=int, default=2)
    args = parser.parse_args()
    if platform.system() != 'Darwin' or platform.machine() != 'arm64':
        parser.error('This qualification uses the macOS ARM64 native backend')
    if not 2 <= args.elements <= 16:
        parser.error('elements must be between 2 and 16 (nominal backend field limit)')
    if args.iterations < 1:
        parser.error('iterations must be positive')
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    codegen = args.codegen.resolve()
    host = output / 'host.c'
    host.write_text(r'''
#include <stdint.h>
#include <stdio.h>
#include <time.h>
extern int32_t neoclr_entry_v2(int32_t, int32_t *);
int main(void) {
    uint64_t sum = 0, expected = 0;
    struct timespec begin, end;
    clock_gettime(CLOCK_MONOTONIC, &begin);
    for (int32_t i = 0; i < ITERATIONS; ++i) {
        int32_t value = 0;
        if (neoclr_entry_v2(i & 1023, &value)) return 1;
        sum += value;
        expected += (i & 1023) + 2;
    }
    clock_gettime(CLOCK_MONOTONIC, &end);
    if (sum != expected) return 2;
    double ns = (end.tv_sec - begin.tv_sec) * 1e9 + end.tv_nsec - begin.tv_nsec;
    printf("%.9f\n", ns / ITERATIONS);
    return 0;
}
''')
    report = {'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
              'working_tree_diff_sha256': hashlib.sha256(subprocess.check_output(['git', 'diff'])).hexdigest(),
              'platform': platform.platform(), 'cpu': subprocess.check_output(['sysctl', '-n', 'machdep.cpu.brand_string'], text=True).strip(),
              'iterations_per_sample': args.iterations, 'elements': args.elements,
              'codegen_sha256': hashlib.sha256(codegen.read_bytes()).hexdigest(),
              'cases': {}, 'limitations': ['one machine', 'microbenchmark of integer tuple construction, copy and call',
                  'does not measure boxed unions, GC pressure, interpreter or JIT',
                  'compilation is timed separately; execution excludes process startup',
                  'eight-byte private slots are not a packed or public aggregate ABI']}
    for kind in ['structural', 'nominal']:
        shape = 'tuple<' + ','.join(['Int32'] * args.elements) + '>' if kind == 'structural' else 'Pair'
        declaration = '' if kind == 'structural' else '.type Pair\n' + ''.join(f'.field F{i} Int32\n' for i in range(args.elements)) + '.end\n'
        padding = 'ldc.i4 0\n' * (args.elements - 2)
        source = output / f'{kind}.neoil'
        source.write_text(f'.module PairCost\n{declaration}.function Echo({shape} value) -> {shape}\nldarg value\nret\n.end\n.function Main(Int32 value) -> Int32\n.local {shape} pair\nldarg value\n{padding}ldc.i4 2\nnewobj {shape}\ncall Echo({shape})\nstloc pair\nldloc pair\nldfld 0\nldloc pair\nldfld {args.elements - 1}\nadd\nret\n.end\n')
        obj = output / f'{kind}.o'
        start = time.perf_counter()
        subprocess.run([str(codegen), str(source), 'Main', str(obj)], check=True, capture_output=True)
        emit = time.perf_counter() - start
        subprocess.run(['clang', '-O2', '-Wall', '-Wextra', '-Werror', f'-DITERATIONS={args.iterations}', str(host), str(obj), '-o', str(output / kind)], check=True, capture_output=True)
        report['cases'][kind] = {'emission_seconds': emit, 'object_bytes': obj.stat().st_size,
            'object_sha256': hashlib.sha256(obj.read_bytes()).hexdigest(),
            'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
            'undefined_symbols': subprocess.check_output(['nm', '-u', str(obj)], text=True).splitlines(),
            'samples_ns_per_call': []}
    def run(kind):
        return float(subprocess.check_output([str(output / kind)], text=True))
    for kind in report['cases']:
        run(kind)
    for index in range(9):
        for kind in (['structural', 'nominal'] if index % 2 == 0 else ['nominal', 'structural']):
            report['cases'][kind]['samples_ns_per_call'].append(run(kind))
    for case in report['cases'].values():
        case['median_ns_per_call'] = statistics.median(case['samples_ns_per_call'])
    report['identical_native_objects'] = report['cases']['structural']['object_sha256'] == report['cases']['nominal']['object_sha256']
    report['structural_over_nominal_ratio'] = report['cases']['structural']['median_ns_per_call'] / report['cases']['nominal']['median_ns_per_call']
    (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()

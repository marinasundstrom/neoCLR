"""Compare native scalar union transport with an explicit nominal tagged record.

Both workloads alternate Int32/Int64 payload cases, call Make and Read, and
validate every result. Compilation and process startup are excluded from timing.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--codegen', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--iterations', type=int, default=5_000_000)
    args = parser.parse_args()
    if (platform.system(), platform.machine()) != ('Darwin', 'arm64') or args.iterations < 1:
        parser.error('Requires macOS ARM64 and positive iterations')
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    root = Path(__file__).resolve().parents[3]
    codegen = args.codegen.resolve()
    host = output / 'host.c'
    host.write_text(r'''
#include "text-arena.h"
#include <stdio.h>
#include <time.h>
int main(void) {
    neoclr_aot_context context = {0};
    int64_t sum = 0, expected = 0;
    struct timespec begin, end;
    clock_gettime(CLOCK_MONOTONIC, &begin);
    for (int32_t i = 0; i < ITERATIONS; ++i) {
        int32_t input = (i & 1023) + 2, value = 0;
        if (neoclr_entry_v4(input, &value, &context)) return 1;
        int32_t expected_value = input + ((input & 1) ? -2 : 2);
        if (value != expected_value) return 2;
        sum += value;
        expected += expected_value;
    }
    clock_gettime(CLOCK_MONOTONIC, &end);
    if (sum != expected || context.text.used || context.fault.code) return 2;
    double ns = (end.tv_sec - begin.tv_sec) * 1e9 + end.tv_nsec - begin.tv_nsec;
    printf("%.9f\n", ns / ITERATIONS);
    return 0;
}
''')
    common = '.module UnionCost\n.type class abstract System.Object\n.end\n'
    union = '''.function Make(Int32 value) -> union<Int32,Int64>
ldarg value
ldc.i4 1
and
brtrue Wide
ldarg value
box Int32
castclass union<Int32,Int64>
ret
Wide:
ldarg value
conv.i8
box Int64
castclass union<Int32,Int64>
ret
.end
.function Read(union<Int32,Int64> value) -> Int32
ldarg value
isinst Int32
ref.isnull
brtrue Wide
ldarg value
unbox.any Int32
ldc.i4 2
add
ret
Wide:
ldarg value
unbox.any Int64
conv.i4
ldc.i4 2
sub
ret
.end
.function Main(Int32 value) -> Int32
ldarg value
call Make(Int32)
call Read(union<Int32,Int64>)
ret
.end
'''
    nominal = '''.type Tagged
.field Tag Int32
.field Bits Int64
.end
.function Make(Int32 value) -> Tagged
ldarg value
ldc.i4 1
and
brtrue Wide
ldc.i4 1
ldarg value
conv.i8
newobj Tagged
ret
Wide:
ldc.i4 2
ldarg value
conv.i8
newobj Tagged
ret
.end
.function Read(Tagged value) -> Int32
ldarg value
ldfld 0
ldc.i4 1
ceq
brfalse Wide
ldarg value
ldfld 1
conv.i4
ldc.i4 2
add
ret
Wide:
ldarg value
ldfld 1
conv.i4
ldc.i4 2
sub
ret
.end
.function Main(Int32 value) -> Int32
ldarg value
call Make(Int32)
call Read(Tagged)
ret
.end
'''
    report = {'runtimeRevision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip(),
              'runtimeDiffSha256': hashlib.sha256(subprocess.check_output(['git', 'diff'], cwd=root)).hexdigest(),
              'platform': platform.platform(), 'cpu': subprocess.check_output(['sysctl', '-n', 'machdep.cpu.brand_string'], text=True).strip(),
              'iterationsPerSample': args.iterations, 'cases': {}, 'commands': [],
              'limitations': ['one machine; macOS ARM64 AOT only', 'valid scalar payloads only; no references, records, GC or async',
                              'nominal carrier uses explicit tag convention, structural union uses runtime membership checks',
                              'excludes compilation and process startup; private eight-byte slots, not public ABI'],
              'files': {}}

    def run(command):
        command = list(map(str, command))
        report['commands'].append(command)
        result = subprocess.run(command, cwd=root, capture_output=True, text=True, timeout=180)
        if result.returncode:
            raise RuntimeError(f'{command}: {result.stderr}')
        return result.stdout

    sdk = os.environ.get('SDKROOT') or subprocess.check_output(['xcrun', '--show-sdk-path'], text=True).strip()
    seed = output / 'System.neoil'
    seed.write_text('.module System\n.references ()\n')
    dependency = output / 'Dependency.neoil'
    dependency.write_text('.module Dependency\n.references ()\n')
    files = [Path(__file__).resolve(), codegen, host, seed, dependency, root / 'docs/experiments/aot-console/text-arena.h']
    for name, body in [('structural', union), ('nominal', nominal)]:
        source, obj, executable = [output / (name + suffix) for suffix in ['.neoil', '.o', '']]
        source.write_text(common + body)
        begin = time.perf_counter()
        run([codegen, '--closed-world', '--compile-system', '--reference-arena', source, 'Main', obj, '--system', seed, '--module', dependency])
        emit = time.perf_counter() - begin
        imports = run(['nm', '-u', obj]).split()
        assert not imports, imports
        run(['clang', '-isysroot', sdk, '-O2', '-Wall', '-Wextra', '-Werror', '-arch', 'arm64',
             '-I', root / 'docs/experiments/aot-console', f'-DITERATIONS={args.iterations}', host, obj, '-o', executable])
        report['cases'][name] = {'emissionSeconds': emit, 'objectBytes': obj.stat().st_size,
                                 'imports': imports, 'samplesNsPerCall': []}
        files += [source, obj, executable]
    for name in report['cases']:
        run([output / name])
    for round_index in range(9):
        for name in (['structural', 'nominal'] if round_index % 2 == 0 else ['nominal', 'structural']):
            report['cases'][name]['samplesNsPerCall'].append(float(run([output / name])))
    for case in report['cases'].values():
        case['medianNsPerCall'] = statistics.median(case['samplesNsPerCall'])
    report['structuralOverNominalRatio'] = report['cases']['structural']['medianNsPerCall'] / report['cases']['nominal']['medianNsPerCall']
    report['files'] = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in files}
    (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()

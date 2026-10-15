"""Qualify fresh Raven union overload selection and native payload widening."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', required=True, type=Path)
    parser.add_argument('--bundle', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--string-union', action='store_true', help='Qualify string/int alternatives instead of numeric widening')
    args = parser.parse_args()
    if (platform.system(), platform.machine()) != ('Darwin', 'arm64'):
        parser.error('This qualification requires macOS ARM64')
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    compiler, bundle = args.compiler.resolve(), args.bundle.resolve()
    project = output / 'project'
    shutil.copytree(Path(__file__).parent / ('string-project' if args.string_union else 'widening-project'), project,
                    ignore=shutil.ignore_patterns('bin', 'obj'))
    project_file = project / 'App.rvnproj'
    project_file.write_text(project_file.read_text().replace(
        '../../../../target/structural-integration-libraries-v4/NeoCLR.ClassLibrary.props',
        str(bundle / 'NeoCLR.ClassLibrary.props')))
    commands = []

    def run(label, command, expected=0):
        command = list(map(str, command))
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=180)
        (output / (label + '.stdout')).write_text(result.stdout)
        (output / (label + '.stderr')).write_text(result.stderr)
        commands.append({'label': label, 'command': command, 'exitCode': result.returncode})
        if result.returncode != expected:
            raise RuntimeError(f'{label}: expected {expected}, got {result.returncode}; see {output}')
        return result.stdout

    compiled = run('compile', ['dotnet', compiler, 'neoclr', '--project', project_file])
    artifact = Path(next(line.removeprefix('Native build output: ') for line in compiled.splitlines()
                         if line.startswith('Native build output: ')))
    runtime, codegen = (ROOT / 'target/debug' / name for name in ('neoclr', 'neoclr-codegen'))
    context = ['--system', bundle / 'System.runtime.neox', '--object-root', bundle / 'System.Runtime.dll']
    for name in ['System.Runtime', 'System.Data', 'System.Networking', 'System.Web']:
        context += ['--module', bundle / (name + '.dll')]
    run('interpreter', [runtime, 'run', artifact, '--execution', 'interpreter', *context], 42)
    flags = ['--compile-system', '--reference-arena']
    if args.string_union:
        flags += ['--native-gc']
    obj = output / 'app.o'
    run('aot', [codegen, '--closed-world', artifact, '@entry', obj, *context, *flags])
    imports = run('imports', ['nm', '-u', obj]).split()
    expected_imports = ['_neoclr_gc_callback_read_v1', '_neoclr_gc_complete_invocation_v1',
                        '_neoclr_gc_enter_v1', '_neoclr_gc_entry_check_v1', '_neoclr_gc_leave_v1',
                        '_neoclr_gc_stack_roots_v1', '_neoclr_gc_transient_v1',
                        '_neoclr_string_concat_rooted_v1'] if args.string_union else []
    assert imports == expected_imports, imports
    host = output / 'host.c'
    host.write_text(r'''
#include "text-arena.h"
#if defined(JIT_HOST) || defined(STRING_UNION)
#include "native-gc.h"
#endif
#include <string.h>
typedef int32_t (*entry_v4)(int32_t, int32_t *, neoclr_aot_context *);
static int32_t check(entry_v4 entry) {
#if defined(JIT_HOST) || defined(STRING_UNION)
    uint64_t heap[8192] = {0};
    neoclr_aot_context context = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
#else
    neoclr_aot_context context = {0};
#endif
    int32_t result = -1;
    int32_t status = entry(0, &result, &context);
    if (status || context.fault.code) return 2;
#ifndef STRING_UNION
    if (context.text.used) return 2;
#endif
#if defined(JIT_HOST) || defined(STRING_UNION)
    if (neoclr_root_probe_head_v1() || neoclr_gc_collect_v1(&context, NULL) || context.text.used) return 2;
#endif
#ifdef STRING_UNION
    neoclr_gc_statistics stats = neoclr_gc_statistics_v1();
    if (stats.allocations != 2 || stats.reclaimed_allocations != 2) return 3;
#endif
    return result;
}
#ifdef JIT_HOST
int32_t neoclr_jit_run_v1(const void *entry, int32_t with_args, int32_t argc, const char **argv) {
    if (!entry || with_args || argc < 1 || !argv) return 2;
    entry_v4 callable;
    _Static_assert(sizeof(callable) == sizeof(entry), "entry pointer size");
    memcpy(&callable, &entry, sizeof(callable));
    return check(callable);
}
#else
int main(void) { return check(neoclr_entry_v4); }
#endif
''')
    sdk = os.environ.get('SDKROOT') or subprocess.check_output(['xcrun', '--show-sdk-path'], text=True).strip()
    link = ['clang', '-isysroot', sdk, '-arch', 'arm64', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror',
            '-I', ROOT / 'docs/experiments/aot-console']
    executable, dylib = output / 'app', output / 'host.dylib'
    base = ROOT / 'docs/experiments/aot-console'
    string_sources = [base / name for name in ['native-gc.c', 'root-probe.c', 'text-arena.c']]
    string_sources += [ROOT / 'docs/experiments/aot-fault-details/render.c']
    if args.string_union:
        link += ['-DSTRING_UNION', '-DNEOCLR_NATIVE_GC']
    run('link-aot', [*link, '-fsanitize=undefined,bounds', host, obj,
                     *(string_sources if args.string_union else []), '-o', executable])
    run('execute-aot', [executable], 42)
    assert not (output / 'execute-aot.stderr').read_text()
    base = ROOT / 'docs/experiments/aot-console'
    jit_sources = [base / name for name in ['native-gc.c', 'root-probe.c', 'text-arena.c', 'native-stack.c']]
    jit_sources += [ROOT / 'docs/experiments/aot-fault-details/render.c']
    run('link-jit', [*link, '-DJIT_HOST', '-DNEOCLR_NATIVE_GC', '-dynamiclib', host, *jit_sources, '-o', dylib])
    run('jit', [codegen, '--jit-managed', '--closed-world', artifact, '@entry', dylib, *context, *flags, *([] if args.string_union else ['--native-gc']), '--native-stack-budget'])
    lines = (output / 'jit.stderr').read_text().splitlines()
    assert len(lines) == 1 and lines[0].startswith('JIT_RESULT '), lines
    measurement = json.loads(lines[0].removeprefix('JIT_RESULT '))
    assert measurement['guestExitCode'] == 42, measurement
    files = [Path(__file__).resolve(), compiler, runtime, codegen, project_file, project / 'Main.rvn',
             artifact, obj, host, executable, dylib, ROOT / 'docs/experiments/aot-console/text-arena.h']
    files += jit_sources + sorted(base.glob('*.h'))
    files += sorted(bundle.glob('*.dll')) + [bundle / 'System.runtime.neox', bundle / 'build-evidence.json']
    files += sorted(compiler.parent.glob('Raven*.dll')) + sorted(compiler.parent.glob('NeoCLR*.dll'))
    report = {'schemaVersion': 1,
              'runtimeRevision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              'compilerRevision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=compiler.parent, text=True).strip(),
              'runtimeDiffSha256': hashlib.sha256(subprocess.check_output(['git', 'diff'], cwd=ROOT)).hexdigest(),
              'compilerDiffSha256': hashlib.sha256(subprocess.check_output(['git', 'diff'], cwd=compiler.parent)).hexdigest(),
              'platform': platform.platform(), 'commands': commands, 'imports': imports, 'jit': measurement,
              'checks': (['string and integer alternatives through reordered calls and typed patterns',
                          'interpreter/AOT/JIT return 42', 'native roots and heap cleared after collection',
                          'AOT undefined/bounds sanitizers'] if args.string_union else ['exact overload selected over wider union', 'scalar to record-union widening',
                         'record union widening/reordering preserves payload', 'interpreter/AOT/JIT return 42',
                         'no native imports and zero arena use', 'AOT undefined/bounds sanitizers']),
              'limitations': ['macOS ARM64 only', 'JIT host not sanitizer-instrumented', 'no throughput claim'],
              'files': {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in files}}
    (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'result': 'PASS', 'evidence': str(output / 'results.json')}))


if __name__ == '__main__':
    main()

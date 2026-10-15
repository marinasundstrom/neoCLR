"""Qualify an artifact-only Raven callable-union consumer under native collection."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[3]
FIXTURES = Path(__file__).resolve().parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    if (platform.system(), platform.machine()) != ('Darwin', 'arm64'):
        parser.error('This qualification requires macOS ARM64')
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    compiler = args.compiler.resolve()
    bundle = ROOT / 'target/structural-integrated-libraries-v3'
    library = FIXTURES / 'callable-import-library'
    project = FIXTURES / 'callable-import-project'
    library_artifact = library / 'bin/neoclr/StructuralCallableLibrary.dll'
    artifact = project / 'bin/neoclr/StructuralCallableConsumer.dll'
    runtime = ROOT / 'target/debug/neoclr'
    codegen = ROOT / 'target/debug/neoclr-codegen'
    commands = []

    def run(label, command, expected=0):
        command = [str(part) for part in command]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
        (output / f'{label}.stdout').write_text(result.stdout)
        (output / f'{label}.stderr').write_text(result.stderr)
        commands.append({'label': label, 'command': command, 'exitCode': result.returncode})
        if result.returncode != expected:
            raise RuntimeError(f'{label}: expected {expected}, got {result.returncode}; see {output}')
        return result.stdout

    run('library', ['dotnet', compiler, 'neoclr', '--project', library / 'Library.rvnproj'])
    run('consumer', ['dotnet', compiler, 'neoclr', '--project', project / 'App.rvnproj'])
    context = ['--system', bundle / 'System.runtime.neox']
    for name in ['System.Runtime', 'System.Data', 'System.Networking', 'System.Web']:
        context += ['--module', bundle / f'{name}.dll']
    context += ['--module', library_artifact, '--object-root', bundle / 'System.Runtime.dll']
    run('interpreter', [runtime, 'run', artifact, '--execution', 'interpreter', *context], 42)
    obj = output / 'consumer.o'
    run('aot', [codegen, '--closed-world', artifact, '@entry', obj,
                '--compile-system', '--reference-arena', '--native-gc', *context])
    imports = run('imports', ['nm', '-u', obj]).split()
    expected = ['_neoclr_allocate_object_rooted_v1', '_neoclr_gc_callback_read_v1',
                '_neoclr_gc_complete_invocation_v1', '_neoclr_gc_enter_v1',
                '_neoclr_gc_entry_check_v1', '_neoclr_gc_leave_v1',
                '_neoclr_gc_stack_roots_v1', '_neoclr_gc_transient_v1']
    assert imports == expected, imports
    host = output / 'host.c'
    host.write_text(r'''#include "native-gc.h"
#include <stdio.h>
int main(void) {
    uint64_t storage[128] = {0};
    neoclr_aot_context ctx = {.text = {(unsigned char*)storage, sizeof(storage), 0}};
    for (int run = 0; run < 32; run++) {
        int32_t result = -1;
        int status = neoclr_entry_v4(0, &result, &ctx);
        if (status || result != 42 || ctx.fault.code || neoclr_root_probe_head_v1()) return 1;
        if (neoclr_gc_collect_v1(&ctx, NULL) || ctx.text.used || neoclr_gc_entry_check_v1(&ctx)) return 2;
    }
    neoclr_gc_statistics stats = neoclr_gc_statistics_v1();
    if (stats.allocations != 64 || stats.reclaimed_allocations != 64 || stats.collections < 32) return 3;
    puts("result=42 iterations=32 allocations=64 reclaimed=64 roots=0 heap=0");
    return 0;
}
''')
    base = ROOT / 'docs/experiments/aot-console'
    sources = [base / name for name in ['native-gc.c', 'root-probe.c', 'text-arena.c']]
    runs = []
    for paced in [False, True]:
        label = 'paced' if paced else 'eager'
        executable = output / label
        run('link-' + label, ['clang', '-arch', 'arm64', '-Wall', '-Wextra', '-Werror',
             '-DNEOCLR_NATIVE_GC', '-fsanitize=undefined,bounds',
             *(['-DNEOCLR_GC_PACED'] if paced else []), '-I', base,
             host, obj, *sources, '-o', executable])
        result = run(label, [executable]).strip()
        assert result == 'result=42 iterations=32 allocations=64 reclaimed=64 roots=0 heap=0', result
        assert not (output / f'{label}.stderr').read_text(), 'Native sanitizer diagnostics'
        runs.append({'paced': paced, 'result': result})
    files = [Path(__file__).resolve(), compiler, runtime, codegen, host, obj,
             library / 'Main.rvn', library / 'Library.rvnproj', library_artifact,
             project / 'Main.rvn', project / 'App.rvnproj', artifact, *sources]
    files += list((ROOT / 'runtime/native/gc').glob('*.[ch]'))
    files += list((ROOT / 'runtime/native/memory').glob('*.[ch]'))
    files += sorted(bundle.glob('*.dll')) + [bundle / 'System.runtime.neox', bundle / 'ownership.json']
    files += sorted(compiler.parent.glob('Raven*.dll')) + sorted(compiler.parent.glob('NeoCLR*.dll'))
    report = {'schemaVersion': 1, 'platform': platform.platform(), 'commands': commands,
              'nativeRuns': runs, 'bundle': str(bundle),
              'checks': ['artifact-only import', 'callable and scalar typed patterns',
                         'bound receiver invocation', 'interpreter result 42',
                         '64 AOT invocations with eager/paced collection and sanitizers',
                         'exact allocations: Cell and callable descriptor only'],
              'limitations': ['retained v3 libraries, fresh compiler/library/consumer',
                              'macOS ARM64 AOT only; JIT lifetime remains unqualified'],
              'files': {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in files}}
    (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'result': 'PASS', 'evidence': str(output / 'results.json')}))


if __name__ == '__main__':
    main()

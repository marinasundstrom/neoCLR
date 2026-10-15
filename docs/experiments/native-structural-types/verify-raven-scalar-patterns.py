"""Build one Raven structural-union consumer and qualify its interpreter/AOT paths."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[3]
PROJECT = Path(__file__).resolve().parent / 'scalar-pattern-project'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    if (platform.system(), platform.machine()) != ('Darwin', 'arm64'):
        parser.error('This AOT qualification requires macOS ARM64')
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    compiler = args.compiler.resolve()
    bundle = ROOT / 'target/structural-integrated-libraries-v3'
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

    run('raven-build', ['dotnet', compiler, 'neoclr', '--project', PROJECT / 'App.rvnproj'])
    artifact = PROJECT / 'bin/neoclr/StructuralScalarPatterns.dll'
    context = ['--system', bundle / 'System.runtime.neox']
    for name in ['System.Runtime', 'System.Data', 'System.Networking', 'System.Web']:
        context += ['--module', bundle / f'{name}.dll']
    context += ['--object-root', bundle / 'System.Runtime.dll']
    run('interpreter', [runtime, 'run', artifact, '--execution', 'interpreter', *context], 42)
    il = output / 'consumer.neoil'
    run('disassemble', [runtime, 'disassemble', artifact, il])
    body = il.read_text()
    assert '"Union":["Int32","Int64","Boolean"]' in body, "Missing native union signature"
    for scalar in ['Int32', 'Int64', 'Boolean']:
        assert body.count(f'isinst "{scalar}"') == 2, scalar
        assert body.count(f'unbox.any "{scalar}"') == 2, scalar
    obj = output / 'consumer.o'
    run('aot', [codegen, '--closed-world', artifact, '@entry', obj,
                '--compile-system', '--reference-arena', *context])
    imports = run('imports', ['nm', '-u', obj])
    assert not imports.strip(), f'Unexpected native imports: {imports}'
    host = output / 'host.c'
    host.write_text('''#include "text-arena.h"
#include <stdio.h>
int main(void) {
    neoclr_aot_context ctx = {0};
    int32_t result = -1;
    int status = neoclr_entry_v4(0, &result, &ctx);
    printf("status=%d result=%d arenaBytes=%llu fault=%u\\n",
           status, result, (unsigned long long)ctx.text.used, ctx.fault.code);
    return status || result != 42 || ctx.text.used || ctx.fault.code;
}
''')
    executable = output / 'consumer'
    run('link', ['clang', '-arch', 'arm64', '-Wall', '-Wextra', '-Werror',
                 '-I', ROOT / 'docs/experiments/aot-console', host, obj, '-o', executable])
    native = run('native', [executable]).strip()
    assert native == 'status=0 result=42 arenaBytes=0 fault=0', native
    files = [compiler, runtime, codegen, PROJECT / 'Main.rvn', PROJECT / 'App.rvnproj', artifact, obj]
    files += sorted(bundle.glob('*.dll')) + [bundle / 'System.runtime.neox', bundle / 'NeoCLR.ClassLibrary.props', bundle / 'ownership.json']
    files += sorted(compiler.parent.glob('Raven*.dll')) + sorted(compiler.parent.glob('NeoCLR*.dll'))
    compiler_revision = subprocess.run(['git', '-C', str(compiler.parent), 'rev-parse', 'HEAD'], capture_output=True, text=True)
    report = {
        'compilerCheckoutRevision': compiler_revision.stdout.strip() if compiler_revision.returncode == 0 else None,
        'runtimeRevision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        'platform': platform.platform(),
        'bundleEvidence': {key: json.loads((bundle / 'build-evidence.json').read_text())[key]
                           for key in ['compilerRevision', 'sourceRevision']},
        'hashes': {str(path): digest(path) for path in files},
        'commands': commands,
        'native': native,
        'checks': ['native union signature preserved', 'six guarded/fallback scalar match inputs return expected values',
                   'two exact tests and extractions per scalar in fresh Raven output',
                   'interpreter exit 42', 'AOT result 42', 'no native imports', 'zero arena use'],
        'limitations': ['one macOS ARM64 consumer', 'retained v3 bundle, not freshly rebuilt libraries',
                        'no mixed payloads, imported union-returning calls, JIT or collector stress',
                        'no throughput claim'],
    }
    (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(native)
    print(f'Evidence: {output / "results.json"}')


if __name__ == '__main__':
    main()

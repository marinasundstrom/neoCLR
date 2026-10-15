"""Build one Raven structural-union consumer and qualify its interpreter/AOT paths."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import re
import subprocess

ROOT = Path(__file__).resolve().parents[3]
PROJECT = Path(__file__).resolve().parent / 'scalar-pattern-project'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--imported', action='store_true', help='Build and consume an artifact-only union library')
    parser.add_argument('--mixed', action='store_true', help='Include a nominal reference alternative')
    parser.add_argument('--generic', action='store_true', help='Use an imported Response<int> with generic payload storage')
    parser.add_argument('--record', action='store_true', help='Use an imported nominal value-record payload')
    parser.add_argument('--record-reference', action='store_true', help='Include a direct HttpError reference beside the value record')
    parser.add_argument('--reference-generic', action='store_true', help='Include Response<Cell> with a shared mutable reference field')
    args = parser.parse_args()
    if args.reference_generic and not (args.record_reference and args.generic):
        parser.error('--reference-generic requires --record-reference --generic')
    if args.record_reference and not args.record:
        parser.error('--record-reference requires --record')
    if args.record and (not (args.mixed and args.imported) or (args.generic and not args.record_reference)):
        parser.error('--record requires --mixed --imported; --generic also requires --record-reference')
    reference_payload = args.mixed and (not args.record or args.record_reference)
    if args.generic and not (args.mixed and args.imported):
        parser.error('--generic requires --mixed --imported')
    project = PROJECT.parent / 'scalar-import-project' if args.imported else PROJECT
    if args.mixed:
        project = PROJECT.parent / ('mixed-import-project' if args.imported else 'mixed-pattern-project')
    library = PROJECT.parent / ('mixed-import-library' if args.mixed else 'scalar-import-library')
    library_name = 'StructuralMixedLibrary' if args.mixed else 'StructuralScalarLibrary'
    if args.generic:
        project = PROJECT.parent / 'generic-mixed-import-project'
        library = PROJECT.parent / 'generic-mixed-import-library'
        library_name = 'StructuralGenericMixedLibrary'
    if args.record:
        project = PROJECT.parent / 'record-import-project'
        library = PROJECT.parent / 'record-import-library'
        library_name = 'StructuralRecordLibrary'
    if args.record_reference:
        project = PROJECT.parent / 'record-reference-import-project'
        library = PROJECT.parent / 'record-reference-import-library'
        library_name = 'StructuralRecordReferenceLibrary'
    if args.record_reference and args.generic:
        project = PROJECT.parent / 'generic-record-reference-import-project'
        library = PROJECT.parent / 'generic-record-reference-import-library'
        library_name = 'StructuralGenericRecordReferenceLibrary'
    if args.reference_generic:
        project = PROJECT.parent / 'reference-generic-record-import-project'
        library = PROJECT.parent / 'reference-generic-record-import-library'
        library_name = 'StructuralReferenceGenericRecordLibrary'
    alternative_count = 7 if args.reference_generic else 6 if args.record_reference and args.generic else 5 if args.record_reference else 4
    input_count = 10 if args.reference_generic else 9 if args.record_reference and args.generic else 8 if args.record_reference else 7 if args.mixed else 6
    arena_bytes = 32 if args.reference_generic else 16 if reference_payload else 0
    library_artifact = library / f'bin/neoclr/{library_name}.dll'
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

    if args.imported:
        run('raven-library', ['dotnet', compiler, 'neoclr', '--project', library / 'Library.rvnproj'])
    run('raven-build', ['dotnet', compiler, 'neoclr', '--project', project / 'App.rvnproj'])
    artifact = project / 'bin/neoclr/StructuralScalarPatterns.dll'
    context = ['--system', bundle / 'System.runtime.neox']
    for name in ['System.Runtime', 'System.Data', 'System.Networking', 'System.Web']:
        context += ['--module', bundle / f'{name}.dll']
    if args.imported:
        context += ['--module', library_artifact]
    context += ['--object-root', bundle / 'System.Runtime.dll']
    run('interpreter', [runtime, 'run', artifact, '--execution', 'interpreter', *context], 42)
    il = output / 'consumer.neoil'
    run('disassemble', [runtime, 'disassemble', artifact, il])
    body = il.read_text()
    if args.mixed:
        unions = [json.JSONDecoder().raw_decode(body[match.end():].lstrip())[0]
                  for match in re.finditer(r'"Union":', body)]
        assert any(len(parts) == alternative_count and all(t in parts for t in ['Int32', 'Int64', 'Boolean'])
                   and any(isinstance(t, dict) and ('Constructed' in t if args.generic else 'Named' in t) for t in parts) for parts in unions), "Missing native mixed union signature"
    else:
        assert '"Union":["Int32","Int64","Boolean"]' in body, "Missing native union signature"
    for scalar in ['Int32', 'Int64', 'Boolean']:
        assert body.count(f'isinst "{scalar}"') == 2, scalar
        assert body.count(f'unbox.any "{scalar}"') == 2, scalar
    if args.imported:
        calls = [json.loads(line.split('call ', 1)[1]) for line in body.splitlines() if ': call ' in line]
        for name in ['Produce', 'Echo']:
            suffix = '.F_' + name.encode().hex().upper()
            assert sum(call['name'].endswith(suffix) for call in calls) == input_count, name
    obj = output / 'consumer.o'
    run('aot', [codegen, '--closed-world', artifact, '@entry', obj,
                '--compile-system', '--reference-arena', *context])
    imports = run('imports', ['nm', '-u', obj])
    expected_imports = ['_neoclr_allocate_object_v1'] if reference_payload else []
    assert imports.split() == expected_imports, f'Unexpected native imports: {imports}'
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
    if reference_payload:
        host.write_text(host.read_text().replace('neoclr_aot_context ctx = {0};',
            'uint64_t storage[32] = {0};\n    neoclr_aot_context ctx = {.text = {(unsigned char*)storage, sizeof(storage), 0}};')
            .replace('|| ctx.text.used ||', f'|| ctx.text.used != {arena_bytes} ||'))
    executable = output / 'consumer'
    run('link', ['clang', '-arch', 'arm64', '-Wall', '-Wextra', '-Werror',
                 '-I', ROOT / 'docs/experiments/aot-console', host, obj, *([ROOT / 'docs/experiments/aot-console/text-arena.c'] if reference_payload else []), '-o', executable])
    native = run('native', [executable]).strip()
    assert native == f'status=0 result=42 arenaBytes={arena_bytes} fault=0', native
    files = [Path(__file__).resolve(), host, compiler, runtime, codegen, project / 'Main.rvn', project / 'App.rvnproj', artifact, obj]
    if reference_payload:
        files += [ROOT / 'docs/experiments/aot-console/text-arena.c']
    if args.imported:
        files += [library / 'Library.rvnproj', library / 'Main.rvn', library_artifact]
    files += sorted(bundle.glob('*.dll')) + [bundle / 'System.runtime.neox', bundle / 'NeoCLR.ClassLibrary.props', bundle / 'ownership.json']
    files += sorted(compiler.parent.glob('Raven*.dll')) + sorted(compiler.parent.glob('NeoCLR*.dll'))
    compiler_revision = subprocess.run(['git', '-C', str(compiler.parent), 'rev-parse', 'HEAD'], capture_output=True, text=True)
    report = {
        'importedLibrary': args.imported,
        'mixedPayloads': reference_payload,
        'nominalValuePayload': args.record,
        'genericPayload': args.generic,
        'recordReferencePayloads': args.record_reference,
        'referenceGenericPayload': args.reference_generic,
        'nativeImports': imports.splitlines(),
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
                   'interpreter exit 42', 'AOT result 42', 'exact native import allowlist', f'arena bytes: {arena_bytes}'],
        'limitations': ['one macOS ARM64 consumer', 'retained v3 bundle, not freshly rebuilt libraries',
                        'no JIT or collector stress',
                        'no throughput claim'],
    }
    if args.reference_generic:
        report['checks'].append('Response<Cell> preserves substituted reference field and shared mutation across record copy')
    if args.record_reference:
        report['checks'].append('eighth imported input preserves direct HttpError reference beside inline Response record')
    if args.generic and args.record_reference:
        report['checks'].append('Response<int> and Response<long> retain distinct closed identity and substituted fields')
    if args.generic:
        report['checks'].append('imported Response<int> preserves its generic field payload')
    if args.mixed:
        report['checks'].append('nominal value-record alternative with wide field and distinct typed match result' if args.record else 'nominal reference alternative through a typed match arm')
    else:
        report['limitations'].append('no mixed payloads')
    if args.imported:
        report['checks'].append('artifact-only library union returns and parameters with reordered alternatives')
    else:
        report['limitations'].append('no imported union-returning calls')
    (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(native)
    print(f'Evidence: {output / "results.json"}')


if __name__ == '__main__':
    main()

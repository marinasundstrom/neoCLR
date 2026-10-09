#!/usr/bin/env python3
"""Compile unchanged production String against native metadata and compare execution."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--raven', required=True, type=Path)
parser.add_argument('--output', required=True, type=Path)
args = parser.parse_args()
output = args.output.resolve()
output.mkdir(parents=True, exist_ok=False)
report = {'scope': 'Production String and 15 production dependencies; native metadata only. Fixture core Object, not full core bootstrap.', 'commands': []}


def run(command, expected=0):
    command = list(map(str, command))
    result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, timeout=240)
    report['commands'].append({'command': command, 'exitCode': result.returncode,
                               'stdout': result.stdout, 'stderr': result.stderr})
    (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    if result.returncode != expected:
        raise RuntimeError(result.stdout + result.stderr)
    return result.stdout + result.stderr


artifacts = output / 'artifacts'
runtime = ROOT / 'target/debug/neoclr'
aot = ROOT / 'tools/aot-poc/target/debug/neoclr-aot-poc'
run(['dotnet', 'run', '--project', HERE / 'Probe.csproj', '-p:RavenRoot=' + str(args.raven.resolve()),
     '-p:NeoClrMetadataProject=' + str(ROOT / 'tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj'),
     '-p:WarningLevel=0', '--', artifacts, '--source-string'])
seed = artifacts / 'System.neox'
run([runtime, 'assemble', HERE / 'System.neoil', seed, '--format', 'neox'])
consumer = artifacts / 'StringConsumer.dll'
deps = ['--module', artifacts / 'NativeCore.dll', '--module', artifacts / 'NativeString.dll',
        '--system', seed, '--object-root', artifacts / 'NativeCore.dll']
assert 'Int32(42)' in run([runtime, 'run', consumer, *deps, '--show-result'], 42)
obj = artifacts / 'string.o'
flags = ['--compile-system', '--reference-arena', '--bind-utf8-text', '--bind-user-fault',
         '--bind-character-text', '--native-gc', '--native-stack-budget']
run([aot, '--closed-world', consumer, '@entry', obj, *deps, *flags])
# Required services and guarded pool lifetime cannot be silently omitted.
for name, rejected_flags in [
    ('missing-text', [flag for flag in flags if flag != '--bind-utf8-text']),
    ('missing-gc', [flag for flag in flags if flag not in ('--native-gc', '--native-stack-budget')]),
]:
    rejected = artifacts / (name + '.o')
    rejected_message = run([aot, '--closed-world', consumer, '@entry', rejected, *deps, *rejected_flags], 1)
    if name == 'missing-gc':
        assert 'String.Intern requires --native-gc' in rejected_message
        inspection = json.loads(run([aot, '--inspect', consumer, '@entry', '--closed-world', *deps, *rejected_flags]))
        assert not inspection['admission']['accepted']
        assert 'String.Intern requires --native-gc' in inspection['admission']['firstError']
    assert not rejected.exists()
kernel = ROOT / 'tools/aot-native-text/target/release/libneoclr_aot_native_text.a'
sources = [ROOT / 'docs/experiments/aot-console' / name for name in
           ['text-arena.c', 'string-unicode.c', 'native-gc.c', 'root-probe.c', 'native-stack.c']]
exe = artifacts / 'string-app'
link = ['clang', '-O2', '-Wall', '-Wextra', '-Werror', '-DNEOCLR_NATIVE_GC',
        HERE / 'string-host.c', ROOT / 'docs/experiments/aot-fault-details/render.c', *sources, obj, kernel]
run([*link, '-o', exe])
run([exe])
sanitized = artifacts / 'string-app-ubsan'
run([*link, '-fsanitize=undefined', '-o', sanitized])
run([sanitized])
for name, expected_fault in [('StringObjectConsumer', 0), ('StringFaultConsumer', 3)]:
    extra = artifacts / (name + '.dll')
    interpreted = run([runtime, 'run', extra, *deps, '--show-result'], 1 if expected_fault else 42)
    extra_obj = artifacts / (name + '.o')
    run([aot, '--closed-world', extra, '@entry', extra_obj, *deps, *flags])
    extra_exe = artifacts / name
    extra_link = [extra_obj if part == obj else part for part in link]
    run([*extra_link, '-DEXPECTED_FAULT=' + str(expected_fault), '-o', extra_exe])
    native = run([extra_exe])
    if expected_fault:
        assert interpreted.strip() == native.strip()
        assert 'RuntimeError' in interpreted and 'at ' in interpreted
    else:
        assert 'Int32(42)' in interpreted and '42;' in native
report['dynamicDependencies'] = run(['otool', '-L', exe])
# Check the ordinary binary; the separate sanitizer build loads compiler diagnostics.
assert all('libSystem.B.dylib' in line for line in report['dynamicDependencies'].splitlines()[1:])
source_names = ["String", "Array", "EquatableTo", "Disposable", "Collections/Iterator",
                "Collections/Iterable", "Collections/Collection", "Collections/Sequence",
                "Collections/MutableSequence", "StringComparison", "Propagatable", "Option",
                "Result", "Attribute", "Runtime/CompilerServices/UnionAttribute", "Text/Utf8SliceError"]
report['sourceHashes'] = {name: hashlib.sha256((ROOT / 'runtime/raven/src/System' / (name + '.rvn')).read_bytes()).hexdigest()
                          for name in source_names}
report['ravenRevision'] = run(['git', '-C', args.raven.resolve(), 'rev-parse', 'HEAD']).strip()
report['neoClrRevision'] = run(['git', 'rev-parse', 'HEAD']).strip()
report['workingTreeChanges'] = run(['git', 'diff', '--stat'])
(output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
print('PASS source String: native-only compilation, interpreter, ARM64, repeated GC entries, required bindings')

"""Verify public attribute inspection with shared interpreter/AOT metadata recipes.

The fixture attaches native annotations after Raven compilation until source emission
is supported. This is deliberately an AOT gate, not compiler annotation evidence.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bundle', type=Path, required=True)
parser.add_argument('--aot', type=Path, required=True)
parser.add_argument('--runner', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
bundle, aot, runner, output = (p.resolve() for p in (args.bundle, args.aot, args.runner, args.output))
output.mkdir(parents=True, exist_ok=False)
commands = []
env = dict(os.environ, NeoClrBundleRoot=str(bundle))


def run(command, expected=0):
    command = list(map(str, command))
    result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, timeout=240)
    commands.append(dict(command=command, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
    (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
    assert result.returncode == expected, result.stdout + result.stderr
    return result


project = output / 'source'
project.mkdir()
for name in ('Main.rvn', 'Native.rvnproj'):
    shutil.copyfile(HERE / name, project / name)
run(['dotnet', bundle / 'sdk/tools/rvnc/rvnc.dll', 'neoclr', '--project', project / 'Native.rvnproj'])
assembly = project / 'bin/neoclr/NativeAttributes.dll'
image, roots = output / 'app.neox', output / 'roots.json'
run(['dotnet', 'run', '--project', ROOT / 'tools/metadata/NeoCLR.Metadata.Experimental.Tests',
     '-p:WarningLevel=0', '--', '--aot-attribute-fixture', assembly, image, roots])
lib = bundle / 'lib'
context = ['--system', lib / 'System.runtime.neox', '--module', lib / 'System.Runtime.dll',
           '--object-root', lib / 'System.Runtime.dll']
expected = 'Native attribute inspection passed\n'
interpreted = run([runner, 'run', image, *context, '--gc-stats'])
assert interpreted.stdout == expected, interpreted.stdout
assert 'live=0' in interpreted.stderr, interpreted.stderr
flags = ['--compile-system', '--bind-user-fault', '--reference-arena', '--native-gc',
         '--bind-console-write-line', '--bind-utf8-text', '--native-stack-budget', '--bind-integer-text']
base = ROOT / 'docs/experiments/aot-console'
windows = platform.system() == 'Windows'
if windows:
    assert platform.machine().lower() in ('amd64', 'x86_64'), 'This gate currently targets Windows x64'
    flags += ['--target', 'x86_64-pc-windows-msvc', '--windows-console-experiment']
    adapters = [ROOT / 'tools/native' / name for name in
                ('windows-console-host.c', 'windows-gc-host.c', 'windows-host-memory.c', 'windows-native-stack.c')]
    adapters += [base / name for name in ('root-probe.c', 'native-gc.c', 'text-arena.c', 'console.c')]
else:
    assert platform.system() == 'Darwin' and platform.machine() == 'arm64', 'This gate currently targets macOS ARM64'
    adapters = [ROOT / 'tools/native/console-host.c', *[base / name for name in
                ('root-probe.c', 'native-gc.c', 'text-arena.c', 'console.c', 'native-stack.c')],
                base.parent / 'aot-scalar/console.c']
adapters += [base.parent / 'aot-fault-details/render.c']


def native(name, policy):
    obj = output / (name + ('.obj' if windows else '.o'))
    selection = json.loads(run([aot, '--closed-world', image, '@entry', obj, *context,
                                *flags, '--reflection-roots', policy]).stdout)
    (output / (name + '-selection.json')).write_text(json.dumps(selection, indent=2) + '\n')
    exe = output / (name + ('.exe' if windows else ''))
    if windows:
        run(['cl', '/nologo', '/W4', '/WX', '/std:c11', '/experimental:c11atomics', '/O2', '/MT',
             '/DNEOCLR_NATIVE_GC', '/I' + str(base), '/Fo' + str(output) + '/', '/Fe:' + str(exe),
             *adapters, obj, '/link', '/STACK:1048576'])
        dependencies = run(['dumpbin', '/dependents', exe]).stdout
        import re
        assert {d.lower() for d in re.findall(r'^\s+([A-Za-z0-9_.-]+\.dll)\s*$', dependencies, re.M)} == {'kernel32.dll'}
    else:
        sdk = run(['xcrun', '--sdk', 'macosx', '--show-sdk-path']).stdout.strip()
        run(['xcrun', 'clang', '-isysroot', sdk, '-arch', 'arm64', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror',
             '-DNEOCLR_NATIVE_GC', '-I', base, *adapters, obj, '-o', exe])
        dependencies = run(['xcrun', 'otool', '-L', exe]).stdout
        assert [line.split()[0] for line in dependencies.splitlines()[1:]] == ['/usr/lib/libSystem.B.dylib']
    return exe, selection


exe, selection = native('attributes', roots)
assert run([exe]).stdout == expected
# Metadata retention must not retain either deliberately faulting user constructor.
source_types = selection['sourceMetadata']['types']
for name in ('NoteAttribute', 'Subject'):
    declaration = next(t for t in source_types if (t.get('origin') or {}).get('name') == name)
    constructor = next(m for m in declaration['declaredMethods'] if m['name'].endswith('..ctor'))
    assert all(f['definition'] != constructor['definition'] for f in selection['functions']), name
policy = json.loads(roots.read_text())
policy['types'][1]['customAttributes'] = False
missing = output / 'unretained.json'
missing.write_text(json.dumps(policy))
exe, _ = native('unretained', missing)
rejected = run([exe], expected=1)
assert 'native custom attribute metadata was not retained' in rejected.stdout + rejected.stderr
report = dict(passed=True, host=platform.platform(), cases=['fixed-and-enum-arguments', 'null-string',
    'repeated-attributes', 'property-method-parameter-targets', 'snapshot-copy', 'constructor-signature',
    'no-user-constructor-roots', 'unannotated-type', 'unretained-type-rejection', 'standalone-dependencies'],
    inputs={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in
            [HERE / 'Main.rvn', HERE / 'Native.rvnproj', Path(__file__), aot, runner, lib / 'System.Runtime.dll',
             bundle / 'sdk/tools/rvnc/Raven.CodeAnalysis.NeoClr.dll', lib / 'System.runtime.neox',
             ROOT / 'tools/metadata/NeoCLR.Metadata.Experimental.Tests/NativeAttributeAotFixture.cs', image, roots]})
(output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
print('PASS native/interpreted attribute inspection and retention checks')

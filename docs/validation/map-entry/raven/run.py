#!/usr/bin/env python3
"""Qualify Raven mutable collection extensions against a native bundle on macOS ARM64."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[4]
SOURCE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--compiler', type=Path, required=True)
parser.add_argument('--bundle', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--case', choices=('success', 'target-fault'), default='success')


args = parser.parse_args()
compiler, bundle, output = (p.resolve() for p in (args.compiler, args.bundle, args.output))
project = output / 'project'
project.mkdir(parents=True, exist_ok=True)
name = 'MapEntryRaven'
negative = args.case == 'target-fault'
expected = 2 if negative else 42
source = SOURCE / ('TargetFault.rvn' if negative else 'Main.rvn')
host = SOURCE.parents[1] / 'interface-variance/native-host.c'
(project / 'Main.rvn').write_text(source.read_text())
(project / 'Primitive.rvn').unlink(missing_ok=True)
from xml.sax.saxutils import escape
(project / 'App.rvnproj').write_text(f'''<Project Sdk="Microsoft.NET.Sdk">
  <Import Project="{escape(str(bundle / 'NeoCLR.ClassLibrary.props'))}"/>
  <PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType>
    <AssemblyName>{name}</AssemblyName></PropertyGroup>
</Project>\n''')
report = {'passed': False, 'commands': []}
def run(label, command, expected=0):
    command = list(map(str, command))
    result = subprocess.run(command, text=True, capture_output=True, timeout=300)
    # Selection manifests are large; retain their hash and raw output locally.
    log = output / (label + '.log')
    log.write_text(result.stdout + result.stderr)
    report['commands'].append(dict(label=label, command=command, exitCode=result.returncode,
        logSha256=hashlib.sha256(log.read_bytes()).hexdigest(),
        output=(result.stdout + result.stderr)[:3000] if label != 'aot' else '(selection manifest in local aot.log)'))
    (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(label, result.returncode, flush=True)
    if result.returncode != expected:
        raise RuntimeError(result.stdout[-3000:] + result.stderr[-3000:])
    return result

run('compile', ['dotnet', compiler, 'neoclr', '--project', project / 'App.rvnproj'])
image = project / 'bin/neoclr' / (name + '.dll')
references = ['--system', bundle / 'System.runtime.neox', '--object-root', bundle / 'System.Runtime.dll']
for library in ('System.Runtime', 'System.Data', 'System.Networking', 'System.Web'):
    references += ['--module', bundle / (library + '.dll')]
interpreter = [ROOT / 'target/debug/neoclr', 'run', image, '--execution', 'interpreter', *references]
interpreted = run('interpreter', interpreter, 1 if negative else 42)
if negative:
    assert 'entry factory failed' in interpreted.stderr
codegen = ROOT / 'target/debug/neoclr-codegen'
obj = output / 'app.o'
obj.unlink(missing_ok=True)
options = [*references, '--compile-system', '--reference-arena', '--bind-user-fault', '--native-gc', '--native-stack-budget', '--bind-utf8-text']
run('aot', [codegen, '--closed-world', image, '@entry', obj, *options])
sdk = subprocess.check_output(['xcrun', '--sdk', 'macosx', '--show-sdk-path'], text=True).strip()
console = ROOT / 'docs/experiments/aot-console'
link = ['clang', '-isysroot', sdk, '-arch', 'arm64', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-I', console, '-DNEOCLR_NATIVE_GC']
support = [console / f for f in ('native-gc.c', 'root-probe.c', 'text-arena.c', 'native-stack.c')]
support += [ROOT / 'docs/experiments/aot-fault-details/render.c']
run('text-kernel', ['cargo', 'build', '--locked', '--release', '--manifest-path', ROOT / 'tools/aot-native-text/Cargo.toml'])
support += [console / 'string-unicode.c', ROOT / 'tools/aot-native-text/target/release/libneoclr_aot_native_text.a']
run('link-aot', [*link, '-fsanitize=undefined,bounds', host, obj, *support, '-o', output / 'app'])
executed = run('execute-aot', [output / 'app'], expected)
if negative:
    assert 'entry factory failed' in executed.stdout + executed.stderr
run('link-jit', [*link, '-DJIT_HOST', '-dynamiclib', host, *support, '-o', output / 'host.dylib'])
result = run('jit', [codegen, '--jit-managed', '--closed-world', image, '@entry', output / 'host.dylib', *options])
report['jit'] = json.loads(result.stderr.split('JIT_RESULT ', 1)[1])
assert report['jit']['guestExitCode'] == expected
if negative:
    assert 'entry factory failed' in result.stdout + result.stderr
report['files'] = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in
    (source, SOURCE / 'run.py', host, compiler, *sorted(compiler.parent.glob('*.dll')), *support, codegen, ROOT / 'target/debug/neoclr', image, bundle / 'bundle.json')}
report['passed'] = True
(output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')

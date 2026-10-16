#!/usr/bin/env python3
"""Qualify Raven instance compound operators against a native bundle on macOS ARM64."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
SOURCE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--compiler', type=Path, required=True)
parser.add_argument('--bundle', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--source', type=Path, help='Alternate focused operator consumer')
parser.add_argument('--value-arrays', action='store_true', help='Qualify interpreter support and the current AOT rejection for nominal value arrays')



args = parser.parse_args()
compiler, bundle, output = (p.resolve() for p in (args.compiler, args.bundle, args.output))
project = output / 'project'
project.mkdir(parents=True, exist_ok=True)
name = 'CompoundOperatorsRaven'
expected = 42
source = args.source.resolve() if args.source else SOURCE / ('ValueArrays.rvn' if args.value_arrays else 'Main.rvn')
host = SOURCE.parent / 'interface-variance/native-host.c'
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
interpreted = run('interpreter', interpreter, 42)
codegen = ROOT / 'target/debug/neoclr-codegen'
obj = output / 'app.o'
obj.unlink(missing_ok=True)
options = [*references, '--compile-system', '--reference-arena', '--bind-user-fault', '--native-gc', '--native-stack-budget']
if args.value_arrays:
    rejected = run('aot-rejection', [codegen, '--closed-world', image, '@entry', obj, *options], 1)
    assert 'unsupported value instruction: NewArray(Named(' in rejected.stderr
    report['limitation'] = 'Nominal value arrays are interpreter-only; AOT rejects allocation explicitly.'
    report['passed'] = True
    report['sourceSha256'] = hashlib.sha256(source.read_bytes()).hexdigest()
    (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    raise SystemExit(0)
run('aot', [codegen, '--closed-world', image, '@entry', obj, *options])
sdk = subprocess.check_output(['xcrun', '--sdk', 'macosx', '--show-sdk-path'], text=True).strip()
console = ROOT / 'docs/experiments/aot-console'
link = ['clang', '-isysroot', sdk, '-arch', 'arm64', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-I', console, '-DNEOCLR_NATIVE_GC']
support = [console / f for f in ('native-gc.c', 'root-probe.c', 'text-arena.c', 'native-stack.c')]
support += [ROOT / 'docs/experiments/aot-fault-details/render.c']
run('link-aot', [*link, '-fsanitize=undefined,bounds', host, obj, *support, '-o', output / 'app'])
executed = run('execute-aot', [output / 'app'], expected)
run('link-jit', [*link, '-DJIT_HOST', '-dynamiclib', host, *support, '-o', output / 'host.dylib'])
result = run('jit', [codegen, '--jit-managed', '--closed-world', image, '@entry', output / 'host.dylib', *options])
report['jit'] = json.loads(result.stderr.split('JIT_RESULT ', 1)[1])
assert report['jit']['guestExitCode'] == expected
report['files'] = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in
    (source, SOURCE / 'run.py', host, compiler, compiler.parent / 'NeoCLR.Metadata.Experimental.dll', compiler.parent / 'Raven.CodeAnalysis.dll', compiler.parent / 'Raven.CodeAnalysis.NeoClr.dll', codegen, ROOT / 'target/debug/neoclr', image, bundle / 'bundle.json')}
report['passed'] = True
(output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')

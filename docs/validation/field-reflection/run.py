#!/usr/bin/env python3
"""Qualify guest field and type-initializer reflection against a native bundle on macOS ARM64."""
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
parser.add_argument('--source', type=Path, help='Alternate focused reflection consumer')
parser.add_argument('--reference', type=Path, action='append', default=[], help='Additional native provider assembly')
parser.add_argument('--overflow', action='store_true', help='Expect checked arithmetic overflow and an untouched result slot on native backends')



args = parser.parse_args()
compiler, bundle, output = (p.resolve() for p in (args.compiler, args.bundle, args.output))
project = output / 'project'
project.mkdir(parents=True, exist_ok=True)
name = 'FieldReflectionRaven'
expected = 2 if args.overflow else 42
source = args.source.resolve() if args.source else SOURCE / 'Main.rvn'
host = SOURCE.parent / 'interface-variance/native-host.c'
(project / 'Main.rvn').write_text(source.read_text())
(project / 'Primitive.rvn').unlink(missing_ok=True)
from xml.sax.saxutils import escape
providers = [p.resolve() for p in args.reference]
provider_items = ''.join(f'<Reference Include="{escape(p.stem)}"><HintPath>{escape(str(p))}</HintPath></Reference>' for p in providers)
(project / 'App.rvnproj').write_text(f'''<Project Sdk="Microsoft.NET.Sdk">
  <Import Project="{escape(str(bundle / 'NeoCLR.ClassLibrary.props'))}"/>
  <PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType>
    <AssemblyName>{name}</AssemblyName></PropertyGroup>
<ItemGroup>{provider_items}</ItemGroup></Project>\n''')
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
    if expected is not None and result.returncode != expected:
        raise RuntimeError(result.stdout[-3000:] + result.stderr[-3000:])
    return result

run('compile', ['dotnet', compiler, 'neoclr', '--project', project / 'App.rvnproj'])
image = project / 'bin/neoclr' / (name + '.dll')
# Resolve the explicit State root from emitted metadata; generated helper types
# may precede source declarations, so source order is not a TypeDef index.
listing = output / 'metadata.neoil'
run('metadata', [ROOT / 'target/debug/neoclr', 'disassemble', image, listing])
metadata, _ = json.JSONDecoder().raw_decode(listing.read_text().split('.module ', 1)[1])
indices = [i for i, t in enumerate(metadata['types']) if t.get('origin', {}).get('name') == 'State']
assert len(indices) == 1
roots = json.loads((SOURCE / 'roots.json').read_text())
roots['types'][0]['definition'] = dict(module=metadata['name'], revision=metadata['revision'], index=indices[0])
roots_path = output / 'roots.json'
roots_path.write_text(json.dumps(roots, indent=2) + '\n')

references = ['--system', bundle / 'System.runtime.neox', '--object-root', bundle / 'System.Runtime.dll']
for library in ('System.Runtime', 'System.Data', 'System.Networking', 'System.Web'):
    references += ['--module', bundle / (library + '.dll')]
for provider in providers:
    references += ['--module', provider]
interpreter = [ROOT / 'target/debug/neoclr', 'run', image, '--execution', 'interpreter', *references]
interpreted = run('interpreter', interpreter, None if args.overflow else 42)
if args.overflow:
    assert interpreted.returncode != 0 and 'ArithmeticOverflow:' in interpreted.stdout + interpreted.stderr
codegen = ROOT / 'target/debug/neoclr-codegen'
obj = output / 'app.o'
obj.unlink(missing_ok=True)
options = ['--reflection-roots', roots_path, *references, '--compile-system', '--reference-arena', '--bind-user-fault', '--native-gc', '--native-stack-budget']
run('aot', [codegen, '--closed-world', image, '@entry', obj, *options])
sdk = subprocess.check_output(['xcrun', '--sdk', 'macosx', '--show-sdk-path'], text=True).strip()
console = ROOT / 'docs/experiments/aot-console'
link = ['clang', '-isysroot', sdk, '-arch', 'arm64', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-I', console, '-DNEOCLR_NATIVE_GC']
support = [console / f for f in ('native-gc.c', 'root-probe.c', 'text-arena.c', 'native-stack.c')]
support += [ROOT / 'docs/experiments/aot-fault-details/render.c']
run('link-aot', [*link, '-fsanitize=undefined,bounds', host, obj, *support, '-o', output / 'app'])
executed = run('execute-aot', [output / 'app'], expected)
if args.overflow:
    assert 'status=2 fault=2 result=-1' in executed.stderr
run('link-jit', [*link, '-DJIT_HOST', '-dynamiclib', host, *support, '-o', output / 'host.dylib'])
result = run('jit', [codegen, '--jit-managed', '--closed-world', image, '@entry', output / 'host.dylib', *options])
report['jit'] = json.loads(result.stderr.split('JIT_RESULT ', 1)[1])
assert report['jit']['guestExitCode'] == expected
if args.overflow:
    assert 'status=2 fault=2 result=-1' in result.stderr
report['files'] = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in
    (*providers, roots_path, source, SOURCE / 'run.py', host, compiler, compiler.parent / 'NeoCLR.Metadata.Experimental.dll', compiler.parent / 'Raven.CodeAnalysis.dll', compiler.parent / 'Raven.CodeAnalysis.NeoClr.dll', codegen, ROOT / 'target/debug/neoclr', image, bundle / 'bundle.json')}
report['passed'] = True
(output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')

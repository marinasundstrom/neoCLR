#!/usr/bin/env python3
"""Qualify Raven constructed generic class bases against a native bundle on macOS ARM64."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[4]
SOURCE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--compiler', type=Path, required=True)
parser.add_argument('--bundle', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--imported', action='store_true', help='Compile the generic owner in a separate native library')

parser.add_argument('--external', action='store_true', help='Derive new source classes from the separately compiled generic bases')
args = parser.parse_args()
if args.external:
    args.imported = True
compiler, bundle, output = (p.resolve() for p in (args.compiler, args.bundle, args.output))
project = output / 'project'
project.mkdir(parents=True, exist_ok=True)
name = 'ConstructedClassBaseRaven'
source = SOURCE / ('External.rvn' if args.external else 'Main.rvn')
host = SOURCE.parents[1] / 'interface-variance/native-host.c'
(project / 'Main.rvn').write_text(('' if args.imported else (SOURCE / 'Library.rvn').read_text()) + (SOURCE / 'Recursive.rvn').read_text() + source.read_text())
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

library_image = None
if args.imported:
    library_project = output / 'library'
    library_project.mkdir(exist_ok=True)
    (library_project / 'Main.rvn').write_text((SOURCE / 'Library.rvn').read_text() + ((SOURCE / 'ExternalRoot.rvn').read_text() if args.external else ''))
    (library_project / 'Library.rvnproj').write_text((project / 'App.rvnproj').read_text()
        .replace('<OutputType>Exe</OutputType>', '<OutputType>Library</OutputType>')
        .replace(name, 'ConstructedBaseLibrary'))
    run('compile-library', ['dotnet', compiler, 'neoclr', '--project', library_project / 'Library.rvnproj'])
    library_image = library_project / 'bin/neoclr/ConstructedBaseLibrary.dll'
    (project / 'App.rvnproj').write_text((project / 'App.rvnproj').read_text().replace('</Project>',
        f'<ItemGroup><Reference Include="ConstructedBaseLibrary"><HintPath>{escape(str(library_image))}</HintPath></Reference></ItemGroup></Project>'))
rejected = output / 'rejected'
rejected.mkdir(exist_ok=True)
shutil.copyfile(project / 'App.rvnproj', rejected / 'App.rvnproj')
rejection_source = SOURCE / ('ExternalRejected.rvn' if args.external else 'Rejected.rvn')
rejection_owners = source.read_text().split('func Main()', 1)[0] if args.external else ('' if args.imported else (SOURCE / 'Library.rvn').read_text())
(rejected / 'Main.rvn').write_text(rejection_owners + rejection_source.read_text())
negative = run('rejected', ['dotnet', compiler, 'neoclr', '--project', rejected / 'App.rvnproj'], 1)
assert negative.stdout.count('error RAV1503:') + negative.stderr.count('error RAV1503:') == 3

run('compile', ['dotnet', compiler, 'neoclr', '--project', project / 'App.rvnproj'])
image = project / 'bin/neoclr' / (name + '.dll')
references = ['--system', bundle / 'System.runtime.neox', '--object-root', bundle / 'System.Runtime.dll']
for library in ('System.Runtime', 'System.Data', 'System.Networking', 'System.Web'):
    references += ['--module', bundle / (library + '.dll')]
if library_image is not None:
    references += ['--module', library_image]
interpreter = [ROOT / 'target/debug/neoclr', 'run', image, '--execution', 'interpreter', *references]
run('interpreter', interpreter, 42)
codegen = ROOT / 'target/debug/neoclr-codegen'
obj = output / 'app.o'
obj.unlink(missing_ok=True)
options = [*references, '--compile-system', '--reference-arena', '--bind-user-fault', '--native-gc', '--native-stack-budget']
run('aot', [codegen, '--closed-world', image, '@entry', obj, *options])
sdk = subprocess.check_output(['xcrun', '--sdk', 'macosx', '--show-sdk-path'], text=True).strip()
console = ROOT / 'docs/experiments/aot-console'
link = ['clang', '-isysroot', sdk, '-arch', 'arm64', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-I', console, '-DNEOCLR_NATIVE_GC']
support = [console / f for f in ('native-gc.c', 'root-probe.c', 'text-arena.c', 'native-stack.c')]
support += [ROOT / 'docs/experiments/aot-fault-details/render.c']
run('link-aot', [*link, '-fsanitize=undefined,bounds', host, obj, *support, '-o', output / 'app'])
run('execute-aot', [output / 'app'], 42)
run('link-jit', [*link, '-DJIT_HOST', '-dynamiclib', host, *support, '-o', output / 'host.dylib'])
result = run('jit', [codegen, '--jit-managed', '--closed-world', image, '@entry', output / 'host.dylib', *options])
report['jit'] = json.loads(result.stderr.split('JIT_RESULT ', 1)[1])
assert report['jit']['guestExitCode'] == 42
report['files'] = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in
    (source, SOURCE / 'Library.rvn', rejection_source, SOURCE / 'Recursive.rvn', SOURCE / 'run.py', host, compiler, compiler.parent / 'NeoCLR.Metadata.Experimental.dll', compiler.parent / 'Raven.CodeAnalysis.dll', compiler.parent / 'Raven.CodeAnalysis.NeoClr.dll', codegen, ROOT / 'target/debug/neoclr', image, bundle / 'bundle.json')}
if args.external:
    report['files'][str(SOURCE / 'ExternalRoot.rvn')] = hashlib.sha256((SOURCE / 'ExternalRoot.rvn').read_bytes()).hexdigest()
if library_image is not None:
    report['files'][str(library_image)] = hashlib.sha256(library_image.read_bytes()).hexdigest()
report['passed'] = True
(output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')

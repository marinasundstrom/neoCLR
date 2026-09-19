"""Copy a built local Raven/neoCLR toolchain into a fresh VS Code workspace.

This is a development snapshot, not a release package. No installed SDK is changed.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
from collection_library import ROOT, build

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('output', type=Path)
parser.add_argument('--raven', required=True, type=Path)
parser.add_argument('--runtime', type=Path, default=ROOT / 'target/debug/neoclr')
args = parser.parse_args()
output, raven = args.output.resolve(), args.raven.resolve()
source = Path(__file__).resolve().parent
inputs = {
    'server': raven / 'src/Raven.LanguageServer/bin/Debug/net11.0',
    'rvnc': raven / 'src/Raven.Compiler/bin/Debug/net11.0',
    'bridge': source / 'bin/Debug/net11.0',
}
for name, filename in [('server', 'Raven.LanguageServer.dll'), ('rvnc', 'rvnc.dll'), ('bridge', 'Probe.dll')]:
    if not (inputs[name] / filename).is_file():
        parser.error(f'Build {inputs[name] / filename} first')
if not args.runtime.is_file():
    parser.error('Build the neoCLR runtime first')
output.mkdir(parents=True, exist_ok=False)
for name, directory in inputs.items():
    shutil.copytree(directory, output / 'tools' / name)
for directory in ['bin', 'lib', 'build', 'demo/.vscode', 'licenses']:
    (output / directory).mkdir(parents=True, exist_ok=True)
shutil.copy2(args.runtime, output / 'bin/neoclr')
shutil.copy2(ROOT / 'build/NeoCLR.Raven.props', output / 'build/NeoCLR.Raven.props')
for filename in ['run_project.py', 'runner_options.py']:
    shutil.copy2(source / filename, output / 'tools' / filename)
for repo, label in [(ROOT, 'neoCLR'), (raven, 'Raven')]:
    for license_file in [*repo.glob('LICENSE*'), *repo.glob('THIRD_PARTY_NOTICES*')]:
        if license_file.is_file():
            shutil.copy2(license_file, output / 'licenses' / (label + '-' + license_file.name))
(output / 'lib/System.neoil').write_text(build(ROOT / 'runtime/System.neoil'))
subprocess.run(['dotnet', str(output / 'tools/bridge/Probe.dll'), '--reference-core',
                str(output / 'demo/NeoCLR.CoreProbe.dll')], check=True)
shutil.copy2(source / 'samples/library-workflow.rvn', output / 'demo/Main.rvn')
(output / 'demo/examples').mkdir()
for name in ['flags', 'reflection', 'introspection-interfaces', 'type-acquisition', 'assembly-info', 'array-shapes', 'array-callbacks']:
    shutil.copy2(source / f'samples/library-{name}.rvn', output / f'demo/examples/{name}.rvn')
(output / 'demo/Demo.rvnproj').write_text('''<Project>
  <PropertyGroup><NeoCLRRoot>$(MSBuildThisFileDirectory)..</NeoCLRRoot></PropertyGroup>
  <Import Project="../build/NeoCLR.Raven.props" />
  <ItemGroup><Compile Include="Main.rvn" /></ItemGroup>
</Project>
''')
settings = {'raven.languageServerPath': str(output / 'tools/server/Raven.LanguageServer.dll'),
            'raven.sdkPath': str(output), 'files.associations': {'*.rvn': 'raven'}}
(output / 'demo/.vscode/settings.json').write_text(json.dumps(settings, indent=2) + '\n')
arguments = [str(output / 'tools/run_project.py'), '${workspaceFolder}/Demo.rvnproj',
             '--runtime', str(output / 'bin/neoclr'), '--bridge', str(output / 'tools/bridge/Probe.dll'),
             '--system', str(output / 'lib/System.neoil')]
tasks = {'version': '2.0.0', 'tasks': [{'label': 'neoCLR: Run saved project', 'type': 'process',
         'command': 'python3', 'args': arguments, 'options': {'cwd': '${workspaceFolder}'},
         'group': {'kind': 'build', 'isDefault': True}, 'problemMatcher': []}]}
(output / 'demo/.vscode/tasks.json').write_text(json.dumps(tasks, indent=2) + '\n')
(output / 'demo/.vscode/extensions.json').write_text(json.dumps({'recommendations': ['raven.raven-vscode']}, indent=2) + '\n')
(output / 'VERSION').write_text('local-development-20260919\n')
(output / 'demo/README.md').write_text('''# Try the Raven library port

Open Main.rvn, edit and save it, then choose **Terminal → Run Task → neoCLR: Run saved project**
(or the default build task). The sample processes three products using ArrayList,
Option and Result. Its output is:

```text
42
Completed
Price overflow
Skipped
Product not found
Skipped
```

The task compiles, imports, verifies and runs your saved source on neoCLR. The
Raven extension's ordinary Run/Debug commands target .NET; use this task for neoCLR.
Completion and hover use this folder's matching language server and reference core.
The examples folder contains flags, typeof/reflection, sealed introspection matching
and array examples. To try
one, copy its contents into Main.rvn and save; only Main.rvn is compiled by the task.

This is a local development snapshot. Python 3 and .NET 11 are required; the copied
runtime executable is built for this machine. Runtime services remain intrinsic.
The eight Info contracts are sealed interfaces. Both typeof(T) and Object.GetType()
return TypeInfo directly; System.Type and the .Info hop are no longer public APIs.
RuntimeContext.Current provides the configured handle resolver and ExecutingAssembly.
AssemblyInfo exposes ReferencedAssemblies, GetModules() and GetTypes() through
Sequence interfaces. MetadataToken is available on the Info contracts, scoped by
Module for type/member/parameter definitions. Discovery covers retained loaded
metadata, including System.Runtime; it does not load assemblies. The former TypeOf<T>.Of helper has
been removed. See examples/type-acquisition.rvn for a runnable acquisition sample, and examples/assembly-info.rvn for discovery.
''')
manifest = {'kind': 'local-development', 'repositories': {}, 'sha256': {}}
for repo, label in [(ROOT, 'neoCLR'), (raven, 'Raven')]:
    manifest['repositories'][label] = subprocess.check_output(['git', '-C', str(repo), 'rev-parse', 'HEAD'], text=True).strip()
for relative in ['bin/neoclr', 'lib/System.neoil', 'demo/NeoCLR.CoreProbe.dll',
                 'tools/server/Raven.LanguageServer.dll', 'tools/server/Raven.CodeAnalysis.dll',
                 'tools/rvnc/rvnc.dll', 'tools/bridge/Probe.dll']:
    manifest['sha256'][relative] = hashlib.sha256((output / relative).read_bytes()).hexdigest()
(output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
print(output / 'demo')

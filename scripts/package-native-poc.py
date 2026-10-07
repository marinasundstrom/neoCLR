#!/usr/bin/env python3
"""Stage and archive explicit native POC tooling/dependencies; does not publish a release."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import shutil
import subprocess
import tarfile
import xml.etree.ElementTree as X

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('sdk', 'vsix', 'libraries', 'runtime', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--core', type=Path, help='Legacy aggregate bundle primitive core')
    parser.add_argument('--seed', type=Path, help='Legacy aggregate bundle runtime seed')
    parser.add_argument('--runtime-revision', required=True, help='Declared source revision of supplied runtime binary')
    args = parser.parse_args()
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    split = (args.libraries / 'bundle.json').is_file()
    if split:
        if args.core or args.seed:
            raise ValueError('Split bundle owns its core and seed; omit --core and --seed')
        catalog = json.loads((args.libraries / 'bundle.json').read_text())
        if catalog['assemblyNames'] != ['System.Runtime', 'System.Data', 'System.Networking', 'System.Web']:
            raise ValueError('Unsupported split class-library selection')
        for relative, expected in catalog['files'].items():
            path = (args.libraries / relative).resolve()
            if args.libraries.resolve() not in path.parents or sha(path) != expected:
                raise ValueError('Class-library bundle mismatch: ' + relative)
        build = json.loads((args.libraries / 'build-evidence.json').read_text())
        if not build['commands'] or any(command['exitCode'] for command in build['commands']):
            raise ValueError('Library build did not pass')
        original_bundle = Path(build['environment']['RavenNeoClrRuntimeSeed']).parent
        built_files = {Path(path).relative_to(original_bundle).as_posix(): digest
                       for path, digest in build['hashes'].items()
                       if original_bundle in Path(path).parents}
        if catalog['files'] != built_files:
            raise ValueError('Bundle file catalog differs from build evidence')
        required = [name + '.dll' for name in catalog['assemblyNames']] + [
            'Core.dll', 'ownership.json', catalog['runtimeSeed'], catalog['projectConfiguration']]
        if any(name not in catalog['files'] for name in required):
            raise ValueError('Bundle is missing required dependency hashes')
        producer = Path(build['commands'][0]['command'][1]).parent
        for name in ('rvnc.dll', 'Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll'):
            if sha(args.sdk / 'tools/rvnc' / name) != build['hashes'][str((producer / name).resolve())]:
                raise ValueError('SDK compiler differs from library producer: ' + name)
        for source, expected in build['hashes'].items():
            path = Path(source).resolve()
            if ROOT in path.parents and sha(path) != expected:
                raise ValueError('Source differs from library build evidence: ' + source)
        library_names = catalog['assemblyNames']
        seed_name = catalog['runtimeSeed']
    else:
        if not args.core or not args.seed:
            raise ValueError('Legacy aggregate libraries require --core and --seed')
        build = json.loads((args.libraries / 'build.json').read_text())
        if not build['passed']:
            raise ValueError('Library build did not pass')
        for entry in build['builds']:
            if sha(args.libraries / (entry['name'] + '.dll')) != entry['sha256']:
                raise ValueError('Library differs from build evidence: ' + entry['name'])
        for relative, expected in build['sources'].items():
            source = (ROOT / relative).resolve()
            if ROOT not in source.parents or sha(source) != expected:
                raise ValueError('Source differs from library build evidence: ' + relative)
        first = build['builds'][0]['command']
        for flag, supplied in [('--core-reference', args.core), ('--runtime-seed', args.seed)]:
            if sha(supplied) != build['inputs'][first[first.index(flag) + 1]]:
                raise ValueError('Bootstrap dependency differs from build evidence: ' + flag)
        for name in ('rvnc.dll', 'Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll'):
            original = str(Path(first[1]).parent / name)
            if sha(args.sdk / 'tools/rvnc' / name) != build['inputs'][original]:
                raise ValueError('SDK compiler differs from library producer: ' + name)
        if sha(args.libraries / 'ownership.json') != sha(ROOT / 'runtime/raven/native/poc-ownership.json'):
            raise ValueError('Unexpected ownership selection')
        library_names = ['Numbers', 'Http']
        seed_name = 'System.neox'
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    staged = output / 'neoclr-native-poc'
    (staged / 'lib').mkdir(parents=True)
    (staged / 'bin').mkdir()
    (staged / 'editor').mkdir()
    (staged / 'tools').mkdir()
    if not split:
        shutil.copy2(ROOT / 'runtime/raven/native/poc-seed.neoil', staged / 'lib/System.neoil')
    shutil.copytree(args.sdk, staged / 'sdk')
    executable = 'neoclr.exe' if platform.system() == 'Windows' else 'neoclr'
    shutil.copy2(args.runtime, staged / 'bin' / executable)
    shutil.copy2(args.vsix, staged / 'editor/raven-vscode.vsix')
    if split:
        for relative in catalog['files']:
            destination = staged / 'lib' / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(args.libraries / relative, destination)
        shutil.copy2(args.libraries / 'bundle.json', staged / 'lib/bundle.json')
    else:
        for source, name in [(args.core, 'Core.dll'), (args.seed, 'System.neox'),
                             (args.libraries / 'ownership.json', 'ownership.json')]:
            shutil.copy2(source, staged / 'lib' / name)
        for name in ('Numbers', 'Http'):
            shutil.copy2(args.libraries / (name + '.dll'), staged / 'lib' / (name + '.dll'))
            shutil.copy2(ROOT / 'api-docs/NeoCLR.CoreProbe.xml', staged / 'lib' / (name + '.xml'))
    for name in ('verify-native-bundle.py', 'verify-native-http-json.py'):
        shutil.copy2(ROOT / 'scripts' / name, staged / 'tools' / name)
    shutil.copytree(ROOT / 'third-party', staged / 'third-party')
    for name in ('LICENSE', 'THIRD_PARTY_NOTICES.md'):
        shutil.copy2(ROOT / name, staged / name)
    samples = {
        'collections': (['raven-target/samples/application-order-collections.rvn'],
                        (ROOT / 'docs/experiments/raven-target/samples/application-order-collections.expected.txt').read_text()),
        'tasks': (['raven-target/samples/library-async.rvn'], 'Suspended\n42\n'),
        'json': (['json-object-mapping/Mapping.rvn', 'json-object-mapping/Main.rvn'], 'JSON object mapping checks passed\n'),
        'http-json-server': (['json-object-mapping/HttpApplication.rvn', 'json-object-mapping/HttpServer.rvn'], None),
        'http-json-client': (['json-object-mapping/HttpApplication.rvn', 'json-object-mapping/HttpClient.rvn'], None),
    }
    for name, (sources, expected) in samples.items():
        folder = staged / 'samples' / name
        folder.mkdir(parents=True)
        for source in sources:
            shutil.copy2(ROOT / 'docs/experiments' / source, folder / Path(source).name)
        project = X.Element('Project', Sdk='Microsoft.NET.Sdk')
        if split:
            X.SubElement(project, 'Import', Project='../../lib/' + catalog['projectConfiguration'])
            group = X.SubElement(project, 'PropertyGroup')
            for key, value in dict(TargetFramework='net10.0', OutputType='Exe', AssemblyName='App').items():
                X.SubElement(group, key).text = value
        else:
            group = X.SubElement(project, 'PropertyGroup')
            for key, value in dict(TargetFramework='net10.0', OutputType='Exe', AssemblyName='App',
                RavenTargetPlatform='NeoCLR', RavenMetadataFormat='NeoCLR',
                RavenNeoClrCoreReference='../../lib/Core.dll', RavenNeoClrRuntimeSeed='../../lib/System.neox',
                RavenNeoClrBootstrapOwnership='../../lib/ownership.json', RavenNeoClrAsyncLibrary='Numbers',
                RavenNeoClrBootstrapIntrinsics='true').items():
                X.SubElement(group, key).text = value
            items = X.SubElement(project, 'ItemGroup')
            for library in ('Numbers', 'Http'):
                X.SubElement(X.SubElement(items, 'Reference', Include=library), 'HintPath').text = '../../lib/' + library + '.dll'
        X.indent(project)
        X.ElementTree(project).write(folder / 'App.rvnproj', encoding='unicode')
        settings = folder / '.vscode'
        settings.mkdir()
        (settings / 'settings.json').write_text(json.dumps({'raven.sdkPath': '../../sdk'}, indent=2) + '\n')
        (settings / 'tasks.json').write_text(json.dumps({'version': '2.0.0', 'tasks': [dict(
            label='neoCLR: Run', type='process', command='dotnet', args=[
                '${workspaceFolder}/../../sdk/tools/rvnc/rvnc.dll', 'neoclr', '--project', '${workspaceFolder}/App.rvnproj',
                '--run', '${workspaceFolder}/../../bin/' + executable], problemMatcher=[])]}, indent=2) + '\n')
    (staged / 'README.md').write_text('''# Native neoCLR POC candidate

This is a development native POC candidate; it is not a published release. The host needs the
.NET SDK/runtime required by the included Raven SDK (net11 compiler, net10 project
reference packs), Python 3 and VS Code for editor use. This candidate is host-specific.

Run from this extracted directory:

```
python3 tools/verify-native-bundle.py --report ../acceptance.json
```

Open an individual samples folder in VS Code after installing editor/raven-vscode.vsix.
Use the `neoCLR: Run` task. Project references and task paths are relative to this
bundle. The compiler and runtime execute native metadata libraries; their source
files are deliberately absent from these consumer projects.

The CLI primitive core and retained runtime seed in lib/ are explicit temporary
bootstrap dependencies, not a complete native System bootstrap. The manifest identifies the shipped assemblies and their runtime seed. Documentation
sidecars do not imply full API-help coverage. No library translation bridge is run.
The original library build evidence and declared revisions are in provenance.json.
''')
    (staged / 'provenance.json').write_text(json.dumps(dict(libraryBuild=build,
        declaredRuntimeRevision=args.runtime_revision, runtimeSha256=sha(args.runtime),
        packagerSha256=sha(Path(__file__)),
        sdkVersion=(args.sdk / 'VERSION').read_text().strip(), vsixSha256=sha(args.vsix)), indent=2) + '\n')
    manifest = dict(format='neoclr-native-poc-v1', sourceRevision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        sourceDirty=bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT, text=True).strip()),
        platform=platform.platform(), runtime='bin/' + executable,
        objectRoot=('lib/' + catalog['objectAssembly'] + '.dll') if split else None,
        runtimeSeed='lib/' + seed_name, modules=['lib/' + name + '.dll' for name in library_names],
        samples={name: dict(expected=expected) for name, (_, expected) in samples.items()},
        files={p.relative_to(staged).as_posix(): sha(p) for p in sorted(staged.rglob('*')) if p.is_file()})
    (staged / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    archive = output / 'neoclr-native-poc.tar.gz'
    with tarfile.open(archive, 'w:gz') as tar:
        tar.add(staged, arcname=staged.name)
    print(archive)


if __name__ == '__main__':
    main()

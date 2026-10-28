#!/usr/bin/env python3
"""Rebuild System.Web against a verified native bundle, preserving its other libraries."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--compiler-directory', type=Path, help='Pinned native compiler directory containing rvnc.dll')
    parser.add_argument('--compiler-revision', help='Revision label for the pinned compiler; exact binary hashes are also retained')
    parser.add_argument('--libraries', type=Path, help='Optional separately verified native library directory')
    args = parser.parse_args()
    if bool(args.compiler_directory) != bool(args.compiler_revision):
        raise ValueError('--compiler-directory and --compiler-revision must be supplied together')
    if args.compiler_directory and not (args.compiler_directory / 'rvnc.dll').is_file():
        raise ValueError('Pinned compiler directory has no rvnc.dll')
    original = args.bundle.resolve()
    manifest = json.loads((original / 'manifest.json').read_text())
    for name, digest in manifest['files'].items():
        path = (original / name).resolve()
        if original not in path.parents or sha(path) != digest:
            raise ValueError('Bundle hash mismatch: ' + name)
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    bundle = out / 'bundle'
    shutil.copytree(original, bundle)
    if args.compiler_directory:
        shutil.rmtree(bundle / 'sdk/tools/rvnc')
        shutil.copytree(args.compiler_directory, bundle / 'sdk/tools/rvnc')
        manifest['developmentCompilerRevision'] = args.compiler_revision
    library_manifest = None
    if args.libraries:
        libraries = args.libraries.resolve()
        library_manifest = json.loads((libraries / 'bundle.json').read_text())
        for name, digest in library_manifest['files'].items():
            path = (libraries / name).resolve()
            if libraries not in path.parents or sha(path) != digest:
                raise ValueError('Library hash mismatch: ' + name)
        shutil.rmtree(bundle / 'lib')
        shutil.copytree(libraries, bundle / 'lib')
        manifest['modules'] = ['lib/' + name + '.dll' for name in library_manifest['assemblyNames']]
    props = ET.parse(bundle / 'lib/NeoCLR.ClassLibrary.props')
    native_core = props.getroot().find('.//RavenNeoClrNativeCoreReference') is not None
    if native_core:
        # Match NativeLibrary.props rather than importing application-local await
        # providers into System.Web itself.
        for group in props.getroot():
            for item in list(group):
                if item.tag in ('RavenNeoClrRuntimeAsyncHelper', 'RavenNeoClrRuntimeAsyncProviderImported', 'Compile'):
                    group.remove(item)
    for group in props.getroot().findall('ItemGroup'):
        for reference in list(group):
            if reference.attrib.get('Include') == 'System.Web':
                group.remove(reference)
    props.write(bundle / 'lib/BuildWeb.props')
    project = ET.Element('Project')
    ET.SubElement(project, 'Import', Project=str(bundle / 'lib/BuildWeb.props'))
    properties = ET.SubElement(project, 'PropertyGroup')
    settings = dict(TargetFramework='net10.0', OutputType='Library', AssemblyName='System.Web',
                    EnableDefaultCompileItems='false')
    if not native_core:
        settings['RavenNeoClrBootstrapIntrinsics'] = 'true'
    for key, value in settings.items():
        ET.SubElement(properties, key).text = value
    items = ET.SubElement(project, 'ItemGroup')
    ET.SubElement(items, 'Compile', Include=str(ROOT / 'runtime/System.Web/src/System.Web/System/Web/**/*.rvn'))
    project_path = out / 'Web.rvnproj'
    ET.ElementTree(project).write(project_path, encoding='unicode')
    command = ['dotnet', str(bundle / 'sdk/tools/rvnc/rvnc.dll'), 'neoclr', '--project', str(project_path)]
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=600)
    evidence = dict(originalManifestSha256=sha(original / 'manifest.json'),
                    compilerRevision=manifest.get('developmentCompilerRevision'),
                    originalLibraryManifestSha256=sha(args.libraries / 'bundle.json') if args.libraries else None,
                    compilerSha256=sha(bundle / 'sdk/tools/rvnc/rvnc.dll'), command=command,
                    exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr,
                    sources={str(p.relative_to(ROOT)): sha(p) for p in (ROOT / 'runtime/System.Web/src/System.Web/System/Web').rglob('*.rvn')})
    (out / 'build-evidence.json').write_text(json.dumps(evidence, indent=2) + '\n')
    if result.returncode:
        raise RuntimeError('System.Web build failed; see build-evidence.json')
    for suffix in ('.dll', '.xml'):
        shutil.copyfile(out / ('bin/neoclr/System.Web' + suffix), bundle / ('lib/System.Web' + suffix))
    shutil.rmtree(bundle / 'lib/System.Web.docs')
    shutil.copytree(out / 'bin/neoclr/System.Web.docs', bundle / 'lib/System.Web.docs')
    shutil.copyfile(out / 'build-evidence.json', bundle / 'lib/http-foundation-build.json')
    (bundle / 'lib/BuildWeb.props').unlink()
    for root, filename in ((bundle / 'lib', 'bundle.json'), (bundle, 'manifest.json')):
        path = root / filename
        updated = json.loads(path.read_text())
        if root == bundle and args.compiler_directory:
            updated['developmentCompilerRevision'] = args.compiler_revision
        updated['httpFoundationBuild'] = 'http-foundation-build.json' if root.name == 'lib' else 'lib/http-foundation-build.json'
        updated['files'] = {str(p.relative_to(root)): sha(p) for p in sorted(root.rglob('*')) if p.is_file() and p != path}
        path.write_text(json.dumps(updated, indent=2) + '\n')
    print(bundle)


if __name__ == '__main__':
    main()

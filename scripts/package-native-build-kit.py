#!/usr/bin/env python3
"""Build and archive a relocatable development native console kit; does not publish."""
import argparse
import importlib.util
import json
from pathlib import Path
import platform
import shutil
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('native_build', ROOT / 'scripts/build-native-project.py')
builder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(builder)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if (platform.system(), platform.machine()) != ('Darwin', 'arm64'):
        parser.error('This kit requires macOS ARM64')
    bundle, output = args.bundle.resolve(), args.output.resolve()
    manifest = json.loads((bundle / 'manifest.json').read_text())
    if manifest.get('format') != 'neoclr-native-poc-v1':
        parser.error('Expected a native POC toolchain bundle')
    builder.verify_files(bundle, manifest['files'])
    output.mkdir(parents=True, exist_ok=False)
    cargo_manifest = ROOT / 'tools/aot-poc/Cargo.toml'
    metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--locked', '--offline',
        '--format-version', '1', '--manifest-path', str(cargo_manifest)], cwd=ROOT, text=True))
    sources = [*ROOT.joinpath('src').rglob('*.rs'), *ROOT.joinpath('tools/aot-poc/src').rglob('*.rs'),
               ROOT / 'docs/experiments/http-server/Server.rvn',
               ROOT / 'Cargo.toml', ROOT / 'Cargo.lock', cargo_manifest, cargo_manifest.parent / 'Cargo.lock',
               Path(__file__).resolve(), *builder.support_files()]
    inputs = {str(p.relative_to(ROOT)): builder.sha(p) for p in sources}
    command = ['cargo', 'build', '--locked', '--manifest-path', str(cargo_manifest), '--message-format=json']
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, check=True)
    artifacts = [json.loads(line) for line in result.stdout.splitlines() if line.startswith('{')]
    binaries = [Path(a['executable']) for a in artifacts if a.get('reason') == 'compiler-artifact'
                and a.get('target', {}).get('name') == 'neoclr-aot-poc' and a.get('executable')]
    if len(binaries) != 1:
        raise ValueError('Cargo did not identify one AOT executable')
    aot = binaries[0]
    if subprocess.check_output(['xcrun', 'lipo', '-archs', aot], text=True).strip() != 'arm64':
        raise ValueError('AOT compiler must be a macOS ARM64 executable')
    dependencies = [line.split()[0] for line in subprocess.check_output(
        ['xcrun', 'otool', '-L', aot], text=True).splitlines()[1:]]
    if any(not name.startswith(('/usr/lib/', '/System/Library/')) for name in dependencies):
        raise ValueError('AOT compiler has non-system dependencies: ' + repr(dependencies))
    staged = output / 'neoclr-native-build-kit'
    staged.mkdir()

    def copy(source, relative):
        destination = staged / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)

    # Copy exactly the catalogued bundle, never generated consumer output or caches.
    for relative in manifest['files']:
        copy(bundle / relative, 'bundle/' + relative)
    copy(bundle / 'manifest.json', 'bundle/manifest.json')
    builder.verify_files(staged / 'bundle', manifest['files'])
    copy(aot, 'bin/neoclr-aot-poc')
    for source in builder.support_files():
        copy(source, source.relative_to(ROOT))
    for source in (ROOT / 'tools/native/samples/hello').iterdir():
        copy(source, 'samples/hello/' + source.name)
    copy(ROOT / 'tools/native/samples/hello/App.rvnproj', 'samples/http/App.rvnproj')
    copy(ROOT / 'docs/experiments/http-server/Server.rvn', 'samples/http/Main.rvn')
    copy(ROOT / 'LICENSE', 'LICENSE')
    copy(cargo_manifest.parent / 'Cargo.lock', 'aot-Cargo.lock')
    licenses = []
    for package in metadata['packages']:
        if not package['source']:
            continue
        directory = Path(package['manifest_path']).parent
        name = package['name'] + '-' + package['version']
        texts = [p for p in directory.iterdir() if p.is_file() and
                 p.name.lower().startswith(('license', 'copying', 'copyright', 'notice'))]
        preserved = ROOT / 'third-party/licenses' / name
        if preserved.is_dir():
            texts += [p for p in preserved.rglob('*') if p.is_file()]
        if not texts and package['license'] == 'Apache-2.0 WITH LLVM-exception':
            texts = [ROOT / 'tools/aot-poc/CRANELIFT-LICENSE']
        if not texts:
            raise ValueError('Missing preserved license text: ' + name)
        paths = []
        for index, source in enumerate(sorted(set(texts))):
            relative = f'licenses/{name}/{index}-{source.name}'
            copy(source, relative)
            paths.append(relative)
        licenses.append(dict(name=package['name'], version=package['version'],
                             license=package['license'], texts=paths))
    (staged / 'dependency-licenses.json').write_text(json.dumps(licenses, indent=2) + '\n')
    (staged / 'README.txt').write_text('''neoCLR development native console build kit (macOS ARM64)

From this extracted directory:
  python3 scripts/build-native-project.py --project samples/hello/App.rvnproj --output ../hello-native
  ../hello-native/app

For the bounded one-request HTTP sample:
  python3 scripts/build-native-project.py --profile http --project samples/http/App.rvnproj --output ../http-native
  ../http-native/app
The server prints an ephemeral loopback port; request GET /greeting within 15 seconds.

Requires Python 3, the bundled Raven compiler's .NET SDKs (net11 compiler host,
net10 project reference packs), and Apple Clang/macOS SDK selected through xcrun.
Building an application requires neither Cargo nor a neoCLR source checkout.
The resulting app needs only macOS libSystem, not .NET or this kit.

Experimental synchronous console profile: bounded 1 MiB nonmoving GC, UTF-8 text,
Int32 formatting and fault diagnostics. The opt-in HTTP profile adds the existing
private socket/task pump and guarded native stack, with a 15-second host deadline.
It is a bounded correctness host, not a production server or runtime Scheduler.
No guest command-line arguments, general reflection or Windows native compilation. Import the
kit's bundle/lib/NeoCLR.ClassLibrary.props in your executable Raven project.
The kit checks recorded hashes before building. Hashes establish consistency, not
an authenticated signature or a complete reproducible MSBuild input inventory.
Treat projects/build inputs as trusted code. Existing output directories are rejected.

This is a local development candidate, not a published release. The bundled
interpreter/editor keep their original qualification scope. See native-build-kit.json
for source/input provenance, aot-Cargo.lock and dependency-licenses.json for AOT
compiler dependencies and preserved texts. Original code is MIT; bundled toolchain
notices remain under bundle/. The runtime ABI and service profile are provisional.
''')
    if any(builder.sha(ROOT / path) != digest for path, digest in inputs.items()):
        raise ValueError('Sources changed while building the kit')
    kit = dict(format='neoclr-native-build-kit-v1', profile=builder.PROFILE, profiles=list(builder.PROFILES.values()),
        revision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        dirty=bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT, text=True).strip()),
        inputs=inputs, aotBuild=dict(command=command, exitCode=result.returncode, stderr=result.stderr,
                                    dependencies=dependencies),
        bundleManifestSha256=builder.sha(bundle / 'manifest.json'),
        files={p.relative_to(staged).as_posix(): builder.sha(p) for p in sorted(staged.rglob('*')) if p.is_file()})
    (staged / 'native-build-kit.json').write_text(json.dumps(kit, indent=2) + '\n')
    archive = output / 'neoclr-native-build-kit-osx-arm64.tar.gz'
    with tarfile.open(archive, 'w:gz') as tar:
        tar.add(staged, arcname=staged.name)
    (output / 'SHA256SUMS').write_text(builder.sha(archive) + '  ' + archive.name + '\n')
    print(archive)


if __name__ == '__main__':
    main()

#!/usr/bin/env python3
"""Rebuild source libraries using a verified bundle's compiler and primitive core."""
import argparse
import hashlib
import json
import os
import stat
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--compiler-revision', required=True, help='Revision of the selected bundle compiler (binary hashes are also retained)')
    parser.add_argument('--build-compiler', action='store_true', help='Build the exact full Raven commit before rebuilding libraries')
    args = parser.parse_args()
    original = args.bundle.resolve()
    manifest = json.loads((original / 'manifest.json').read_text())
    for relative, digest in manifest['files'].items():
        path = (original / relative).resolve()
        if original not in path.parents or sha(path) != digest:
            raise ValueError('Original bundle hash mismatch: ' + relative)
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    report = dict(sourceRevision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                  originalManifestSha256=sha(original / 'manifest.json'), commands=[])
    def run(command):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True, timeout=600)
        report['commands'].append(dict(command=result.args, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        (out / 'preparation.json').write_text(json.dumps(report, indent=2) + '\n')
        if result.returncode:
            raise RuntimeError(result.stdout + result.stderr)
    compiler = original / 'sdk/tools/rvnc/rvnc.dll'
    compiler_directory = None
    if args.build_compiler:
        if len(args.compiler_revision) != 40 or any(c not in '0123456789abcdef' for c in args.compiler_revision):
            raise ValueError('--build-compiler requires an exact full Raven commit')
        # Raven generators launch executables beneath this checkout. Keeping it
        # below a deeply nested evidence directory exceeds Windows launch limits.
        scratch = ROOT / 'target'
        scratch.mkdir(exist_ok=True)
        source = Path(tempfile.mkdtemp(prefix='rvnc-', dir=scratch))
        try:
            run(['git', 'init', source])
            run(['git', '-C', source, 'remote', 'add', 'origin', 'https://github.com/marinasundstrom/raven.git'])
            run(['git', '-C', source, 'fetch', '--depth', '1', 'origin', args.compiler_revision])
            run(['git', '-C', source, 'checkout', '--detach', args.compiler_revision])
            actual = subprocess.check_output(['git', '-C', str(source), 'rev-parse', 'HEAD'], text=True).strip()
            if actual != args.compiler_revision:
                raise ValueError('Raven checkout revision mismatch')
            metadata = ROOT / 'tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj'
            run(['dotnet', 'build', source / 'src/Raven.Compiler/Raven.Compiler.csproj', '-f', 'net10.0',
                 '-p:WarningLevel=0', '-p:UseRavenCoreReference=false', '-p:Platform=AnyCPU',
                 '-p:NeoClrMetadataProject=' + str(metadata)])
            compiler_directory = out / 'compiler'
            shutil.copytree(source / 'src/Raven.Compiler/bin/Debug/net10.0', compiler_directory)
            compiler = compiler_directory / 'rvnc.dll'
            report['compilerRevision'] = actual
            report['compilerFiles'] = {p.relative_to(compiler_directory).as_posix(): sha(p)
                                       for p in sorted(compiler_directory.rglob('*')) if p.is_file()}
        finally:
            def remove_readonly(function, path, error):
                os.chmod(path, stat.S_IWRITE)
                function(path)
            shutil.rmtree(source, onerror=remove_readonly)
    translator_project = ROOT / 'tools/metadata/NeoCLR.Metadata.Translate/NeoCLR.Metadata.Translate.csproj'
    translator = translator_project.parent / 'bin/Release/net10.0/NeoCLR.Metadata.Translate.dll'
    run(['dotnet', 'build', translator_project, '-c', 'Release', '-p:WarningLevel=0', '-p:Platform=AnyCPU'])
    vm = original / ('bin/neoclr.exe' if sys.platform == 'win32' else 'bin/neoclr')
    run([sys.executable, ROOT / 'scripts/prepare-native-bootstrap.py', '--core-reference', original / 'lib/Core.dll',
         '--runtime', vm, '--translator', translator, '--output', out / 'bootstrap'])
    # The library evidence retains compiler binaries as well as its declared revision.
    run([sys.executable, ROOT / 'scripts/build-native-class-library.py', '--compiler', compiler,
         '--core', out / 'bootstrap/Core.dll', '--bootstrap-directory', out / 'bootstrap', '--translator', translator,
         '--output', out / 'libraries', '--compiler-revision', args.compiler_revision])
    bundle = out / 'bundle'
    shutil.copytree(original, bundle, ignore=lambda directory, names: [n for n in names if Path(directory) == original and n in ('lib', 'manifest.json')])
    shutil.copytree(out / 'libraries', bundle / 'lib')
    if compiler_directory is not None:
        shutil.rmtree(bundle / 'sdk/tools/rvnc')
        shutil.copytree(compiler_directory, bundle / 'sdk/tools/rvnc')
    manifest['files'] = {p.relative_to(bundle).as_posix(): sha(p) for p in sorted(bundle.rglob('*')) if p.is_file()}
    manifest['developmentLibraries'] = dict(sourceRevision=report['sourceRevision'], originalManifestSha256=report['originalManifestSha256'])
    (bundle / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    (out / 'preparation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(bundle)

if __name__ == '__main__':
    main()

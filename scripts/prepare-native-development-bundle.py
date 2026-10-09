#!/usr/bin/env python3
"""Rebuild source libraries using a verified bundle's compiler and primitive core."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--compiler-revision', required=True, help='Revision of the selected bundle compiler (binary hashes are also retained)')
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
    translator_project = ROOT / 'tools/metadata/NeoCLR.Metadata.Translate/NeoCLR.Metadata.Translate.csproj'
    translator = translator_project.parent / 'bin/Release/net10.0/NeoCLR.Metadata.Translate.dll'
    run(['dotnet', 'build', translator_project, '-c', 'Release', '-p:WarningLevel=0', '-p:Platform=AnyCPU'])
    vm = original / ('bin/neoclr.exe' if sys.platform == 'win32' else 'bin/neoclr')
    run([sys.executable, ROOT / 'scripts/prepare-native-bootstrap.py', '--core-reference', original / 'lib/Core.dll',
         '--runtime', vm, '--translator', translator, '--output', out / 'bootstrap'])
    # The library evidence retains compiler binaries as well as its declared revision.
    run([sys.executable, ROOT / 'scripts/build-native-class-library.py', '--compiler', original / 'sdk/tools/rvnc/rvnc.dll',
         '--core', out / 'bootstrap/Core.dll', '--bootstrap-directory', out / 'bootstrap', '--translator', translator,
         '--output', out / 'libraries', '--compiler-revision', args.compiler_revision])
    bundle = out / 'bundle'
    shutil.copytree(original, bundle, ignore=lambda directory, names: [n for n in names if Path(directory) == original and n in ('lib', 'manifest.json')])
    shutil.copytree(out / 'libraries', bundle / 'lib')
    manifest['files'] = {p.relative_to(bundle).as_posix(): sha(p) for p in sorted(bundle.rglob('*')) if p.is_file()}
    manifest['developmentLibraries'] = dict(sourceRevision=report['sourceRevision'], originalManifestSha256=report['originalManifestSha256'])
    (bundle / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(bundle)

if __name__ == '__main__':
    main()

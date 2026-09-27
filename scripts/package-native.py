#!/usr/bin/env python3
"""Archive and smoke-test a native runtime package on its target host (no SDK)."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    runtime = args.runtime.resolve()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    target = {'Windows': 'win', 'Darwin': 'osx', 'Linux': 'linux'}[platform.system()]
    machine = platform.machine().lower()
    arch = {'amd64': 'x64', 'x86_64': 'x64', 'arm64': 'arm64', 'aarch64': 'arm64'}[machine]
    version = re.search(r'^version = "([^"]+)"', (ROOT / 'Cargo.toml').read_text(), re.MULTILINE).group(1)
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    dirty = bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT, text=True).strip())
    name = f'neoclr-{version}-{target}-{arch}'
    staged = output / name
    staged.mkdir()
    (staged / 'bin').mkdir()
    (staged / 'lib').mkdir()
    executable = 'neoclr.exe' if target == 'win' else 'neoclr'
    shutil.copy2(runtime, staged / 'bin' / executable)
    spec = importlib.util.spec_from_file_location('collection_library', ROOT / 'docs/experiments/raven-target/collection_library.py')
    library = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(library)
    (staged / 'lib/System.neoil').write_text(library.build(ROOT / 'runtime/System.neoil'), encoding='utf-8')
    shutil.copytree(ROOT / 'examples/preview', staged / 'samples/neoil')
    shutil.copytree(ROOT / 'third-party', staged / 'third-party')
    for source in ('LICENSE', 'THIRD_PARTY_NOTICES.md'):
        shutil.copy2(ROOT / source, staged / source)
    (staged / 'README.md').write_text(f'''# neoCLR native runtime preview

Target: {target}-{arch}. Source: {revision}.

Run from this extracted directory:

```
./bin/{executable} run samples/neoil/type-categories.neoil --system lib/System.neoil
```

Expected output: 42, 7 and 9 on separate lines. On Windows use PowerShell.
No .NET installation is required for these direct-runtime samples. The Raven SDK,
compiler bridge and VS Code extension are separate tools and are not in this
native-only package. This package does not claim Windows Raven SDK qualification.
See https://github.com/marinasundstrom/neoCLR for source and preview limitations.
''', encoding='utf-8')
    hashes = {p.relative_to(staged).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
              for p in staged.rglob('*') if p.is_file()}
    (staged / 'manifest.json').write_text(json.dumps({'revision': revision, 'version': version,
        'target': f'{target}-{arch}', 'dirty_source': dirty, 'scope': 'native runtime and direct-runtime samples; no Raven SDK',
        'files': hashes}, indent=2) + '\n', encoding='utf-8')
    archive = output / (name + '.zip')
    with zipfile.ZipFile(archive, 'x', compression=zipfile.ZIP_DEFLATED) as zipped:
        for path in sorted(staged.rglob('*')):
            if path.is_file():
                zipped.write(path, name + '/' + path.relative_to(staged).as_posix())
    with tempfile.TemporaryDirectory(prefix='neoclr-native-package-') as temporary:
        with zipfile.ZipFile(archive) as zipped:
            zipped.extractall(temporary)  # Archive contains only the enumerated staged paths above.
        extracted = Path(temporary) / name
        for relative, digest in hashes.items():
            assert hashlib.sha256((extracted / relative).read_bytes()).hexdigest() == digest, relative
        binary = extracted / 'bin' / executable
        if target != 'win':
            binary.chmod(0o755)
        result = subprocess.run([sys.executable, str(ROOT / 'docs/experiments/raven-target/verify_neoil.py'),
            '--runtime', str(binary), '--system', str(extracted / 'lib/System.neoil'),
            '--samples', str(extracted / 'samples/neoil')], capture_output=True, text=True)
        (output / 'smoke.log').write_text(result.stdout + result.stderr, encoding='utf-8')
        result.check_returncode()
    report = {'status': 'passed', 'revision': revision, 'target': f'{target}-{arch}',
              'dirty_source': dirty, 'archive': archive.name, 'sha256': hashlib.sha256(archive.read_bytes()).hexdigest(),
              'files_verified': len(hashes), 'extracted_samples_passed': 4}
    (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()

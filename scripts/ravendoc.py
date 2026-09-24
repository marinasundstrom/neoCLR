#!/usr/bin/env python3
"""Verify and run the repository's portable RavenDoc build (.NET 10 required)."""
import hashlib
import json
from pathlib import Path
import tempfile
import subprocess
import sys
import zipfile

ROOT = Path(__file__).resolve().parent.parent
TOOL = ROOT / 'tools/ravendoc'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def runtime():
    lock = json.loads((TOOL / 'version.json').read_text())
    archive = TOOL / lock['archive']
    if digest(archive) != lock['sha256']:
        raise ValueError('RavenDoc archive checksum mismatch')
    output = ROOT / 'target/tools/ravendoc' / lock['sha256']
    marker = output / '.verified'
    if not marker.exists():
        output.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='.extract-', dir=output.parent) as temporary:
            staging = Path(temporary) / 'runtime'
            staging.mkdir()
            with zipfile.ZipFile(archive) as package:
                for name in package.namelist():
                    if not (staging / name).resolve().is_relative_to(staging.resolve()):
                        raise ValueError('Unsafe RavenDoc archive member')
                package.extractall(staging)
            (staging / '.verified').touch()
            try:
                staging.rename(output)
            except OSError:
                # Another process may have published the same verified extraction.
                if not marker.exists():
                    raise
    return output / 'RavenDoc.dll'


if __name__ == '__main__':
    sys.exit(subprocess.call(['dotnet', str(runtime()), *sys.argv[1:]], cwd=ROOT))

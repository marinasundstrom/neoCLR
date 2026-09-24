#!/usr/bin/env python3
"""Build and vendor the reviewed upstream RavenDoc publisher."""
import argparse
from contextlib import contextmanager
import errno
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import time
import zipfile

ROOT = Path(__file__).resolve().parent.parent
TOOL = ROOT / 'tools/ravendoc'
REPOSITORY = 'https://github.com/marinasundstrom/raven.git'
DEFAULT_REVISION = json.loads((TOOL / 'version.json').read_text())['revision']


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def package(published, revision):
    archive = TOOL / 'ravendoc-net10.0.zip'
    with zipfile.ZipFile(archive, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=9) as output:
        for path in sorted(published.rglob('*')):
            if path.is_file() and path.suffix not in ('.pdb', '.xml'):
                info = zipfile.ZipInfo(path.relative_to(published).as_posix(), (2026, 1, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                output.writestr(info, path.read_bytes())
    lock = dict(repository=REPOSITORY, revision=revision, framework='net10.0',
                archive=archive.name, sha256=digest(archive))
    (TOOL / 'version.json').write_text(json.dumps(lock, indent=2) + '\n')


@contextmanager
def temporary_checkout():
    directory = Path(tempfile.mkdtemp(prefix='ravendoc-update-', dir=ROOT / 'target'))
    try:
        yield directory
    finally:
        # A build server may finish writing generated files during cleanup.
        for attempt in range(3):
            try:
                shutil.rmtree(directory)
                break
            except FileNotFoundError:
                break
            except OSError as error:
                if error.errno != errno.ENOTEMPTY or attempt == 2:
                    raise
                time.sleep(0.25)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--revision', default=DEFAULT_REVISION, help='Full reviewed upstream commit SHA')
    parser.add_argument('--source', type=Path, help='Local Raven repository to read the pinned commit from')
    args = parser.parse_args()
    if len(args.revision) != 40 or any(c not in '0123456789abcdef' for c in args.revision):
        parser.error('Use a full lowercase commit SHA, not a moving branch')
    (ROOT / 'target').mkdir(exist_ok=True)
    with temporary_checkout() as temp:
        source = Path(temp) / 'source'
        subprocess.run(['git', 'clone', '--no-checkout', str(args.source or REPOSITORY), str(source)], check=True)
        subprocess.run(['git', 'checkout', '--detach', args.revision], cwd=source, check=True)
        published = Path(temp) / 'publish'
        subprocess.run(['dotnet', 'publish', str(source / 'src/RavenDoc/RavenDoc.csproj'),
                        '-c', 'Release', '-f', 'net10.0', '-p:TargetFrameworks=net10.0',
                        '-p:UseAppHost=false', '-o', str(published)], check=True)
        subprocess.run(['dotnet', str(published / 'RavenDoc.dll'), '--help'], check=True)
        package(published, args.revision)
        shutil.copyfile(source / 'LICENSE', TOOL / 'LICENSE')
    print('Updated RavenDoc. Run the full website validation before committing.')


if __name__ == '__main__':
    main()

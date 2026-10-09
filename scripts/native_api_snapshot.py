"""Read a reviewed, checksum-pinned native API rendering for website publication."""
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import stat
import tempfile
import zipfile


def extract_snapshot(manifest_path, destination):
    manifest = json.loads(manifest_path.read_text())
    archive_name = manifest['archive']
    if manifest['format'] != 1 or Path(archive_name).name != archive_name:
        raise ValueError('Invalid native API snapshot manifest')
    archive = manifest_path.parent / archive_name
    if hashlib.sha256(archive.read_bytes()).hexdigest() != manifest['sha256']:
        raise ValueError('Native API snapshot checksum mismatch')
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=destination.parent, prefix='native-api-') as temporary:
        staging = Path(temporary) / 'audit'
        staging.mkdir()
        with zipfile.ZipFile(archive) as files:
            names = set()
            total = 0
            for member in files.infolist():
                path = PurePosixPath(member.filename)
                total += member.file_size
                if (member.filename in names or path.is_absolute() or '..' in path.parts
                        or '\\' in member.filename or path.parts[:3] != ('site', 'docs', 'api')
                        or stat.S_ISLNK(member.external_attr >> 16)
                        or total > 128 * 1024 * 1024):
                    raise ValueError('Invalid native API snapshot member: ' + member.filename)
                names.add(member.filename)
            files.extractall(staging)
        if not (staging / 'site/docs/api/System/Object/index.html').is_file():
            raise ValueError('Native API snapshot is missing Object')
        shutil.rmtree(destination, ignore_errors=True)
        staging.rename(destination)
    return destination

"""Read a reviewed, checksum-pinned native API rendering for website publication."""
from html.parser import HTMLParser
from urllib.parse import quote
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import stat
import tempfile
import zipfile


SOURCE_LINK_CHECKS = {
    'System/IO/FileSystemWatcher/index.html': 'runtime/System.Runtime/src/System.Runtime/System/IO/FileSystemWatcher.rvn',
    'System/IO/FileSystemWatcher/method_OnChanged.html': 'runtime/System.Runtime/src/System.Runtime/System/IO/FileSystemWatcher.rvn',
    'Neo/Configuration/Json/JsonConfigurationProvider/index.html': 'extensions/Neo.Configuration/src/Neo.Configuration.Json/Neo/Configuration/Json/JsonConfiguration.rvn',
    'Neo/Configuration/Configuration/index.html': 'extensions/Neo.Configuration/src/Neo.Configuration.Abstractions/Neo/Configuration/Contracts.rvn',
    'Neo/Logging/Logger/index.html': 'extensions/Neo.Logging/src/Neo.Logging.Abstractions/Neo/Logging/Contracts.rvn',
    'Neo/Logging/DefaultLoggerFactory/index.html': 'extensions/Neo.Logging/src/Neo.Logging/Neo/Logging/DefaultLoggerFactory.rvn',
    'Neo/DependencyInjection/ServiceCollection/index.html': 'extensions/Neo.DependencyInjection/src/Neo.DependencyInjection.Abstractions/Neo/DependencyInjection/Services.rvn',
    'Neo/Localization/Localizer/index.html': 'extensions/Neo.Localization/src/Neo.Localization.Abstractions/Neo/Localization/Localizer.rvn',
    'Neo/Localization/JsonLocalizerProvider/index.html': 'extensions/Neo.Localization/src/Neo.Localization/Neo/Localization/JsonLocalizerProvider.rvn',
    'System/Data/Json/JsonValue/index.html': 'runtime/System.Data/src/System.Data/System/Data/Json/JsonValue.rvn',
    'System/Data/Json/JsonValue/method_ToJsonString.html': 'runtime/System.Data/src/System.Data/System/Data/Json/JsonValue.rvn',
    'System/Data/Json/JsonSerializerOptions/index.html': 'runtime/System.Data/src/System.Data/System/Data/Json/JsonSerializerOptions.rvn',
    'System/Data/Json/JsonSerializerOptions/property_WriteIndented.html': 'runtime/System.Data/src/System.Data/System/Data/Json/JsonSerializerOptions.rvn',
    'System/Concurrency/Tasks/Task`1/index.html': 'runtime/System.Runtime/src/System.Runtime/System/Concurrency/Tasks/Tasks.rvn',
    'System/Data/PostgreSql/PostgresDatabase/index.html': 'runtime/System.Data.PostgreSql/src/System.Data.PostgreSql/System/Data/PostgreSql/PostgresDatabase.rvn',
    'System/Object/index.html': 'runtime/System.Runtime/src/System.Runtime/System/Object.rvn',
    'System/Object/method_GetHashCode.html': 'runtime/System.Runtime/src/System.Runtime/System/Object.rvn',
    'System/Web/Http/HttpClient/index.html': 'runtime/System.Web/src/System.Web/System/Web/Http/HttpClient.rvn',
    'System/Web/Http/HttpClient/method_Send.html': 'runtime/System.Web/src/System.Web/System/Web/Http/HttpClient.rvn',
}


def validate_source_links(source, repository):
    """Guard real declaration-file links on both type and member pages."""
    class Links(HTMLParser):
        def __init__(self):
            super().__init__()
            self.hrefs = []
        def handle_starttag(self, tag, attrs):
            if tag == 'a':
                self.hrefs.extend(value for key, value in attrs if key == 'href')
    base = repository['url'].rstrip('/') + '/blob/' + quote(repository.get('revision', 'main'), safe='') + '/'
    for relative, declaration in SOURCE_LINK_CHECKS.items():
        page = source / relative
        links = Links()
        if page.is_file():
            links.feed(page.read_text())
        expected = base + declaration
        if not any(href and href.split('#', 1)[0] == expected for href in links.hrefs):
            raise ValueError('Native API declaration source link missing or incorrect: ' + relative)


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
        if manifest.get('sourceRepository'):
            validate_source_links(staging / 'site/docs/api', manifest['sourceRepository'])
        shutil.rmtree(destination, ignore_errors=True)
        staging.rename(destination)
    return destination

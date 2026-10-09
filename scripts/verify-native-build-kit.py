#!/usr/bin/env python3
"""Qualify an archived native build kit after extraction outside the source checkout."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, archiveSha256=sha(args.archive), cases=[])
    try:
        with tempfile.TemporaryDirectory(prefix='native kit extracted with spaces ') as temporary:
            location = Path(temporary).resolve()
            with tarfile.open(args.archive) as archive:
                # Python 3.9-compatible extraction of this regular-file-only format.
                members = archive.getmembers()
                names = set()
                for member in members:
                    destination = (location / member.name).resolve()
                    if (Path(member.name).is_absolute() or location not in destination.parents
                            or not (member.isfile() or member.isdir()) or member.name in names):
                        raise ValueError('Invalid build kit archive member: ' + member.name)
                    names.add(member.name)
                archive.extractall(location, members=members)
            kit = location / 'neoclr-native-build-kit'
            manifest = json.loads((kit / 'native-build-kit.json').read_text())
            report['kitManifestSha256'] = sha(kit / 'native-build-kit.json')
            report['profile'] = manifest['profile']
            report['sourceRevision'] = manifest['revision']
            report['sourceDirty'] = manifest['dirty']
            report['sourceInputs'] = manifest['inputs']
            report['aotBuild'] = manifest['aotBuild']
            report['bundleManifestSha256'] = manifest['bundleManifestSha256']
            report['packagedFileCount'] = len(manifest['files'])
            # Keep OS build tools and dotnet available, but remove the Cargo toolchain.
            environment = {**os.environ, 'PATH': str(Path(shutil.which('dotnet')).parent) +
                           ':/usr/bin:/bin:/usr/sbin:/sbin'}
            assert shutil.which('cargo', path=environment['PATH']) is None

            def build(name):
                destination = location / name
                command = [sys.executable, kit / 'scripts/build-native-project.py',
                           '--project', kit / 'samples/hello/App.rvnproj', '--output', destination]
                result = subprocess.run(list(map(str, command)), cwd=location, env=environment,
                                        capture_output=True, text=True, timeout=240)
                evidence = json.loads((destination / 'build.json').read_text())
                row = dict(name=name, exitCode=result.returncode, stdout=result.stdout,
                           stderr=result.stderr, build=evidence)
                report['cases'].append(row)
                return result, evidence, destination

            result, evidence, destination = build('native hello output')
            assert result.returncode == 0 and evidence['passed'], result.stderr
            assert all(kit in Path(path).parents for path in evidence['inputs'])
            assert all(str(ROOT) not in str(c['command']) for c in evidence['commands'])
            standalone = location / 'standalone'
            standalone.mkdir()
            shutil.copy2(destination / 'app', standalone / 'app')
            native = subprocess.run([str(standalone / 'app')], cwd=standalone, env={}, capture_output=True, timeout=30)
            assert native.returncode == 0 and native.stdout == 'Hello, Café 🌍\n'.encode() and not native.stderr
            assert list(standalone.iterdir()) == [standalone / 'app']
            report['execution'] = dict(exitCode=0, stdout=native.stdout.decode(), executableOnlyDirectory=True,
                                       emptyEnvironment=True, buildWithoutCargo=True, checkoutPathsAbsent=True)
            for name, relative in [('adapter', 'tools/native/console-host.c'), ('backend', 'bin/neoclr-aot-poc'),
                                   ('compiler', 'bundle/sdk/tools/rvnc/rvnc.runtimeconfig.json')]:
                path = kit / relative
                size = path.stat().st_size
                with path.open('ab') as stream:
                    stream.write(b'\nmodified\n')
                try:
                    result, evidence, destination = build('reject changed ' + name)
                    assert result.returncode != 0 and not evidence['passed']
                    assert evidence['commands'] == [] and 'Package file mismatch' in evidence['error']
                    assert not (destination / 'app').exists() and not (destination / 'app.dll').exists()
                finally:
                    with path.open('r+b') as stream:
                        stream.truncate(size)
            # Retain compact evidence; command streams remain byte-verifiable.
            for case in report['cases']:
                for command in case['build']['commands']:
                    for field in ('stdout', 'stderr'):
                        value = command[field]
                        if len(value) > 2500:
                            command[field + 'Sha256'] = hashlib.sha256(value.encode()).hexdigest()
                            command[field + 'Bytes'] = len(value.encode())
                            command[field] = value[:1000] + '\n[compacted]\n' + value[-1000:]
            report['passed'] = True
    finally:
        (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Passed: extracted native kit, no checkout/Cargo build dependency, standalone UTF-8 execution and three pre-build integrity rejections')


if __name__ == '__main__':
    main()

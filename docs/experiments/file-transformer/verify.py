"""Compile once, then check file-transformer success and failed-save behavior."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', required=True, type=Path)
args = parser.parse_args()
bundle = args.toolchain_root.resolve()
payload = '{"station":"Café","readings":[21,22.5]}'.encode('utf-8')
with tempfile.TemporaryDirectory(prefix='neoclr-file-transformer-') as folder:
    root = Path(folder)
    for name in ('file-transformer', 'json-document', 'json-message'):
        target = root / name
        target.mkdir()
        for source in (HERE.parent / name).glob('*.rvn'):
            shutil.copyfile(source, target / source.name)
    project = root / 'file-transformer'
    shutil.copyfile(HERE / 'FileTransformer.rvnproj', project / 'FileTransformer.rvnproj')
    env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
    built = subprocess.run(['dotnet', 'msbuild', str(project / 'FileTransformer.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=120)
    assert built.returncode == 0, built.stdout + built.stderr
    cases = [
        ('valid', payload, None, 'Saved reply.json\n'),
        ('existing', payload, b'preserve me', 'Failed: AlreadyExists\n'),
        ('invalid-utf8', b'\xc3', None, 'Failed: InvalidUtf8\n'),
        ('oversized', b' ' * 129, None, 'Failed: LimitExceeded\n'),
        ('malformed-json', b'{"station":', None, None),
        ('missing-field', '{"station":"Café"}'.encode('utf-8'), None, None),
        ('wrong-reading', b'{"station":"x","readings":[1.5]}', None, None),
        ('missing-source', None, None, 'Failed: NotFound\n'),
        ('directory-source', None, None, 'Failed: WrongKind\n'),
    ]
    for name, source, existing, expected in cases:
        work = root / name
        storage = work / 'reports'
        storage.mkdir(parents=True)
        if source is not None:
            (storage / 'report.json').write_bytes(source)
        elif name == 'directory-source':
            (storage / 'report.json').mkdir()
        if existing is not None:
            (storage / 'reply.json').write_bytes(existing)
        result = subprocess.run([str(bundle / 'bin/neoclr'), 'run', str(project / 'bin/neoclr/Debug/App.neoil'), '--system', str(bundle / 'lib/System.neoil')], cwd=work, capture_output=True, text=True, timeout=60)
        assert result.returncode == 0, (name, result.stdout, result.stderr)
        if expected is not None:
            assert result.stdout == expected, (name, result.stdout)
        else:
            assert result.stdout.startswith('Failed: '), (name, result.stdout)
        destination = storage / 'reply.json'
        if name == 'valid':
            actual = destination.read_bytes()
            assert actual == '{"station":"Café","accepted":true,"count":2,"note":null}'.encode('utf-8'), actual
            assert json.loads(actual) == {'station': 'Café', 'accepted': True, 'count': 2, 'note': None}
        elif existing is not None:
            assert destination.read_bytes() == existing
        else:
            assert not destination.exists(), name
        if source is not None:
            assert (storage / 'report.json').read_bytes() == source, name
        assert not (work / 'reply.json').exists()
        print(f'{name}: {result.stdout.strip()}', flush=True)
    print(f'File Transformer: {len(cases)} cases passed')

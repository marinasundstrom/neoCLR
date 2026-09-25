"""Focused JSON string/stream adapter, short-transfer and ownership checks."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--toolchain-root', type=Path, required=True)
    parser.add_argument('--runner', type=Path, required=True)
    args = parser.parse_args()
    bundle = args.toolchain_root.resolve()
    here = Path(__file__).resolve().parent
    env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
    with tempfile.TemporaryDirectory(prefix='neoclr-json-streams-') as folder:
        root = Path(folder)
        reference = root / 'reference'
        reference.mkdir()
        (reference / 'global.json').write_text('{"sdk":{"version":"10.0.100","rollForward":"disable"}}')
        (reference / 'Reference.csproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>Exe</OutputType><TargetFramework>net10.0</TargetFramework></PropertyGroup></Project>')
        shutil.copyfile(here / 'Reference.cs', reference / 'Program.cs')
        baseline = subprocess.run(['dotnet', 'run', '--project', str(reference / 'Reference.csproj'), '-v:q'], cwd=reference, capture_output=True, text=True, timeout=120)
        assert baseline.returncode == 0 and '.NET memory/JSON baseline passed' in baseline.stdout, baseline.stdout + baseline.stderr
        print(baseline.stdout, flush=True)
        for name in ('json-message', 'json-document', 'json-streams'):
            (root / name).mkdir()
        for group, files in {
            'json-message': ['JsonMessage.rvn'],
            'json-document': ['JsonValue.rvn', 'JsonDocument.rvn'],
            'json-streams': ['JsonSerializer.rvn', 'Main.rvn', 'JsonStreams.rvnproj'],
        }.items():
            for name in files:
                shutil.copyfile(here.parent / group / name, root / group / name)
        project = root / 'json-streams'
        built = subprocess.run(['dotnet', 'msbuild', str(project / 'JsonStreams.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=240)
        assert built.returncode == 0, built.stdout + built.stderr
        assert 'warning ' not in built.stdout, built.stdout
        result = subprocess.run([str(args.runner.resolve()), str(project / 'bin/neoclr/Debug/App.neoil'), str(bundle / 'lib/System.neoil'), '512', '100000000'], capture_output=True, text=True, timeout=120)
        assert result.returncode == 0, result.stdout + result.stderr
        lines = result.stdout.splitlines()
        assert json.loads(lines[0]) == {'station': 'Café 😀', 'readings': [21, 22.5], 'ready': True, 'note': None}, result.stdout
        assert lines[1:] == ['JSON stream checks passed'], result.stdout
        assert 'live=0' in result.stderr, result.stderr
        print(result.stdout + result.stderr)


if __name__ == '__main__':
    main()

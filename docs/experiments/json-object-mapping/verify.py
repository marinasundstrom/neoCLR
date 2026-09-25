"""Compile and execute the JSON object-mapping consumer against a matching local bundle."""
import argparse
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
    with tempfile.TemporaryDirectory(prefix='neoclr-json-mapping-') as folder:
        root = Path(folder)
        for name in ('Main.rvn', 'Mapping.rvn', 'JsonObjectMapping.rvnproj'):
            shutil.copyfile(here / name, root / name)
        built = subprocess.run(['dotnet', 'msbuild', str(root / 'JsonObjectMapping.rvnproj'),
                                '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=240)
        assert built.returncode == 0, built.stdout + built.stderr
        assert 'warning ' not in built.stdout, built.stdout
        result = subprocess.run([str(args.runner.resolve()), str(root / 'bin/neoclr/Debug/App.neoil'),
                                 str(bundle / 'lib/System.neoil'), '512', '100000000'],
                                capture_output=True, text=True, timeout=120)
        assert result.returncode == 0, result.stdout + result.stderr
        assert result.stdout.splitlines() == ['JSON object mapping checks passed'], result.stdout
        assert 'live=0' in result.stderr, result.stderr
        print(result.stdout + result.stderr)


if __name__ == '__main__':
    main()

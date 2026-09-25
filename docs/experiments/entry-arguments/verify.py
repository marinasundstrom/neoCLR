"""Check managed entry arguments, host-vector separation and empty startup."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


def run(command, **kwargs):
    result = subprocess.run(command, capture_output=True, text=True, timeout=240, **kwargs)
    assert result.returncode == 0, result.stdout + result.stderr
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--toolchain-root', type=Path, required=True)
    parser.add_argument('--runner', type=Path, required=True)
    args = parser.parse_args()
    bundle = args.toolchain_root.resolve()
    here = Path(__file__).resolve().parent
    env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
    with tempfile.TemporaryDirectory(prefix='neoclr-entry-arguments-') as folder:
        root = Path(folder)
        for name in ('Main.rvn', 'EntryArguments.rvnproj'):
            shutil.copyfile(here / name, root / name)
        run(['dotnet', 'msbuild', str(root / 'EntryArguments.rvnproj'), '-nologo', '-v:minimal'], env=env)
        command = [str(args.runner.resolve()), str(root / 'bin/neoclr/Debug/App.neoil'),
                   str(bundle / 'lib/System.neoil'), '512', '100000000']
        cases = [
            ([], ['0']),
            (['--'], ['0']),
            (['--', 'café 😀', '', 'two words', '--option'],
             ['4', '[café 😀]', '[]', '[two words]', '[--option]', 'independent']),
        ]
        for suffix, expected in cases:
            result = run(command + suffix)
            assert result.stdout.splitlines() == expected, result.stdout
            assert 'live=0' in result.stderr, result.stderr
        print('Entry arguments: 3 cases passed; no retained heap objects')


if __name__ == '__main__':
    main()

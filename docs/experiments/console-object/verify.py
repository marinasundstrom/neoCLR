"""Check Console object fallback and existing overload selection."""
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
    with tempfile.TemporaryDirectory(prefix='neoclr-console-object-') as folder:
        root = Path(folder)
        for name in ('Main.rvn', 'ConsoleObject.rvnproj'):
            shutil.copyfile(here / name, root / name)
        run(['dotnet', 'msbuild', str(root / 'ConsoleObject.rvnproj'), '-nologo', '-v:minimal'], env=env)
        imported = (root / 'bin/neoclr/Debug/App.neoil').read_text()
        for scalar in ('Boolean', 'Char', 'SByte', 'Byte', 'Int16', 'UInt16', 'UInt32', 'Int64', 'UInt64', 'IntPtr', 'UIntPtr'):
            assert 'call System.Console::WriteLine(' + scalar + ')' in imported, scalar
        library = (bundle / 'lib/System.neoil').read_text()
        for scalar in ('Boolean', 'Char', 'SByte', 'Byte', 'Int16', 'UInt16', 'UInt32', 'Int64', 'UInt64', 'IntPtr', 'UIntPtr'):
            body = library.split('.method static WriteLine(' + scalar + ' ', 1)[1].split('.end', 1)[0]
            assert 'box ' not in body, scalar
        command = [str(args.runner.resolve()), str(root / 'bin/neoclr/Debug/App.neoil'),
                   str(bundle / 'lib/System.neoil'), '512', '100000000']
        cases = [([], ['Named instance', 'Named instance', '42', 'True', 'boxed text', '', 'text', '7', 'False', '😀', '-128', '255', '-32768', '65535', '4294967295', '-9223372036854775808', '18446744073709551615', '0', '0'])]
        for suffix, expected in cases:
            result = run(command + suffix)
            assert result.stdout.splitlines() == expected, result.stdout
            assert 'live=0' in result.stderr, result.stderr
        print('Console object fallback passed; no retained heap objects')


if __name__ == '__main__':
    main()

"""Compile the Console sample and exercise separate UTF-8/byte standard channels."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', required=True, type=Path)
parser.add_argument('--case', choices=('all', 'greeting', 'contracts', 'propagation'), default='all')
args = parser.parse_args()
bundle = args.toolchain_root.resolve()
with tempfile.TemporaryDirectory(prefix='neoclr-console-streams-') as folder:
    root = Path(folder)
    shutil.copyfile(HERE / 'ConsoleStreams.rvnproj', root / 'ConsoleStreams.rvnproj')
    env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
    def build(source):
        shutil.copyfile(HERE / source, root / 'Main.rvn')
        result = subprocess.run(['dotnet', 'msbuild', str(root / 'ConsoleStreams.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=120)
        assert result.returncode == 0, result.stdout + result.stderr
    def run(data):
        result = subprocess.run([str(bundle / 'bin/neoclr'), 'run', str(root / 'bin/neoclr/Debug/App.neoil'), '--system', str(bundle / 'lib/System.neoil')], input=data, capture_output=True, timeout=60)
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout, result.stderr
    if args.case in ('all', 'greeting'):
        build('Main.rvn')
        for data, out, err in [
            ('värld\r\n'.encode(), 'Name: Hello, värld\n'.encode(), b'Read one line\n'),
            (b'\n', b'Name: Hello, \n', b'Read one line\n'),
            (b'', b'Name: End of input\n', b''),
            (b'last', b'Name: Hello, last\n', b'Read one line\n'),
            (b'\xff\n', b'Name: ', b'Could not read a UTF-8 line (maximum 128 bytes)\n'),
            (b'a' * 129 + b'\n', b'Name: ', b'Could not read a UTF-8 line (maximum 128 bytes)\n'),
        ]:
            assert run(data) == (out, err)
        print('Greeting: UTF-8, CRLF, empty line, EOF, final line, malformed input and bounds passed')
    if args.case in ('all', 'contracts'):
        build('Contracts.rvn')
        assert run('å\r\n\ntail'.encode()) == (b'A\n42\nConsole stream contracts passed\n', b'')
        print('Short writes, UTF-8 bytes, stream dispatch, ranges and close ownership passed')

    if args.case in ('all', 'propagation'):
        for source in ('Propagation.rvn', 'IfLet.rvn'):
            build(source)
            for data, out, err in [
                (b'Ada\n', b'Input is: Ada\n', b''),
                (b'', b'No input\n', b''),
                (b'\n', b'Input is: \n', b''),
                (b'\xff\n', b'', b'Could not read input\n'),
            ]:
                assert run(data) == (out, err)
            print(source + ': propagation, binding, EOF and empty-line distinction passed')

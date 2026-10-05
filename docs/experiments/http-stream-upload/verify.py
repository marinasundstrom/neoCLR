"""Known-length upload checks against an independent HTTP peer; build once, run serially."""
import argparse
import json
import os
from pathlib import Path
import shutil
import socket
import subprocess
import tempfile
import threading

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path)
for name in ('compiler', 'core', 'seed', 'ownership'):
    parser.add_argument('--' + name, type=Path)
parser.add_argument('--native-library', type=Path, action='append', default=[])
parser.add_argument('--runner', type=Path, required=True)
parser.add_argument('--sdk', type=Path, help='Separate extracted Raven SDK; defaults to TOOLCHAIN_ROOT/raven-sdk')
cases = ('success', 'borrowed', 'empty', 'eof', 'read-error', 'overread',
         'cancel', 'precancel', 'disposed', 'invalid-header', 'refused')
parser.add_argument('--case', choices=cases, help='Run one case instead of the full focused matrix')
args = parser.parse_args()
native = args.compiler is not None
if native:
    assert args.core and args.seed and args.ownership and args.native_library
else:
    assert args.toolchain_root
bundle = args.toolchain_root.resolve() if args.toolchain_root else Path('.')
runner = args.runner.resolve()
here = Path(__file__).resolve().parent
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(args.sdk.resolve() if args.sdk else bundle / 'raven-sdk'))
with tempfile.TemporaryDirectory(prefix='neoclr-stream-upload-') as folder:
    root = Path(folder)
    for name in ('Main.rvn', 'Upload.rvnproj'):
        shutil.copyfile(here / name, root / name)
    command = ['dotnet', 'msbuild', str(root / 'Upload.rvnproj'), '-nologo', '-v:minimal']
    if native:
        command = ['dotnet', str(args.compiler.resolve()), 'neoclr', '--core-reference', str(args.core.resolve()),
                   '--runtime-seed', str(args.seed.resolve()), '--bootstrap-intrinsics',
                   '--bootstrap-ownership', str(args.ownership.resolve())]
        for library in args.native_library:
            command += ['--reference', str(library.resolve())]
        command += ['-o', str(root / 'App.dll'), str(root / 'Main.rvn')]
    built = subprocess.run(command,
                           env=env, capture_output=True, text=True, timeout=240)
    assert built.returncode == 0, built.stdout + built.stderr
    app = root / ('App.dll' if native else 'bin/neoclr/Debug/App.neoil')
    for mode in ([args.case] if args.case else cases):
        with socket.socket() as listener:
            listener.bind(('127.0.0.1', 0))
            port = listener.getsockname()[1]
            # A bound, non-listening socket makes the refused case deterministic.
            network = mode not in ('precancel', 'disposed', 'invalid-header', 'refused')
            errors = []
            if network:
                listener.listen(1)
                listener.settimeout(90)

            def peer():
                try:
                    with listener.accept()[0] as connection:
                        connection.settimeout(30)
                        head = bytearray()
                        while not head.endswith(b'\r\n\r\n'):
                            part = connection.recv(1)
                            assert part, 'EOF before headers'
                            head.extend(part)
                            assert len(head) <= 2048
                        length = 0 if mode == 'empty' else 1025
                        assert head.startswith(b'POST /upload HTTP/1.1\r\n'), head
                        assert f'Content-Length: {length}\r\n'.encode() in head, head
                        assert b'Transfer-Encoding:' not in head
                        body = bytearray()
                        if mode in ('success', 'borrowed', 'empty'):
                            while len(body) < length:
                                part = connection.recv(min(113, length - len(body)))
                                assert part, 'Premature upload EOF'
                                body.extend(part)
                            assert body == bytes(i % 251 for i in range(length))
                            connection.sendall(b'HTTP/1.1 201 Created\r\nContent-Length: 0\r\n\r\n')
                        assert connection.recv(1) == b'', 'Unsent/error body leaked or connection remained open'
                except BaseException as error:
                    errors.append(repr(error))

            thread = threading.Thread(target=peer, daemon=True) if network else None
            if thread:
                thread.start()
            command = [str(runner), str(app), str(bundle / 'lib/System.neoil'), '256', '100000000']
            if native:
                command = [str(runner), 'run', str(app), '--system', str(args.seed.resolve()), '--gc-stats', '--instructions', '100000000']
                for library in args.native_library:
                    command += ['--module', str(library.resolve())]
            command += ['--', f'http://127.0.0.1:{port}/upload', mode]
            result = subprocess.run(command,
                                    capture_output=True, text=True, timeout=120)
            if thread:
                thread.join(timeout=35)
                assert not thread.is_alive(), f'{mode}: peer did not finish; {result.stdout} {result.stderr}'
            assert result.returncode == 0 and 'Stream upload checks passed' in result.stdout, (
                mode, result.stdout, result.stderr, errors)
            assert 'live=0 ' in result.stderr, result.stderr
            assert not errors, (mode, errors)
            print(json.dumps({'case': mode, 'outcome': 'passed', 'statistics': result.stderr.strip()}), flush=True)

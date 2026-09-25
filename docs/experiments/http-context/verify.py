"""Independent peers for bounded, explicit HTTP exchange ownership."""
import argparse
import os
from pathlib import Path
import selectors
import shutil
import socket
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
parser.add_argument('--runner', type=Path, required=True)
args = parser.parse_args()
bundle = args.toolchain_root.resolve()
here = Path(__file__).resolve().parent
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
with tempfile.TemporaryDirectory(prefix='neoclr-http-context-') as folder:
    root = Path(folder)
    for name in ('Main.rvn', 'Sample.rvn', 'HttpContext.rvnproj'):
        shutil.copyfile(here / name, root / name)
    build = subprocess.run(['dotnet', 'msbuild', str(root / 'HttpContext.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=240)
    assert build.returncode == 0, build.stdout + build.stderr
    process = subprocess.Popen([str(args.runner.resolve()), str(root / 'bin/neoclr/Debug/App.neoil'), str(bundle / 'lib/System.neoil'), '1024', '100000000', '--live-output'], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        with selectors.DefaultSelector() as selector:
            selector.register(process.stdout, selectors.EVENT_READ)
            assert selector.select(180), 'No server port'
        line = process.stdout.readline().strip()
        assert line.isdecimal(), (line, process.communicate(timeout=10))
        port = int(line)
        def connect(target):
            peer = socket.create_connection(('127.0.0.1', port), timeout=30)
            peer.sendall(f'GET /{target} HTTP/1.1\r\nHost: localhost\r\n\r\n'.encode())
            return peer
        def read(peer):
            result = bytearray()
            while True:
                try:
                    part = peer.recv(4096)
                except ConnectionResetError:
                    break
                if not part:
                    break
                result.extend(part)
                assert len(result) <= 4096
            return bytes(result)
        with connect('first') as first, connect('second') as second:
            for peer, text in ((second, 'Second: Café'), (first, 'First: Café')):
                response = read(peer)
                head, body = response.split(b'\r\n\r\n', 1)
                assert body == text.encode(), response
                assert b'Content-Type: text/plain; charset=utf-8' in head, response
                assert f'Content-Length: {len(body)}'.encode() in head, response
        with connect('sample') as peer:
            assert read(peer).split(b'\r\n\r\n', 1)[1] == b'Requested: /sample'
        with connect('status-only') as peer:
            wire = read(peer)
            assert wire.startswith(b'HTTP/1.1 204 ') and wire.endswith(b'\r\n\r\n') and b'Content-Type:' not in wire, wire
        for target in ('abandon', 'invalid', 'handler-error', 'handler-cancel', 'handler-stop', 'pre-cancelled', 'closing-send', 'shutdown'):
            if target == 'shutdown':
                assert process.stdout.readline().strip() == 'Ready for shutdown'
            with connect(target) as peer:
                wire = read(peer)
                if target != 'closing-send':
                    assert not wire, (target, wire)
        output, errors = process.communicate(timeout=60)
        assert process.returncode == 0 and 'Context lifecycle checks passed' in output and 'live=0' in errors, output + errors
        print(output + errors, flush=True)
    finally:
        if process.poll() is None:
            process.kill()
            process.communicate()

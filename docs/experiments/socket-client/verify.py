#!/usr/bin/env python3
"""Run the Raven TCP client against a bounded, real loopback echo server."""
import argparse
import os
from pathlib import Path
import re
import shutil
import socket
import subprocess
import tempfile
import threading
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
parser.add_argument('--runner', type=Path, required=True)
args = parser.parse_args()
here = Path(__file__).resolve().parent
bundle = args.toolchain_root.resolve()
with tempfile.TemporaryDirectory(prefix='neoclr-socket-client-') as folder, socket.socket() as listener:
    listener.bind(('127.0.0.1', 0))
    listener.listen(1)
    listener.settimeout(90)
    root = Path(folder)
    (root / 'Main.rvn').write_text((here / 'Main.rvn').read_text().replace('19090', str(listener.getsockname()[1])))
    shutil.copyfile(here / 'SocketClient.rvnproj', root / 'SocketClient.rvnproj')
    env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
    build = subprocess.run(['dotnet', 'msbuild', str(root / 'SocketClient.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=150)
    assert build.returncode == 0, build.stdout + build.stderr
    errors = []
    def server():
        try:
            with listener.accept()[0] as peer:
                peer.settimeout(30)
                greeting = bytearray()
                while len(greeting) < 2:
                    part = peer.recv(2 - len(greeting))
                    assert part, 'Guest closed before sending the greeting'
                    greeting.extend(part)
                assert greeting == b'Hi', greeting
                time.sleep(0.1)
                peer.sendall(greeting[:1])
                time.sleep(0.1)
                peer.sendall(greeting[1:])
                peer.shutdown(socket.SHUT_WR)
                assert peer.recv(1) == b'', 'Guest failed to close the connection'
        except BaseException as error:
            errors.append(error)
    thread = threading.Thread(target=server, daemon=True)
    thread.start()
    run = subprocess.run([str(args.runner.resolve()), str(root / 'bin/neoclr/Debug/App.neoil'), str(bundle / 'lib/System.neoil'), '256'], capture_output=True, text=True, timeout=90)
    assert run.returncode == 0, run.stdout + run.stderr
    thread.join(5)
    assert not thread.is_alive() and not errors, errors
    assert run.stdout == 'Other work runs while TCP is pending\nResolved localhost\nSent Hi\nReceived Hi; peer finished sending\nSocket closed\n', run.stdout
    stats = {key: int(value) for key, value in re.findall(r'(\w+)=(\d+)', run.stderr)}
    assert stats['live'] == 0 and stats['collections'] > 1, stats
    print(run.stdout, end='')
    print(run.stderr, end='')
    for name, code in [
        ('opaque constructor', 'let socket = Socket(1L)'),
        ('private DNS completion', 'let completion = System.Networking.DnsCompletion(Promise<Result<System.Collections.Sequence<string>, System.Networking.DnsError>>())'),
        ('private completion type', 'let completion = SocketConnectCompletion(Promise<Result<Socket, SocketError>>())'),
    ]:
        (root / 'Main.rvn').write_text('import System.*\nimport System.Tasks.*\nimport System.Networking.Sockets.*\nfunc Main() {\n' + code + '\n}\n')
        build = subprocess.run(['dotnet', 'msbuild', str(root / 'SocketClient.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=90)
        diagnostics = build.stdout + build.stderr
        assert build.returncode != 0 and 'error RAV' in diagnostics and 'Unhandled exception' not in diagnostics, diagnostics
        print(name + ': rejected')

#!/usr/bin/env python3
"""Run the Raven TCP client against a bounded, real loopback echo server."""
import argparse
import os
import json
from pathlib import Path
import re
import shutil
import socket
import subprocess
import tempfile
import threading
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bundle', type=Path, required=True)
args = parser.parse_args()
here = Path(__file__).resolve().parent
bundle = args.bundle.resolve()
with tempfile.TemporaryDirectory(prefix='neoclr-socket-client-') as folder, socket.socket() as listener:
    listener.bind(('127.0.0.1', 0))
    listener.listen(1)
    listener.settimeout(90)
    root = Path(folder)
    (root / 'Main.rvn').write_text((here / 'Main.rvn').read_text().replace('19090', str(listener.getsockname()[1])))
    shutil.copyfile(here / 'SocketClient.rvnproj', root / 'SocketClient.rvnproj')
    env = dict(os.environ, NeoClrBundleRoot=str(bundle))
    compile_command = ['dotnet', str(bundle / 'sdk/tools/rvnc/rvnc.dll'), 'neoclr', '--project', str(root / 'SocketClient.rvnproj')]
    build = subprocess.run(compile_command, env=env, capture_output=True, text=True, timeout=150)
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
    assemblies = [line.removeprefix('Native build output: ') for line in build.stdout.splitlines()
                  if line.startswith('Native build output: ')]
    assert len(assemblies) == 1, build.stdout
    lib = bundle / 'lib'
    catalog = json.loads((lib / 'bundle.json').read_text())
    command = [str(bundle / 'bin' / ('neoclr.exe' if os.name == 'nt' else 'neoclr')),
               'run', assemblies[0], '--system', str(lib / catalog['runtimeSeed'])]
    for name in catalog['assemblyNames']:
        command += ['--module', str(lib / (name + '.dll'))]
    command += ['--object-root', str(lib / 'System.Runtime.dll'), '--execution', 'interpreter']
    run = subprocess.run(command, capture_output=True, text=True, timeout=90)
    assert run.returncode == 0, run.stdout + run.stderr
    thread.join(5)
    assert not thread.is_alive() and not errors, errors
    assert run.stdout == 'Other work runs while TCP is pending\nResolved localhost\nSent Hi\nReceived Hi; peer finished sending\nSocket closed\n', run.stdout
    assert run.stderr == '', run.stderr
    print(run.stdout, end='')
    for name, code in [
        ('opaque constructor', 'let socket = Socket(1L)'),
        ('private DNS completion', 'let completion = System.Networking.DnsCompletion(Promise<Result<System.Collections.Sequence<string>, System.Networking.DnsError>>())'),
        ('private completion type', 'let completion = SocketConnectCompletion(Promise<Result<Socket, SocketError>>())'),
    ]:
        (root / 'Main.rvn').write_text('import System.*\nimport System.Concurrency.Tasks.*\nimport System.Networking.Sockets.*\nfunc Main() {\n' + code + '\n}\n')
        build = subprocess.run(compile_command, env=env, capture_output=True, text=True, timeout=90)
        diagnostics = build.stdout + build.stderr
        assert build.returncode != 0 and 'error RAV' in diagnostics and 'Unhandled exception' not in diagnostics, diagnostics
        print(name + ': rejected')

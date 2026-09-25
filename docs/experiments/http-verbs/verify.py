"""Focused PUT/PATCH/DELETE client and server interoperability."""
import argparse
import os
from pathlib import Path
import selectors
import shutil
import socket
import subprocess
import tempfile
import threading

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
parser.add_argument('--runner', type=Path, required=True)
args = parser.parse_args()
bundle, runner = args.toolchain_root.resolve(), args.runner.resolve()
here = Path(__file__).resolve().parent
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
bodies = ['Café'.encode(), bytes([0, 255]), b'', b'']
methods = ['PUT', 'PATCH', 'DELETE', 'HEAD']

def build(root, source):
    root.mkdir()
    (root / 'Main.rvn').write_text(source)
    for name in ('Sample.rvn', 'HttpVerbs.rvnproj'):
        shutil.copyfile(here / name, root / name)
    result = subprocess.run(['dotnet', 'msbuild', str(root / 'HttpVerbs.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=240)
    assert result.returncode == 0, result.stdout + result.stderr
    return root / 'bin/neoclr/Debug/App.neoil'

def command(app, live=False):
    return [str(runner), str(app), str(bundle / 'lib/System.neoil'), '256', '100000000'] + (['--live-output'] if live else [])

with tempfile.TemporaryDirectory(prefix='neoclr-http-verbs-') as temporary, socket.socket() as listener:
    root = Path(temporary)
    listener.bind(('127.0.0.1', 0))
    listener.listen()
    listener.settimeout(180)
    client = build(root / 'client', (here / 'Client.rvn').read_text().replace('19091', str(listener.getsockname()[1])))
    server = build(root / 'server', (here / 'Server.rvn').read_text())
    reference = root / 'reference'
    reference.mkdir()
    shutil.copyfile(here / 'Reference.cs', reference / 'Program.cs')
    (reference / 'Reference.csproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>Exe</OutputType><TargetFramework>net10.0</TargetFramework></PropertyGroup></Project>')
    compiled = subprocess.run(['dotnet', 'build', str(reference / 'Reference.csproj'), '-v:q'], capture_output=True, text=True, timeout=120)
    assert compiled.returncode == 0, compiled.stdout + compiled.stderr
    failures = []
    def peer():
        try:
            for index, expected in enumerate(bodies):
                connection, _ = listener.accept()
                with connection:
                    connection.settimeout(35)
                    head = bytearray()
                    while not head.endswith(b'\r\n\r\n'):
                        part = connection.recv(1)
                        assert part
                        head.extend(part)
                        assert len(head) <= 2048
                    lines = bytes(head).decode('ascii').split('\r\n')
                    assert lines[0] == methods[index] + ' /item HTTP/1.1', lines
                    fields = {}
                    for line in lines[1:-2]:
                        name, value = line.split(':', 1)
                        fields.setdefault(name.lower(), []).append(value.strip())
                    if index < 2:
                        assert fields['content-length'] == [str(len(expected))], fields
                        assert fields['content-type'] == [('text/plain; charset=utf-8' if index == 0 else 'application/octet-stream')], fields
                    else:
                        assert 'content-length' not in fields and 'content-type' not in fields, fields
                    body = bytearray()
                    while len(body) < len(expected):
                        part = connection.recv(len(expected)-len(body))
                        assert part
                        body.extend(part)
                    assert body == expected, body
                    response = b'HTTP/1.1 204 No Content\r\n\r\n' if index == 2 else f'HTTP/1.1 200 OK\r\nContent-Length: {len(body)}\r\n\r\n'.encode() + body
                    if index == 3:
                        response = b'HTTP/1.1 200 OK\r\nContent-Length: 999999\r\n\r\n'
                    connection.sendall(response)
                    assert connection.recv(1) == b'', 'Client waited for EOF'
        except BaseException as error:
            failures.append(error)
    worker = threading.Thread(target=peer, daemon=True)
    worker.start()
    result = subprocess.run(command(client), capture_output=True, text=True, timeout=240)
    assert result.returncode == 0 and 'Verb client checks passed' in result.stdout and 'live=0' in result.stderr, result.stdout + result.stderr
    worker.join(40)
    assert not worker.is_alive() and not failures, failures
    print(result.stdout + result.stderr, flush=True)
    process = subprocess.Popen(command(server, True), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        with selectors.DefaultSelector() as selector:
            selector.register(process.stdout, selectors.EVENT_READ)
            assert selector.select(180), 'Server did not report a port'
        port = int(process.stdout.readline().strip())
        baseline = subprocess.run(['dotnet', str(reference / 'bin/Debug/net10.0/Reference.dll'), str(port)], capture_output=True, text=True, timeout=120)
        assert baseline.returncode == 0, baseline.stdout + baseline.stderr
        print(baseline.stdout, flush=True)
        for request in [b'DELETE /item HTTP/1.1\r\nHost: localhost\r\nContent-Length: 1\r\n\r\nx', b'OPTIONS /item HTTP/1.1\r\nHost: localhost\r\n\r\n']:
            with socket.create_connection(('127.0.0.1', port), timeout=30) as connection:
                connection.sendall(request)
                connection.shutdown(socket.SHUT_WR)
                try:
                    assert connection.recv(1) == b'', 'Rejected request produced a response'
                except ConnectionResetError:
                    pass
        output, errors = process.communicate(timeout=120)
        assert process.returncode == 0 and 'Verb server checks passed' in output and 'live=0' in errors, output + errors
        assert output.count('Served') == 4 and 'Rejected: DELETE request bodies unsupported' in output and 'Rejected: Unsupported request method' in output, output
        print(output + errors, flush=True)
    finally:
        if process.poll() is None:
            process.kill()
            process.communicate()

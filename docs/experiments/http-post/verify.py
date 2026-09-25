"""Bounded POST, byte framing, handler/token and independent peer checks."""
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
parser.add_argument('--client-only', action='store_true', help='Run propagation/handler checks and the independent client peer only')
args = parser.parse_args()
bundle, runner = args.toolchain_root.resolve(), args.runner.resolve()
here = Path(__file__).resolve().parent
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
bodies = ['Café 🌍'.encode(), bytes([0,255,13,10]), b'', b'']

def build(root, source):
    root.mkdir()
    (root / 'Main.rvn').write_text(source)
    shutil.copyfile(here / 'HttpPost.rvnproj', root / 'HttpPost.rvnproj')
    shutil.copyfile(here / 'Sample.rvn', root / 'Sample.rvn')
    built = subprocess.run(['dotnet', 'msbuild', str(root / 'HttpPost.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=240)
    assert built.returncode == 0, built.stdout + built.stderr
    return root / 'bin/neoclr/Debug/App.neoil'

def command(app, live=False):
    return [str(runner), str(app), str(bundle / 'lib/System.neoil'), '256', '100000000'] + (['--live-output'] if live else [])

def read_head(peer):
    data = bytearray()
    while not data.endswith(b'\r\n\r\n'):
        part = peer.recv(1)
        assert part, 'EOF before headers'
        data.extend(part)
        assert len(data) <= 2048
    return bytes(data)

def run_client(app):
    result = subprocess.run(command(app), capture_output=True, text=True, timeout=240)
    assert result.returncode == 0 and 'POST echo checks passed' in result.stdout and 'live=0' in result.stderr, result.stdout + result.stderr
    print(result.stdout + result.stderr, flush=True)

with tempfile.TemporaryDirectory(prefix='neoclr-post-') as folder, socket.socket() as listener:
    root = Path(folder)
    listener.bind(('127.0.0.1', 0))
    listener.listen(4)
    listener.settimeout(120)
    port = listener.getsockname()[1]
    client = build(root / 'client', (here / 'Client.rvn').read_text().replace('19091', str(port)))
    sample = build(root / 'sample', (here / 'SampleChecks.rvn').read_text())
    checked = subprocess.run(command(sample), capture_output=True, text=True, timeout=120)
    assert checked.returncode == 0 and 'Propagating POST sample checks passed' in checked.stdout and 'live=0' in checked.stderr, checked.stdout + checked.stderr
    print(checked.stdout + checked.stderr, flush=True)
    server = None if args.client_only else build(root / 'server', (here / 'Server.rvn').read_text())
    reference = root / 'reference'
    reference.mkdir()
    shutil.copyfile(here / 'Reference.cs', reference / 'Program.cs')
    (reference / 'Reference.csproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><ImplicitUsings>enable</ImplicitUsings></PropertyGroup></Project>')
    built = subprocess.run(['dotnet', 'build', str(reference / 'Reference.csproj'), '-v:q'], capture_output=True, text=True, timeout=120)
    assert built.returncode == 0, built.stdout + built.stderr
    errors = []
    def raw_server():
        try:
            for index, expected in enumerate(bodies):
                with listener.accept()[0] as peer:
                    peer.settimeout(30)
                    head = read_head(peer)
                    assert head.startswith(b'GET /empty HTTP/1.1\r\n' if index == 3 else b'POST /echo HTTP/1.1\r\n'), head
                    if index != 3:
                        assert f'Content-Length: {len(expected)}\r\n'.encode() in head, head
                    if index == 0:
                        assert b'Content-Type: text/plain; charset=utf-8\r\n' in head
                    elif index == 1:
                        assert b'Content-Type:' not in head
                    body = bytearray()
                    while len(body) < len(expected):
                        part = peer.recv(len(expected) - len(body))
                        assert part
                        body.extend(part)
                    assert bytes(body) == expected, body
                    code = 200 if index == 3 else 201
                    peer.sendall(f'HTTP/1.1 {code} \r\nContent-Length: {len(body)}\r\n\r\n'.encode() + body)
                    assert peer.recv(1) == b'', 'Client did not finish before EOF'
        except BaseException as error:
            errors.append(error)
    worker = threading.Thread(target=raw_server, daemon=True)
    worker.start()
    run_client(client)
    worker.join(35)
    assert not worker.is_alive() and not errors, errors
    print('Independent peer verified POST bytes and framing', flush=True)

    if args.client_only:
        raise SystemExit(0)
    process = subprocess.Popen(command(server, True), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        with selectors.DefaultSelector() as selector:
            selector.register(process.stdout, selectors.EVENT_READ)
            assert selector.select(120), 'Server did not report a port'
        server_port = int(process.stdout.readline().strip())
        integrated = build(root / 'integrated', (here / 'Client.rvn').read_text().replace('19091', str(server_port)))
        run_client(integrated)
        baseline = subprocess.run(['dotnet', str(reference / 'bin/Debug/net10.0/Reference.dll'), str(server_port)], capture_output=True, text=True, timeout=120)
        assert baseline.returncode == 0, baseline.stdout + baseline.stderr
        print(baseline.stdout, flush=True)
        cases = [
            (b'POST /echo HTTP/1.1\r\nHost: localhost\r\nContent-Length: 4\r\n\r\n\x00\xff\r\n', True),
            (b'GET / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 1\r\n\r\nx', False),
            (b'POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\nContent-Length: 0\r\n\r\n', False),
            (b'POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\nTransfer-Encoding: chunked\r\n\r\n', False),
            (b'POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: -1\r\n\r\n', False),
            (b'POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 1025\r\n\r\n', False),
            (b'POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 4\r\n\r\nxx', False),
            (b'POST / HTTP/1.1\r\nHost: localhost\r\nExpect: 100-continue\r\nContent-Length: 0\r\n\r\n', False),
            (b'POST / HTTP/1.1\r\nHost: localhost\r\nContent-Encoding: gzip\r\nContent-Length: 0\r\n\r\n', False),
        ]
        for wire, valid in cases:
            with socket.create_connection(('127.0.0.1', server_port), timeout=30) as peer:
                if valid:
                    for offset in range(0, len(wire), 3):
                        peer.sendall(wire[offset:offset+3])
                else:
                    peer.sendall(wire)
                peer.shutdown(socket.SHUT_WR)
                reply = bytearray()
                while True:
                    try:
                        part = peer.recv(4096)
                    except ConnectionResetError:
                        assert not valid
                        break
                    if not part: break
                    reply.extend(part)
                if valid:
                    head, body = bytes(reply).split(b'\r\n\r\n')
                    assert head.startswith(b'HTTP/1.1 201 ') and body == bytes([0,255,13,10]), reply
                else:
                    assert not reply, reply
        out, err = process.communicate(timeout=60)
        assert process.returncode == 0 and out.count('Served') == 8 and out.count('Rejected:') == 8 and 'POST server checks passed' in out and 'live=0' in err, out + err
        print(out + err, flush=True)
    finally:
        if process.poll() is None:
            process.kill()
            process.communicate()

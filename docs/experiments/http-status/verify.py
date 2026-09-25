"""Focused HTTP status/framing and Raven propagation-conversion checks."""
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
bundle = args.toolchain_root.resolve()
runner = args.runner.resolve()
here = Path(__file__).resolve().parent
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
codes = [201, 204, 205, 304, 400, 404, 500, 599]
invalid = [b'HTTP/1.1 20A Bad\r\nContent-Length: 0\r\n\r\n',
           b'HTTP/1.1 600 Bad\r\nContent-Length: 0\r\n\r\n',
           b'HTTP/1.1 100 Continue\r\n\r\n',
           b'HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n',
           b'HTTP/1.1 205 Reset Content\r\nContent-Length: 1\r\n\r\nx',
           b'HTTP/1.1 201 Created\r\n\r\n']

def build(root, source):
    root.mkdir()
    (root / 'Main.rvn').write_text(source)
    shutil.copyfile(here / 'HttpStatus.rvnproj', root / 'HttpStatus.rvnproj')
    result = subprocess.run(['dotnet', 'msbuild', str(root / 'HttpStatus.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=240)
    assert result.returncode == 0, result.stdout + result.stderr
    return root / 'bin/neoclr/Debug/App.neoil'

def command(app, live=False):
    return [str(runner), str(app), str(bundle / 'lib/System.neoil'), '256', '100000000'] + (['--live-output'] if live else [])

with tempfile.TemporaryDirectory(prefix='neoclr-http-status-') as folder, socket.socket() as listener:
    root = Path(folder)
    listener.bind(('127.0.0.1', 0))
    listener.listen(1)
    listener.settimeout(150)
    port = listener.getsockname()[1]
    app = build(root / 'client', (here / 'Main.rvn').read_text().replace('19091', str(port)))
    server_app = build(root / 'server', (here / 'Server.rvn').read_text())
    ref = root / 'reference'
    ref.mkdir()
    shutil.copyfile(here / 'Reference.cs', ref / 'Program.cs')
    (ref / 'Reference.csproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><ImplicitUsings>enable</ImplicitUsings></PropertyGroup></Project>')
    built = subprocess.run(['dotnet', 'build', str(ref / 'Reference.csproj'), '-v:q'], capture_output=True, text=True, timeout=120)
    assert built.returncode == 0, built.stdout + built.stderr

    def exchange(cmd, include_invalid):
        errors = []
        def peer():
            try:
                targets = [str(code) for code in codes for _ in range(2)] + (['invalid-' + str(i) for i in range(len(invalid))] if include_invalid else [])
                for target in targets:
                    with listener.accept()[0] as stream:
                        stream.settimeout(30)
                        request = bytearray()
                        while not request.endswith(b'\r\n\r\n'):
                            part = stream.recv(1)
                            assert part, 'Early request EOF'
                            request.extend(part)
                            assert len(request) < 2048
                        assert request.startswith(f'GET /{target} HTTP/1.1\r\n'.encode()), request
                        if target.startswith('invalid-'):
                            wire = invalid[int(target[8:])]
                        else:
                            code = int(target)
                            length = b'' if code == 204 else b'Content-Length: 65536\r\n' if code == 304 else b'Content-Length: 0\r\n' if code == 205 else b'Content-Length: 4\r\n'
                            wire = f'HTTP/1.1 {code} \r\n'.encode() + length + b'Connection: close\r\n\r\n' + (b'' if code in (204,205,304) else b'body')
                        stream.sendall(wire)
                        try:
                            assert stream.recv(1) == b'', 'Client did not complete before EOF'
                        except ConnectionResetError:
                            assert target.startswith('invalid-')
            except BaseException as error:
                errors.append(error)
        worker = threading.Thread(target=peer, daemon=True)
        worker.start()
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=240)
        assert result.returncode == 0, result.stdout + result.stderr
        worker.join(35)
        assert not worker.is_alive() and not errors, (errors, result.stdout, result.stderr)
        return result
    baseline = exchange(['dotnet', str(ref / 'bin/Debug/net10.0/Reference.dll'), str(port)], False)
    print(baseline.stdout, flush=True)
    result = exchange(command(app), True)
    assert 'HTTP status and conversion checks passed' in result.stdout and 'live=0' in result.stderr, result.stdout + result.stderr
    print(result.stdout + result.stderr, flush=True)

    process = subprocess.Popen(command(server_app, True), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        with selectors.DefaultSelector() as selector:
            selector.register(process.stdout, selectors.EVENT_READ)
            assert selector.select(120), 'Server did not print its port'
        server_port = int(process.stdout.readline().strip())
        for target in [str(code) for code in codes] + ['bad-body', 'bad-code']:
            with socket.create_connection(('127.0.0.1', server_port), timeout=30) as peer:
                peer.sendall(f'GET /{target} HTTP/1.1\r\nHost: localhost\r\n\r\n'.encode())
                response = bytearray()
                while True:
                    part = peer.recv(4096)
                    if not part: break
                    response.extend(part)
                if target.startswith('bad-'):
                    assert not response, response
                    continue
                code = int(target)
                head, body = bytes(response).split(b'\r\n\r\n')
                assert head.startswith(f'HTTP/1.1 {code} '.encode()), head
                if code in (204,304):
                    assert b'Content-Length:' not in head and not body, response
                else:
                    expected = b'' if code == 205 else b'body'
                    assert body == expected and f'Content-Length: {len(expected)}'.encode() in head, response
        out, err = process.communicate(timeout=60)
        assert process.returncode == 0 and 'HTTP statuses served' in out and out.count('Rejected response') == 2 and 'live=0' in err, out + err
        print(out + err, flush=True)
    finally:
        if process.poll() is None:
            process.kill()
            process.communicate()

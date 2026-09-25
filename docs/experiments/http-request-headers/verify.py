"""Focused request header construction and independent wire checks."""
import argparse
import os
from pathlib import Path
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
with tempfile.TemporaryDirectory(prefix='neoclr-request-headers-') as temporary, socket.socket() as listener:
    root = Path(temporary)
    listener.bind(('127.0.0.1', 0))
    listener.listen()
    listener.settimeout(300)
    port = listener.getsockname()[1]
    (root / 'Main.rvn').write_text((here / 'Main.rvn').read_text().replace('19091', str(port)))
    for name in ('Sample.rvn', 'HttpRequestHeaders.rvnproj'):
        shutil.copyfile(here / name, root / name)
    built = subprocess.run(['dotnet', 'msbuild', str(root / 'HttpRequestHeaders.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=240)
    assert built.returncode == 0, built.stdout + built.stderr
    failures = []
    def peer():
        try:
            for index in range(2):
                connection, _ = listener.accept()
                with connection:
                    connection.settimeout(35)
                    head = bytearray()
                    while not head.endswith(b'\r\n\r\n'):
                        part = connection.recv(1)
                        assert part, 'EOF before request headers'
                        head.extend(part)
                        assert len(head) <= 2048
                    lines = bytes(head).decode('ascii').split('\r\n')
                    assert lines[0] == ('POST /report HTTP/1.1' if index == 0 else 'GET /read HTTP/1.1'), lines
                    headers = {}
                    for line in lines[1:-2]:
                        name, value = line.split(':', 1)
                        headers.setdefault(name.lower(), []).append(value.strip())
                    assert headers['host'] == [f'127.0.0.1:{port}'], headers
                    assert headers['connection'] == ['close'], headers
                    assert headers['x-request-id'] == [('sample-42' if index == 0 else 'get-42')], headers
                    if index == 0:
                        assert headers['accept'] == ['text/plain'], headers
                        assert headers['content-type'] == ['text/plain; charset=utf-8'], headers
                        assert headers['content-length'] == ['5'], headers
                        body = bytearray()
                        while len(body) < 5:
                            part = connection.recv(5-len(body))
                            assert part
                            body.extend(part)
                        assert bytes(body) == 'Café'.encode(), body
                    else:
                        assert 'content-length' not in headers and 'content-type' not in headers, headers
                    connection.sendall(b'HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n')
                    assert connection.recv(1) == b'', 'Client did not close before EOF'
        except BaseException as error:
            failures.append(error)
    worker = threading.Thread(target=peer, daemon=True)
    worker.start()
    result = subprocess.run([str(runner), str(root / 'bin/neoclr/Debug/App.neoil'), str(bundle / 'lib/System.neoil'), '256', '100000000'], capture_output=True, text=True, timeout=300)
    assert result.returncode == 0, result.stdout + result.stderr
    assert 'Request header contracts passed' in result.stdout and 'Request header wire checks passed' in result.stdout, result.stdout + result.stderr
    assert 'live=0' in result.stderr, result.stderr
    worker.join(40)
    assert not worker.is_alive() and not failures, failures
    print(result.stdout + result.stderr)
    print('Independent peer verified GET and POST application headers')

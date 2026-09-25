#!/usr/bin/env python3
"""Compile the Raven client once; exercise framing, peer lifetime and a .NET baseline."""
import argparse
import http.server
import os
from pathlib import Path
import re
import select
import shutil
import socket
import subprocess
import tempfile
import threading
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
parser.add_argument('--runner', type=Path, required=True)
parser.add_argument('--case', action='append', help='Select named Raven cases; the .NET baseline still runs')
args = parser.parse_args()
here = Path(__file__).resolve().parent
bundle = args.toolchain_root.resolve()
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
body = 'Café 🌍'.encode('utf-8')
head = b'HTTP/1.1 200 OK\r\nContent-Length: 10\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n'

with tempfile.TemporaryDirectory(prefix='neoclr-http-client-') as folder, socket.socket() as listener:
    root = Path(folder)
    listener.bind(('127.0.0.1', 0))
    listener.listen(1)
    listener.settimeout(120)
    port = listener.getsockname()[1]
    for name in ['Handlers.rvn', 'HttpClient.rvnproj']:
        shutil.copyfile(here / name, root / name)
    (root / 'Main.rvn').write_text((here / 'Main.rvn').read_text().replace('19091', str(port)))
    build = subprocess.run(['dotnet', 'msbuild', str(root / 'HttpClient.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=150)
    assert build.returncode == 0, build.stdout + build.stderr
    reference = root / 'reference'
    reference.mkdir()
    shutil.copyfile(here / 'Reference.cs', reference / 'Program.cs')
    (reference / 'Reference.csproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><ImplicitUsings>enable</ImplicitUsings></PropertyGroup></Project>')
    build = subprocess.run(['dotnet', 'build', str(reference / 'Reference.csproj'), '-v:q', '-p:WarningLevel=0'], capture_output=True, text=True, timeout=120)
    assert build.returncode == 0, build.stdout + build.stderr

    def exchange(command, fragments, truncate=False, allow_reset=False):
        errors = []
        def serve():
            try:
                with listener.accept()[0] as peer:
                    peer.settimeout(60)
                    request = bytearray()
                    while not request.endswith(b'\r\n\r\n'):
                        part = peer.recv(1)
                        assert part, 'EOF before request headers'
                        request.extend(part)
                        assert len(request) <= 1024
                    lines = bytes(request).split(b'\r\n')
                    assert lines[0] == b'GET /greeting HTTP/1.1', request
                    assert f'Host: localhost:{port}'.encode() in lines, request
                    assert b'Connection: close' in lines, request
                    for fragment in fragments:
                        if isinstance(fragment, tuple):
                            data, delay = fragment
                            try:
                                peer.sendall(data)
                                if select.select([peer], [], [], delay)[0]:
                                    assert peer.recv(1) == b'', 'Unexpected request bytes'
                                    return
                            except (BrokenPipeError, ConnectionResetError):
                                assert allow_reset, 'Unexpected close while sending a valid response'
                                return
                        else:
                            peer.sendall(fragment)
                            time.sleep(0.002)
                    if truncate:
                        peer.shutdown(socket.SHUT_WR)
                    # In the valid case keep the write direction open: completion
                    # must follow Content-Length rather than awaiting peer EOF.
                    try:
                        assert peer.recv(1) == b'', 'Client did not close after its result'
                    except ConnectionResetError:
                        assert allow_reset, 'Unexpected reset on a valid response'
            except BaseException as error:
                errors.append(error)
        thread = threading.Thread(target=serve, daemon=True)
        thread.start()
        run = subprocess.run(command, capture_output=True, text=True, timeout=180)
        assert run.returncode == 0, run.stdout + run.stderr
        thread.join(65)
        assert not thread.is_alive() and not errors, (errors, run.stdout, run.stderr)
        return run

    fragments = [bytes([value]) for value in head + body]
    baseline = exchange(['dotnet', str(reference / 'bin/Debug/net10.0/Reference.dll'), str(port)], fragments)
    assert baseline.stdout == 'HTTP 200\nCafé 🌍\n', baseline.stdout
    print('.NET: Content-Length completes before EOF; UTF-8 body matches', flush=True)
    command = [str(args.runner.resolve()), str(root / 'bin/neoclr/Debug/App.neoil'), str(bundle / 'lib/System.neoil'), '256', '10000000']
    prefix = 'Handler checks passed\nOther work runs while HTTP is pending\n'
    cases = [
        ('trickling body', [head] + [(bytes([value]), 3) for value in body], False, 'HTTP error: Request deadline exceeded\n'),
        ('trickling headers', [(bytes([value]), 1) for value in head], False, 'HTTP error: Request deadline exceeded\n'),
        ('stalled headers', [], False, 'HTTP error: TimedOut\n'),
        ('stalled body', [head, body[:1]], False, 'HTTP error: TimedOut\n'),
        ('fragmented UTF-8', fragments, False, 'HTTP 200\nCafé 🌍\n'),
        ('truncated body', [head, body[:-1]], True, 'HTTP error: EOF before complete response\n'),
        ('ambiguous framing', [b'HTTP/1.1 200 OK\r\nContent-Length: 0\r\ncontent-length: 1\r\n\r\n'], False, 'HTTP error: Duplicate Content-Length\n'),
    ]
    cases.extend([
        ('chunked UTF-8', [bytes([value]) for value in b'HTTP/1.1 200 OK\r\nTransfer-Encoding: ChUnKeD\r\n\r\n3\r\n' + body[:3] + b'\r\n7\r\n' + body[3:] + b'\r\n0\r\n\r\n'], False, 'HTTP 200\nCafé 🌍\n'),
        ('close-delimited UTF-8', [b'HTTP/1.1 200 OK\r\n\r\n' + body], True, 'HTTP 200\nCafé 🌍\n'),
        ('truncated chunk', [b'HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nx'], True, 'HTTP error: EOF before complete response\n'),
        ('invalid chunk size', [b'HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\nZ'], False, 'HTTP error: Invalid chunk size\n'),
        ('chunk body limit', [b'HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n401'], False, 'HTTP error: Body limit exceeded\n'),
        ('close body limit', [b'HTTP/1.1 200 OK\r\n\r\n' + b'x' * 1025], True, 'HTTP error: Body limit exceeded\n'),
        ('chunk extensions', [b'HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n1;'], False, 'HTTP error: Chunk extensions unsupported\n'),
        ('chunk trailers', [b'HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n0\r\nX'], False, 'HTTP error: Response trailers unsupported\n'),
        ('chunk 205 content', [b'HTTP/1.1 205 Reset Content\r\nTransfer-Encoding: chunked\r\n\r\n1\r\n'], False, 'HTTP error: Status 205 cannot contain content\n'),
        ('empty body', [b'HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n'], False, 'HTTP 200\n\n'),
        ('mixed case and whitespace', [b'HTTP/1.1 200 OK\r\ncOnTeNt-LeNgTh:\t10 \t\r\n\r\n' + body], False, 'HTTP 200\nCafé 🌍\n'),
        ('missing length', [b'HTTP/1.1 200 OK\r\n\r\n'], True, 'HTTP 200\n\n'),
        ('invalid 204 length', [b'HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n'], False, 'HTTP error: Content-Length is prohibited for status 204\n'),
        ('bare LF', [b'HTTP/1.1 200 OK\n'], False, 'HTTP error: Bare LF in headers\n'),
        ('bare CR', [b'HTTP/1.1 200 OK\rX'], False, 'HTTP error: Bare CR in headers\n'),
        ('negative length', [b'HTTP/1.1 200 OK\r\nContent-Length: -1\r\n\r\n'], False, 'HTTP error: Invalid Content-Length\n'),
        ('overflowing length', [b'HTTP/1.1 200 OK\r\nContent-Length: 999999999999999\r\n\r\n'], False, 'HTTP error: Body limit exceeded\n'),
        ('body limit', [b'HTTP/1.1 200 OK\r\nContent-Length: 1025\r\n\r\n'], False, 'HTTP error: Body limit exceeded\n'),
        ('transfer encoding', [b'HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Length: 0\r\n\r\n'], False, 'HTTP error: Ambiguous response framing\n'),
        ('content encoding', [b'HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nContent-Length: 0\r\n\r\n'], False, 'HTTP error: Unsupported encoding\n'),
        ('invalid field name', [b'HTTP/1.1 200 OK\r\nContent-Length : 0\r\n\r\n'], False, 'HTTP error: Invalid header name\n'),
        ('header count', [b'HTTP/1.1 200 OK\r\nContent-Length: 0\r\n' + b'X: y\r\n' * 16 + b'\r\n'], False, 'HTTP error: Header count exceeded\n'),
        ('header bytes', [b'HTTP/1.1 200 OK\r\nX: ' + b'x' * 2048], False, 'HTTP error: Header limit exceeded\n'),
        ('invalid UTF-8', [b'HTTP/1.1 200 OK\r\nContent-Length: 1\r\n\r\n' + bytes([255])], False, 'HTTP error: Invalid UTF-8 body\n'),
    ])
    unknown = set(args.case or []) - {case[0] for case in cases} - {'independent server'}
    assert not unknown, 'Unknown HTTP cases: ' + ', '.join(sorted(unknown))
    for name, parts, truncate, expected in cases:
        if args.case and name not in args.case:
            continue
        run = exchange(command, parts, truncate, allow_reset=expected.startswith('HTTP error:'))
        assert run.stdout == prefix + expected, run.stdout
        stats = {key: int(value) for key, value in re.findall(r'(\w+)=(\d+)', run.stderr)}
        assert stats['live'] == 0 and stats['collections'] > 1, stats
        print(name + ': ' + expected.strip(), flush=True)
        print('GC:', stats, flush=True)

    if args.case and 'independent server' not in args.case:
        raise SystemExit(0)

    class IndependentHandler(http.server.BaseHTTPRequestHandler):
        protocol_version = 'HTTP/1.1'

        def do_GET(self):
            assert self.path == '/greeting', self.path
            self.send_response(200)
            self.send_header('Content-Length', str(len(body)))
            self.send_header('Content-Type', 'text/plain; charset=utf-8')
            self.send_header('Connection', 'close')
            self.end_headers()
            self.wfile.write(body)
            self.close_connection = True

        def log_message(self, *_):
            pass

    # Reuse the selected port so the compiled app remains identical. This peer uses
    # Python's HTTP server implementation, not the hand-written response framing.
    server = http.server.HTTPServer(('127.0.0.1', port), IndependentHandler, bind_and_activate=False)
    server.socket.close()
    server.socket = listener
    server.timeout = 120
    server_errors = []
    server.handle_error = lambda *_: server_errors.append('Independent server request failed')
    thread = threading.Thread(target=server.handle_request, daemon=True)
    thread.start()
    run = subprocess.run(command, capture_output=True, text=True, timeout=180)
    thread.join(5)
    assert not thread.is_alive() and not server_errors, (server_errors, run.stdout, run.stderr)
    assert run.returncode == 0 and run.stdout == prefix + 'HTTP 200\nCafé 🌍\n', run.stdout + run.stderr
    stats = {key: int(value) for key, value in re.findall(r'(\w+)=(\d+)', run.stderr)}
    assert stats['live'] == 0 and stats['collections'] > 1, stats
    print('Python HTTP server: HTTP 200, matching UTF-8 body; GC:', stats, flush=True)

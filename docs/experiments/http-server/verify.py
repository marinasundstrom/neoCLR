#!/usr/bin/env python3
"""Verify managed HTTP serving with .NET, a neoCLR client and controlled raw requests."""
import argparse
import os
from pathlib import Path
import re
import selectors
import shutil
import socket
import subprocess
import tempfile
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
parser.add_argument('--runner', type=Path, required=True)
selection = parser.add_mutually_exclusive_group()
selection.add_argument('--responses-only', action='store_true', help='Run only response validation and handler-error cases')
selection.add_argument('--requests-only', action='store_true', help='Run only interoperability and request validation cases')
parser.add_argument('--case', action='append', help='Run only the named case; repeat for multiple cases')
args = parser.parse_args()
here = Path(__file__).resolve().parent
client_sources = here if (here / 'Main.rvn').exists() else here.parent / 'http-client'
bundle = args.toolchain_root.resolve()
runner = args.runner.resolve()
runtime = bundle / 'lib/System.neoil'
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
body = 'Café 🌍'.encode('utf-8')


def build(project):
    run = subprocess.run(['dotnet', 'msbuild', str(project), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=180)
    assert run.returncode == 0, run.stdout + run.stderr


def stats(stderr):
    result = {key: int(value) for key, value in re.findall(r'(\w+)=(\d+)', stderr)}
    assert result['live'] == 0 and result['collections'] > 1, stderr
    return result


with tempfile.TemporaryDirectory(prefix='neoclr-http-server-') as folder:
    root = Path(folder)
    for name in ['Server.rvn', 'Server.rvnproj']:
        shutil.copyfile(here / name, root / name)
    build(root / 'Server.rvnproj')
    app = root / 'bin/neoclr/Debug/App.neoil'
    reference = root / 'reference'
    reference.mkdir()
    shutil.copyfile(client_sources / 'Reference.cs', reference / 'Program.cs')
    (reference / 'Reference.csproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><ImplicitUsings>enable</ImplicitUsings></PropertyGroup></Project>')
    built = subprocess.run(['dotnet', 'build', str(reference / 'Reference.csproj'), '-v:q', '-p:WarningLevel=0'], capture_output=True, text=True, timeout=120)
    assert built.returncode == 0, built.stdout + built.stderr

    def serve(name, client, expected='Served greeting; server closed', application=app):
        if args.case and name not in args.case:
            return
        server = subprocess.Popen([str(runner), str(application), str(runtime), '256', '10000000', '--live-output'], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            with selectors.DefaultSelector() as selector:
                selector.register(server.stdout, selectors.EVENT_READ)
                assert selector.select(120), 'Server did not report its port'
            line = server.stdout.readline().strip()
            assert line.isdecimal(), (line, server.communicate(timeout=5))
            port = int(line)
            client(port)
            output, errors = server.communicate(timeout=120)
            assert server.returncode == 0, output + errors
            assert output == 'Other work runs while HTTP accept is pending\n' + expected + '\n', output
            print(name + ': ' + expected + '; GC: ' + str(stats(errors)), flush=True)
        finally:
            if server.poll() is None:
                server.kill()
                server.communicate()

    def dotnet_client(port):
        result = subprocess.run(['dotnet', str(reference / 'bin/Debug/net10.0/Reference.dll'), str(port)], capture_output=True, text=True, timeout=30)
        assert result.returncode == 0 and result.stdout == 'HTTP 200\nCafé 🌍\n', result.stdout + result.stderr

    if not args.responses_only:
        serve('.NET HttpClient', dotnet_client)

    def raven_client(port):
        client = root / 'client'
        client.mkdir()
        for name in ['HttpClient.rvnproj', 'Handlers.rvn']:
            shutil.copyfile(client_sources / name, client / name)
        (client / 'Main.rvn').write_text((client_sources / 'Main.rvn').read_text().replace('19091', str(port)))
        build(client / 'HttpClient.rvnproj')
        result = subprocess.run([str(runner), str(client / 'bin/neoclr/Debug/App.neoil'), str(runtime), '256', '10000000'], capture_output=True, text=True, timeout=180)
        assert result.returncode == 0, result.stdout + result.stderr
        assert result.stdout == 'Handler checks passed\nOther work runs while HTTP is pending\nHTTP 200\nCafé 🌍\n', result.stdout
        print('neoCLR client GC:', stats(result.stderr), flush=True)

    if not args.responses_only:
        serve('neoCLR HttpClient', raven_client)

    def raw(request, valid=False, fragment=False, stall=False):
        def send(port):
            wire = request.replace(b'{port}', str(port).encode())
            with socket.create_connection(('127.0.0.1', port), timeout=10) as peer:
                peer.settimeout(60)
                if fragment:
                    for value in wire:
                        peer.sendall(bytes([value]))
                        time.sleep(0.002)
                else:
                    peer.sendall(wire)
                if not stall:
                    peer.shutdown(socket.SHUT_WR)
                response = bytearray()
                try:
                    while True:
                        chunk = peer.recv(1024)
                        if not chunk:
                            break
                        response.extend(chunk)
                except ConnectionResetError:
                    assert not valid, 'Valid request reset'
                if valid:
                    head, payload = bytes(response).split(b'\r\n\r\n', 1)
                    assert head.startswith(b'HTTP/1.1 200 OK\r\n'), head
                    assert b'Content-Length: 10' in head and b'Connection: close' in head, head
                    assert b'Content-Type: text/plain; charset=utf-8' in head, head
                    assert payload == body, payload
                else:
                    assert not response, response
        return send

    if not args.responses_only:
        serve('fragmented request', raw(b'GET /greeting HTTP/1.1\r\nhOsT:\tlocalhost:{port} \t\r\nContent-Length: 0\r\n\r\n', valid=True, fragment=True))
    if not args.responses_only:
        serve('stalled request', raw(b'GET /greeting HTTP/1.1\r\nHost: local', stall=True), 'Server error: TimedOut')
    cases = [
        ('Host with path', b'GET /greeting HTTP/1.1\r\nHost: localhost/path\r\n\r\n', 'Invalid Host authority'),
        ('missing Host', b'GET /greeting HTTP/1.1\r\n\r\n', 'Host required'),
        ('duplicate Host', b'GET /greeting HTTP/1.1\r\nHost: localhost\r\nhost: localhost\r\n\r\n', 'Duplicate Host'),
        ('unsupported method', b'POST /greeting HTTP/1.1\r\nHost: localhost\r\n\r\n', 'Unsupported request line'),
        ('bare LF', b'GET /greeting HTTP/1.1\n', 'Bare LF in request'),
        ('truncated request', b'GET /greeting HTTP/1.1\r\nHost: local', 'EOF before complete request'),
        ('request body', b'GET /greeting HTTP/1.1\r\nHost: localhost\r\nContent-Length: 1\r\n\r\nx', 'Request bodies unsupported'),
        ('transfer encoding', b'GET /greeting HTTP/1.1\r\nHost: localhost\r\nTransfer-Encoding: chunked\r\n\r\n', 'Request transfer encoding unsupported'),
        ('duplicate length', b'GET /greeting HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\nContent-Length: 0\r\n\r\n', 'Duplicate request Content-Length'),
        ('header bytes', b'GET /greeting HTTP/1.1\r\nHost: localhost\r\nX: ' + b'x' * 2048, 'Request header limit exceeded'),
        ('header count', b'GET /greeting HTTP/1.1\r\nHost: localhost\r\n' + b'X: y\r\n' * 16 + b'\r\n', 'Request header count exceeded'),
    ]
    if not args.responses_only:
        for name, wire, error in cases:
            serve(name, raw(wire), 'Server error: ' + error)

    source = (here / 'Server.rvn').read_text()
    variants = [
        ('header-injection', source.replace('text/plain; charset=utf-8', r'text/plain\r\nInjected: yes'), 'Invalid response header value'),
        ('framing-override', source.replace('"Content-Type"', '"Content-Length"'), 'Response framing headers are server-owned'),
        ('invalid-status', source.replace('HttpResponse(200,', 'HttpResponse(600,'), 'A final response status from 200 through 599 is required'),
        ('header-limit', source.replace('text/plain; charset=utf-8', 'x' * 2047), 'Response header limit exceeded'),
        ('body-limit', source.replace('Café 🌍', 'x' * 1025), 'Response body limit exceeded'),
        ('handler-error', source.replace('source.Complete(Ok(HttpResponse(200, headers, Utf8.Encode("Café 🌍"))))', 'source.Complete(Error(HttpError.Handler("Handler rejected request")))'), 'Handler rejected request'),
    ]
    for name, code, error in ([] if args.requests_only else variants):
        if args.case and name not in args.case:
            continue
        variant = root / name
        variant.mkdir()
        (variant / 'Server.rvn').write_text(code)
        shutil.copyfile(here / 'Server.rvnproj', variant / 'Server.rvnproj')
        build(variant / 'Server.rvnproj')
        serve(name, raw(b'GET /greeting HTTP/1.1\r\nHost: localhost\r\n\r\n'), 'Server error: ' + error, variant / 'bin/neoclr/Debug/App.neoil')

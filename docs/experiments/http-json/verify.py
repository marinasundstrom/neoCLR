#!/usr/bin/env python3
"""Exercise a JSON report over the development HTTP APIs and independent peers."""
import argparse
import json
import os
from pathlib import Path
import re
import selectors
import shutil
import subprocess
import tempfile
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.request import urlopen

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
parser.add_argument('--runner', type=Path, required=True)
args = parser.parse_args()
here = Path(__file__).resolve().parent
bundle = args.toolchain_root.resolve()
runner = args.runner.resolve()
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
expected_report = {'station': 'Café', 'readings': [21, 22.5]}
expected_reply = {'station': 'Café', 'accepted': True, 'count': 2}


def build(project):
    result = subprocess.run(['dotnet', 'msbuild', str(project), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=180)
    assert result.returncode == 0, result.stdout + result.stderr
    return project.parent / 'bin/neoclr/Debug/App.neoil'


def command(app, live=False):
    return [str(runner), str(app), str(bundle / 'lib/System.neoil'), '256', '10000000'] + (['--live-output'] if live else [])


def check_stats(stderr):
    stats = {key: int(value) for key, value in re.findall(r'(\w+)=(\d+)', stderr)}
    assert stats['live'] == 0 and stats['collections'] > 1, stderr
    return stats


with tempfile.TemporaryDirectory(prefix='neoclr-http-json-') as folder:
    root = Path(folder)
    for directory, names in [('json-message', ['JsonMessage.rvn']), ('json-document', ['JsonValue.rvn', 'JsonDocument.rvn'])]:
        target = root / directory
        target.mkdir()
        for name in names:
            shutil.copyfile(here.parent / directory / name, target / name)
    for name in ['Server', 'Client']:
        target = root / name.lower()
        target.mkdir()
        for extension in ['rvn', 'rvnproj']:
            shutil.copyfile(here / (name + '.' + extension), target / (name + '.' + extension))
    server_app = build(root / 'server/Server.rvnproj')

    def client(port):
        source = (here / 'Client.rvn').read_text().replace('localhost:19091', 'localhost:' + str(port))
        (root / 'client/Client.rvn').write_text(source)
        app = build(root / 'client/Client.rvnproj')
        result = subprocess.run(command(app), capture_output=True, text=True, timeout=120)
        assert result.returncode == 0, result.stdout + result.stderr
        lines = result.stdout.splitlines()
        assert lines[:2] == ['Station: Café', 'First reading: 21'], lines
        assert len(lines) == 3 and json.loads(lines[2]) == expected_reply, lines
        print('JSON client GC: ' + str(check_stats(result.stderr)), flush=True)

    def serve(consume):
        server = subprocess.Popen(command(server_app, True), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            with selectors.DefaultSelector() as selector:
                selector.register(server.stdout, selectors.EVENT_READ)
                assert selector.select(120), 'Server did not report a port'
            line = server.stdout.readline().strip()
            assert line.isdecimal(), (line, server.communicate(timeout=5))
            consume(int(line))
            output, errors = server.communicate(timeout=120)
            assert server.returncode == 0 and output == 'Report served\n', output + errors
            print('JSON server GC: ' + str(check_stats(errors)), flush=True)
        finally:
            if server.poll() is None:
                server.kill()
                server.communicate()

    def independent(port):
        with urlopen('http://127.0.0.1:' + str(port) + '/report', timeout=120) as response:
            assert response.status == 200
            assert response.headers['Content-Type'] == 'application/json'
            assert json.load(response) == expected_report

    serve(independent)
    serve(client)

    class Peer(BaseHTTPRequestHandler):
        protocol_version = 'HTTP/1.1'
        payload = json.dumps(expected_report, ensure_ascii=False).encode('utf-8')
        def do_GET(self):
            assert self.path == '/report', self.path
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Content-Length', str(len(self.payload)))
            self.end_headers()
            self.wfile.write(self.payload)
        def log_message(self, *args):
            pass

    peer = ThreadingHTTPServer(('127.0.0.1', 0), Peer)
    worker = threading.Thread(target=peer.serve_forever, daemon=True)
    worker.start()
    try:
        client(peer.server_port)
    finally:
        peer.shutdown()
        peer.server_close()
        worker.join()
    print('HTTP JSON application and independent peers passed', flush=True)

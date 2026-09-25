"""Verify public JSON DOM over HTTP with neoCLR and independent Python peers."""
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
from urllib.request import Request, urlopen
from urllib.error import HTTPError

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
parser.add_argument('--runner', type=Path, required=True)
parser.add_argument('--case', choices=['all', 'server', 'pair', 'client'], default='all')
args = parser.parse_args()
here = Path(__file__).resolve().parent
bundle = args.toolchain_root.resolve()
runner = args.runner.resolve()
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
report = {'station': 'Café'}
reply = {'accepted': True}


def build(project):
    result = subprocess.run(['dotnet', 'msbuild', str(project), '-nologo', '-v:minimal', '-t:NeoCLRImport'], env=env, capture_output=True, text=True, timeout=240)
    assert result.returncode == 0, result.stdout + result.stderr
    # The measured runner verifies the linked program; avoid repeating that in MSBuild.
    return next((project.parent / 'obj').glob('**/imported/App.neoil'))


def command(app, *arguments, live=False):
    return [str(runner), str(app), str(bundle / 'lib/System.neoil'), '1024', '100000000'] + (['--live-output'] if live else []) + ['--', *arguments]


def check_stats(stderr):
    stats = {key: int(value) for key, value in re.findall(r'(\w+)=(\d+)', stderr)}
    assert stats['live'] == 0 and stats['collections'] > 0, stderr
    return stats


with tempfile.TemporaryDirectory(prefix='neoclr-http-json-') as folder:
    root = Path(folder)
    apps = {}
    for name in (('Server',) if args.case == 'server' else ('Client',) if args.case == 'client' else ('Server', 'Client')):
        target = root / name.lower()
        target.mkdir()
        for filename in (name + '.rvn', name + '.rvnproj', 'Application.rvn'):
            shutil.copyfile(here / filename, target / filename)
        apps[name] = build(target / (name + '.rvnproj'))

    def client(port):
        result = subprocess.run(command(apps['Client'], f'http://localhost:{port}/'), capture_output=True, text=True, timeout=180)
        assert result.returncode == 0, result.stdout + result.stderr
        assert json.loads(result.stdout) == reply, result.stdout
        print('JSON client: ' + str(check_stats(result.stderr)), flush=True)

    def serve(consume, count):
        server = subprocess.Popen(command(apps['Server'], str(count), live=True), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            with selectors.DefaultSelector() as selector:
                selector.register(server.stdout, selectors.EVENT_READ)
                assert selector.select(180), 'Server did not report a port'
            line = server.stdout.readline().strip()
            assert line.isdecimal(), (line, server.communicate(timeout=5))
            try:
                consume(int(line))
            except Exception:
                if server.poll() is None:
                    server.kill()
                output, errors = server.communicate()
                print(output + errors, flush=True)
                raise
            output, errors = server.communicate(timeout=180)
            assert server.returncode == 0 and output == 'Reports served\n', output + errors
            print('JSON server: ' + str(check_stats(errors)), flush=True)
        finally:
            if server.poll() is None:
                server.kill()
                server.communicate()

    def independent(port):
        cases = [
            ('GET', '/report', None, 200, report),
            ('POST', '/reports', json.dumps(report, ensure_ascii=False).encode(), 201, reply),
            ('POST', '/reports', b'{', 400, {'error': 'Invalid report'}),
            ('POST', '/reports', b'{"station":"a","station":"b","readings":[]}', 400, {'error': 'Invalid report'}),
            ('POST', '/reports', b'[]', 400, {'error': 'Invalid report'}),
            ('POST', '/reports', b'{"station":12,"readings":[]}', 400, {'error': 'Invalid report'}),
            ('POST', '/reports', b'{"station":"\xff","readings":[]}', 400, {'error': 'Invalid report'}),
            ('GET', '/missing', None, 404, {'error': 'Not found'}),
        ]
        for method, path, payload, status, expected in cases:
            print(f'Peer case: {method} {path} -> {status}', flush=True)
            request = Request(f'http://127.0.0.1:{port}{path}', data=payload, method=method, headers={'Content-Type': 'application/json; charset=utf-8'})
            try:
                response = urlopen(request, timeout=120)
            except HTTPError as error:
                response = error
            with response:
                body = response.read()
                assert response.status == status, (response.status, body)
                assert response.headers['Content-Type'] == 'application/json; charset=utf-8'
                assert int(response.headers['Content-Length']) == len(body)
                assert json.loads(body) == expected, body

    if args.case in ('all', 'server'):
        serve(independent, 8)
    if args.case in ('all', 'pair'):
        serve(client, 1)

    class Peer(BaseHTTPRequestHandler):
        protocol_version = 'HTTP/1.1'
        def do_POST(self):
            assert self.path == '/reports', self.path
            assert self.headers['Content-Type'] == 'application/json; charset=utf-8'
            assert json.loads(self.rfile.read(int(self.headers['Content-Length']))) == report
            payload = json.dumps(reply, ensure_ascii=False).encode()
            self.send_response(201)
            self.send_header('Content-Type', 'application/json; charset=utf-8')
            self.send_header('Content-Length', str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)
        def log_message(self, *args):
            pass

    if args.case in ('all', 'client'):
        peer = ThreadingHTTPServer(('127.0.0.1', 0), Peer)
        worker = threading.Thread(target=peer.serve_forever, daemon=True)
        worker.start()
        try:
            client(peer.server_port)
        finally:
            peer.shutdown()
            peer.server_close()
            worker.join()
    print('Public JSON DOM + HTTP checks passed: ' + args.case, flush=True)

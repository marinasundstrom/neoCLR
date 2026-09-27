#!/usr/bin/env python3
"""Build and exercise the installed HTTP/JSON sample using Python and Raven clients."""
import argparse
import json
from pathlib import Path
import selectors
import subprocess
from urllib.error import HTTPError
from urllib.request import Request, urlopen


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', required=True, type=Path)
    parser.add_argument('--managed-pair', action='store_true', help='Also test the Raven client; currently sensitive to transport timeouts')
    args = parser.parse_args()
    bundle = args.bundle.resolve()
    for kind in ('server', 'client'):
        subprocess.run(['dotnet', 'msbuild', str(bundle / f'editable-samples/http-json/{kind}/{kind.title()}.rvnproj'),
                        '-v:minimal'], check=True, timeout=180)

    def command(kind, *arguments):
        return [str(bundle / 'tools/http-runner'),
                str(bundle / f'editable-samples/http-json/{kind}/bin/neoclr/Debug/App.neoil'),
                str(bundle / 'lib/System.neoil'), '1024', '100000000',
                *(['--live-output'] if kind == 'server' else []), '--', *arguments]

    server = subprocess.Popen(command('server', '4'), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        with selectors.DefaultSelector() as selector:
            selector.register(server.stdout, selectors.EVENT_READ)
            if not selector.select(120):
                raise TimeoutError('Server did not report its port')
        port = server.stdout.readline().strip()
        assert port.isdecimal(), 'Expected server port: ' + port
        for method, path, payload, expected_status, expected_body in (
            ('GET', '/report', None, 200, {'station': 'Café'}),
            ('POST', '/reports', b'{"station":"Cafe"}', 201, {'accepted': True}),
            ('POST', '/reports', b'{', 400, {'error': 'Invalid report'}),
            ('GET', '/missing', None, 404, {'error': 'Not found'}),
        ):
            request = Request(f'http://127.0.0.1:{port}{path}', data=payload, method=method,
                              headers={'Content-Type': 'application/json'})
            try:
                response = urlopen(request, timeout=60)
            except HTTPError as error:
                response = error
            with response:
                body = json.loads(response.read())
                assert response.status == expected_status and body == expected_body, (response.status, body)
                print(method, path, response.status, body, flush=True)
        output, errors = server.communicate(timeout=60)
        assert server.returncode == 0 and output.strip() == 'Reports served', output + errors
        if not args.managed_pair:
            print('HTTP/JSON server flow passed.', flush=True)
            return
        # Use a fresh server for the managed pair, matching the reference verifier.
        server = subprocess.Popen(command('server', '2'), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        with selectors.DefaultSelector() as selector:
            selector.register(server.stdout, selectors.EVENT_READ)
            if not selector.select(120):
                raise TimeoutError('Managed-pair server did not report its port')
        port = server.stdout.readline().strip()
        assert port.isdecimal(), 'Expected server port: ' + port
        client = subprocess.run(command('client', f'http://127.0.0.1:{port}/'),
                                capture_output=True, text=True, timeout=120)
        assert client.returncode == 0, client.stdout + client.stderr
        assert json.loads(client.stdout) == {'accepted': True}, client.stdout
        print('Raven client GET /report + POST /reports:', client.stdout.strip(), flush=True)
        output, errors = server.communicate(timeout=60)
        assert server.returncode == 0 and output.strip() == 'Reports served', output + errors
        print('HTTP/JSON flow passed.', flush=True)
    finally:
        if server.poll() is None:
            server.kill()
            output, errors = server.communicate()
            print(output + errors, flush=True)
        elif server.returncode:
            print(server.stderr.read(), flush=True)


if __name__ == '__main__':
    main()

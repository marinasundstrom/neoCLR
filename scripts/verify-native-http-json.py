#!/usr/bin/env python3
"""Execute native Raven HTTP/JSON sample assemblies against explicit dependencies."""
import argparse
import hashlib
import json
from pathlib import Path
import queue
import threading
import subprocess
from urllib.error import HTTPError
from urllib.request import Request, urlopen


def read_port(stream, timeout=60):
    """Read readiness from a pipe on Windows as well as Unix, with a deadline."""
    result = queue.Queue(maxsize=1)

    def read():
        try:
            result.put(stream.readline())
        except Exception as error:
            result.put(error)

    reader = threading.Thread(target=read, daemon=True)
    reader.start()
    try:
        value = result.get(timeout=timeout)
    except queue.Empty as error:
        raise TimeoutError('Server did not report a port') from error
    reader.join()
    if isinstance(value, Exception):
        raise value
    return value.strip()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('runtime', 'server', 'client', 'seed', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--module', type=Path, action='append', required=True)
    parser.add_argument('--object-root', type=Path)
    args = parser.parse_args()
    inputs = [args.runtime, args.server, args.client, args.seed, *args.module, *([args.object_root] if args.object_root else [])]
    report = {'inputs': {str(p.resolve()): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, 'runs': [], 'passed': False}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    if args.output.exists():
        raise FileExistsError(args.output)

    def command(assembly, argument):
        result = [str(args.runtime.resolve()), 'run', str(assembly.resolve()), '--system', str(args.seed.resolve()), '--instructions', '100000000']
        for module in args.module:
            result += ['--module', str(module.resolve())]
        if args.object_root:
            result += ['--object-root', str(args.object_root.resolve())]
        return result + ['--', argument]

    try:
        for paired in (False, True):
            run = {'serverCommand': command(args.server, '2' if paired else '4'), 'requests': []}
            report['runs'].append(run)
            server = subprocess.Popen(run['serverCommand'], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, encoding='utf-8')
            try:
                port = read_port(server.stdout)
                run['port'] = port
                if not port.isdecimal():
                    raise AssertionError('Expected server port: ' + port)
                base = f'http://127.0.0.1:{port}'
                if paired:
                    run['clientCommand'] = command(args.client, base + '/')
                    client = subprocess.run(run['clientCommand'], capture_output=True, text=True, encoding='utf-8', timeout=60)
                    run['client'] = {'exitCode': client.returncode, 'stdout': client.stdout, 'stderr': client.stderr}
                    assert client.returncode == 0 and not client.stderr, run['client']
                    assert json.loads(client.stdout) == {'accepted': True}, run['client']
                else:
                    for method, path, payload, status, body in (
                        ('GET', '/report', None, 200, {'station': 'Café'}),
                        ('POST', '/reports', b'{"station":"Cafe"}', 201, {'accepted': True}),
                        ('POST', '/reports', b'{', 400, {'error': 'Invalid report'}),
                        ('GET', '/missing', None, 404, {'error': 'Not found'}),
                    ):
                        request = Request(base + path, data=payload, method=method, headers={'Content-Type': 'application/json'})
                        try:
                            response = urlopen(request, timeout=30)
                        except HTTPError as error:
                            response = error
                        with response:
                            actual = json.loads(response.read())
                            run['requests'].append({'method': method, 'path': path, 'status': response.status, 'body': actual})
                            assert response.status == status and actual == body, run['requests'][-1]
                stdout, stderr = server.communicate(timeout=60)
                run['server'] = {'exitCode': server.returncode, 'stdout': stdout, 'stderr': stderr}
                assert server.returncode == 0 and stdout == 'Reports served\n' and not stderr, run['server']
                print('Native HTTP/JSON ' + ('client/server pair' if paired else 'server requests') + ' passed', flush=True)
            finally:
                if server.poll() is None:
                    server.kill()
                if 'server' not in run:
                    stdout, stderr = server.communicate()
                    run['server'] = {'exitCode': server.returncode, 'stdout': stdout, 'stderr': stderr}
        report['passed'] = True
    finally:
        args.output.write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()

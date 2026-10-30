#!/usr/bin/env python3
"""Execute native Raven HTTP/JSON sample assemblies against explicit dependencies."""
import argparse
import hashlib
import json
import os
import signal
from pathlib import Path
import queue
import re
import threading
import subprocess
from urllib.error import HTTPError
from urllib.request import Request, urlopen


def expected_retention_diagnostics(stderr, execution, explicit_roots):
    """Accept bounded-analysis warnings only for samples with explicit retention.

    Runtime output and exit status still have exact checks, and raw diagnostics
    remain in the report. Interpreter runs and unrelated diagnostics stay strict.
    """
    if not stderr:
        return True
    if execution not in ('default', 'jit') or not explicit_roots:
        return False
    patterns = (
        r'warning: DAM2004: dynamic-dispatch implementation bodies are not analyzed; reflection inside them may require explicit retention',
        r'warning: DAM2001: .+ instruction [0-9]+ -> .+: cannot prove dynamic member requirement (512|513); annotate the incoming TypeInfo parameter or supply explicit retention',
    )
    return all(any(re.fullmatch(pattern, line) for pattern in patterns)
               for line in stderr.splitlines())


def start_process(command, **kwargs):
    """Own the server and its JIT descendants, not just the launcher PID."""
    return subprocess.Popen(command, start_new_session=os.name != 'nt', **kwargs)


def stop_process(process, timeout=5):
    """Stop owned descendants even when the launcher has already exited."""
    if os.name == 'nt':
        if process.poll() is None:
            subprocess.run(['taskkill', '/PID', str(process.pid), '/T', '/F'],
                           capture_output=True, timeout=timeout)
    else:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
    # Always bound pipe draining: an escaped descendant must not hang qualification.
    return process.communicate(timeout=timeout)


def run_process(command, timeout, **kwargs):
    process = start_process(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, **kwargs)
    try:
        stdout, stderr = process.communicate(timeout=timeout)
        return subprocess.CompletedProcess(command, process.returncode, stdout, stderr)
    finally:
        stop_process(process)


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
    parser.add_argument('--execution', choices=('default', 'interpreter'), default='interpreter')
    parser.add_argument('--server-reflection-roots', type=Path)
    parser.add_argument('--client-reflection-roots', type=Path)
    args = parser.parse_args()
    inputs = [args.runtime, args.server, args.client, args.seed, *args.module, *([args.object_root] if args.object_root else [])]
    inputs += [p for p in (args.server_reflection_roots, args.client_reflection_roots) if p]
    report = {'execution': args.execution, 'inputs': {str(p.resolve()): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, 'runs': [], 'passed': False}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    if args.output.exists():
        raise FileExistsError(args.output)

    def command(assembly, argument):
        result = [str(args.runtime.resolve()), 'run', str(assembly.resolve())]
        if args.execution == 'interpreter':
            result += ['--execution', 'interpreter', '--system', str(args.seed.resolve()), '--instructions', '100000000']
            for module in args.module:
                result += ['--module', str(module.resolve())]
            if args.object_root:
                result += ['--object-root', str(args.object_root.resolve())]
        else:
            roots = args.server_reflection_roots if assembly == args.server else args.client_reflection_roots
            if roots:
                result += ['--reflection-roots', str(roots.resolve())]
        return result + ['--', argument]

    try:
        for paired in (False, True):
            run = {'serverCommand': command(args.server, '2' if paired else '4'), 'requests': []}
            report['runs'].append(run)
            server = start_process(run['serverCommand'], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, encoding='utf-8')
            try:
                port = read_port(server.stdout)
                run['port'] = port
                if not port.isdecimal():
                    raise AssertionError('Expected server port: ' + port)
                base = f'http://127.0.0.1:{port}'
                if paired:
                    run['clientCommand'] = command(args.client, base + '/')
                    client = run_process(run['clientCommand'], text=True, encoding='utf-8', timeout=60)
                    run['client'] = {'exitCode': client.returncode, 'stdout': client.stdout, 'stderr': client.stderr}
                    assert client.returncode == 0 and expected_retention_diagnostics(client.stderr, args.execution, args.client_reflection_roots), run['client']
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
                assert server.returncode == 0 and stdout == 'Reports served\n' and expected_retention_diagnostics(stderr, args.execution, args.server_reflection_roots), run['server']
                print('Native HTTP/JSON ' + ('client/server pair' if paired else 'server requests') + ' passed', flush=True)
            finally:
                stdout, stderr = stop_process(server)
                if 'server' not in run:
                    run['server'] = {'exitCode': server.returncode, 'stdout': stdout, 'stderr': stderr}
        report['passed'] = True
    finally:
        args.output.write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()

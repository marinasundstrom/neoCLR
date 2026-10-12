#!/usr/bin/env python3
"""Qualify managed HTTPS against independent Python/OpenSSL peers in both directions."""
import argparse
import hashlib
import http.client
import importlib.util
import json
from pathlib import Path
import platform
import queue
import shutil
import socket
import ssl
import subprocess
import sys
import threading

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
PAYLOAD = '{"message":"Café 🌍","nested":{"items":[1,true,null]}}'.encode('utf-8')
VERSIONS = (ssl.TLSVersion.TLSv1_2, ssl.TLSVersion.TLSv1_3)


def context(version, cert, key=None):
    if key is None:
        ctx = ssl.create_default_context(cafile=str(cert))
    else:
        ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        ctx.load_cert_chain(str(cert), str(key))
    ctx.minimum_version = ctx.maximum_version = version
    return ctx


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('bundle', 'aot', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--reuse-build', action='store_true', help='Reuse this output only after checking recorded build input hashes')
    args = parser.parse_args()
    if not ssl.HAS_TLSv1_3:
        parser.error('Use Python linked against OpenSSL with TLS 1.3 support; this interpreter uses ' + ssl.OPENSSL_VERSION)
    bundle, aot, output = (getattr(args, name).resolve() for name in ('bundle', 'aot', 'output'))
    output.mkdir(parents=True, exist_ok=args.reuse_build)
    report = dict(passed=False, platform=platform.platform(), openssl=ssl.OPENSSL_VERSION,
                  commands=[], cases=[])
    windows = platform.system() == 'Windows'

    def run(command):
        result = subprocess.run(list(map(str, command)), capture_output=True, text=True,
                                encoding='utf-8', timeout=900)
        report['commands'].append(dict(command=list(map(str, command)), exitCode=result.returncode,
                                       stdout=result.stdout, stderr=result.stderr))
        if result.returncode:
            raise RuntimeError(result.stdout + result.stderr)
        return result

    def external_server(listener, cert, key, outcomes, mode):
        try:
            listener.settimeout(180)
            for version in VERSIONS:
                peer, _ = listener.accept()
                peer.settimeout(60)
                with peer, context(version, cert, key).wrap_socket(peer, server_side=True) as secure:
                    with secure.makefile('rb') as stream:
                        assert stream.readline(8193) == b'POST /echo HTTP/1.1\r\n'
                        headers = {}
                        for _ in range(100):
                            line = stream.readline(8193)
                            if line == b'\r\n':
                                break
                            name, value = line.decode('ascii').split(':', 1)
                            headers[name.lower()] = value.strip()
                        else:
                            raise AssertionError('Header limit exceeded')
                        assert int(headers['content-length']) == len(PAYLOAD)
                        assert stream.read(len(PAYLOAD)) == PAYLOAD
                        # Chunked framing and UTF-8 codepoints deliberately cross TLS record boundaries.
                        wire = b'HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n'
                        for start in range(0, len(PAYLOAD), 5):
                            part = PAYLOAD[start:start + 5]
                            wire += format(len(part), 'x').encode() + b'\r\n' + part + b'\r\n'
                        wire += b'0\r\n\r\n'
                        for start in range(0, len(wire), 7):
                            secure.sendall(wire[start:start + 7])
                        outcomes.put(dict(mode=mode, peer='external-server', protocol=secure.version(), passed=True))
        except Exception as error:
            outcomes.put(error)
        finally:
            listener.close()

    try:
        project, build = output / 'project', output / 'native'
        cert, key = output / 'cert.pem', output / 'key.pem'
        if args.reuse_build:
            previous = json.loads((build / 'build.json').read_text())
            assert previous['passed'], 'Cannot reuse a failed build'
            assert str(bundle / 'lib/bundle.json') in previous['inputs'], 'Bundle path changed'
            assert str(aot) in previous['inputs'], 'Native compiler path changed'
            for name, expected in previous['inputs'].items():
                assert hashlib.sha256(Path(name).read_bytes()).hexdigest() == expected, 'Build input changed: ' + name
            for source in (HERE / 'interop-project').glob('*'):
                if source.is_file():
                    assert source.read_bytes() == (project / source.name).read_bytes(), 'Consumer source changed'
            report['reusedBuild'] = str(build / 'build.json')
        else:
            spec = importlib.util.spec_from_file_location('bindings', HERE / 'validate-bindings.py')
            helpers = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(helpers)
            cert, key = helpers.create_credentials(output, run)
            project = output / 'project'
            shutil.copytree(HERE / 'interop-project', project, ignore=shutil.ignore_patterns('bin', 'obj'))
            source = 'import System.*\ninternal static class Fixtures {\n'
            for name, path in [('Certificate', cert), ('Key', key)]:
                source += '    internal static func ' + name + '() -> byte[] {\n        return ['
                source += ', '.join('(byte)' + str(value) for value in path.read_bytes()) + ']\n    }\n'
            (project / 'Fixtures.rvn').write_text(source + '}\n')
            build = output / 'native'
            run([sys.executable, ROOT / 'scripts/build-native-project.py', '--profile',
                 'windows-http' if windows else 'http', '--project', project / 'App.rvnproj',
                 '--bundle', bundle, '--aot', aot, '--output', build,
                 '--http-heap-bytes', '8388608', '--command-timeout', '600'])
        assert json.loads((build / 'build.json').read_text())['managedTls']
        runtime = bundle / ('bin/neoclr.exe' if windows else 'bin/neoclr')
        catalog = json.loads((bundle / 'lib/bundle.json').read_text())
        modules = ['--system', bundle / 'lib' / catalog['runtimeSeed'], '--object-root', bundle / 'lib/System.Runtime.dll']
        for name in catalog['assemblyNames']:
            modules += ['--module', bundle / 'lib' / (name + '.dll')]
        commands = {
            'interpreter': [runtime, 'run', build / 'app.dll', '--execution', 'interpreter',
                            '--instructions', '100000000', '--array-elements', '1048576',
                            '--array-bytes', '67108864', *modules, '--'],
            'default-jit': [runtime, 'run', build / 'app.dll', '--'],
            'aot': [build / ('app.exe' if windows else 'app')],
        }
        for mode, command in commands.items():
            outcomes = queue.Queue()
            listener = socket.socket()
            listener.bind(('127.0.0.1', 0))
            listener.listen(8)
            port = listener.getsockname()[1]
            worker = threading.Thread(target=external_server, args=(listener, cert, key, outcomes, mode), daemon=True)
            worker.start()
            try:
                try:
                    result = run([*command, 'client', f'https://localhost:{port}/echo'])
                except Exception:
                    worker.join(2)
                    report['peerOutcomes'] = []
                    while not outcomes.empty():
                        report['peerOutcomes'].append(str(outcomes.get()))
                    raise
                assert result.stdout == 'PASS managed client\n', result.stdout
                worker.join(65)
                assert not worker.is_alive(), 'External server did not finish'
                for _ in VERSIONS:
                    outcome = outcomes.get(timeout=1)
                    if isinstance(outcome, Exception):
                        raise outcome
                    report['cases'].append(outcome)
            finally:
                listener.close()
            process = subprocess.Popen(list(map(str, [*command, 'server'])), stdout=subprocess.PIPE,
                                       stderr=subprocess.PIPE, text=True, encoding='utf-8')
            lines = queue.Queue()
            def read_port():
                lines.put(process.stdout.readline())
            reader = threading.Thread(target=read_port, daemon=True)
            reader.start()
            try:
                port_line = lines.get(timeout=180)
                reader.join()
                port = int(port_line.strip())
                for version in VERSIONS:
                    client = http.client.HTTPSConnection('localhost', port, timeout=60, context=context(version, cert))
                    try:
                        client.connect()
                        protocol = client.sock.version()
                        client.request('POST', '/echo', PAYLOAD, {'Content-Type': 'application/json'})
                        response = client.getresponse()
                        assert response.status == 200
                        assert json.loads(response.read()) == json.loads(PAYLOAD)
                        report['cases'].append(dict(mode=mode, peer='external-client', protocol=protocol, passed=True))
                    finally:
                        client.close()
                stdout, stderr = process.communicate(timeout=65)
                report['commands'].append(dict(command=list(map(str, [*command, 'server'])),
                                               exitCode=process.returncode, stdout=port_line + stdout, stderr=stderr))
                assert process.returncode == 0 and stdout == 'PASS managed server\n', (stdout, stderr)
            finally:
                if process.poll() is None:
                    process.kill()
                    process.communicate()
        report['passed'] = True
        report['sourceHashes'] = {p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                                  for p in (HERE / 'interop-project').glob('*') if p.is_file()}
    except Exception as error:
        report['error'] = str(error)
        raise
    finally:
        (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()

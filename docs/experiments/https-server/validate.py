#!/usr/bin/env python3
"""Native HTTPS server proof against Python/OpenSSL with certificate verification."""
import argparse
import hashlib
import http.client
import json
import os
from pathlib import Path
import platform
import re
import importlib.util
import socket
import ssl
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('http_json', ROOT / 'scripts/verify-native-http-json.py')
http_json = importlib.util.module_from_spec(spec)
spec.loader.exec_module(http_json)
read_port = http_json.read_port
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--recipe', type=Path, default=HERE.parent / 'runtime-async/collector-walk-showcase.json')
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--windows-jit', action='store_true', help='Also qualify a Windows JIT helper DLL built from the project recipe')
    parser.add_argument('--cleartext-build', type=Path, help='Matching ordinary project build for cleartext isolation')
    parser.add_argument('--project-build', type=Path, help='Validate a matching --https-server project build instead of rebuilding experimental hosts')
    args = parser.parse_args()
    if not ssl.HAS_TLSv1_3:
        parser.error('requires Python linked to OpenSSL with TLS 1.3 support')
    windows = platform.system() == 'Windows' and platform.machine().lower() in ('amd64', 'x86_64')
    if args.windows_jit and not (windows and args.project_build):
        parser.error('--windows-jit requires a Windows project build')
    if windows and not args.project_build:
        parser.error('Windows qualification requires --project-build')
    if not windows and (platform.system(), platform.machine()) != ('Darwin', 'arm64'):
        parser.error('requires macOS ARM64 or Windows x64')
    out = args.output.resolve(); out.mkdir(parents=True, exist_ok=False)
    recipe = json.loads(args.recipe.read_text()); assert recipe['passed']
    report = dict(passed=False, platform=platform.platform(), ssl=ssl.OPENSSL_VERSION,
                  commands=[], cases=[], inputsSha256={}, recipeSha256=sha(args.recipe),
                  credentialPathFixture='non-ASCII and non-BMP directory')

    def command(label):
        cmd = next(row['command'].copy() for row in recipe['commands'] if row.get('label') == label)
        return [str(args.bundle.resolve() / 'lib' / Path(v).name) if '/data-cases-bundle/lib/' in v else v for v in cmd]

    def run(label, cmd, **kwargs):
        for value in cmd:
            if Path(value).is_file() and not Path(value).is_relative_to(out):
                report['inputsSha256'][str(Path(value).resolve())] = sha(value)
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=180, cwd=ROOT, **kwargs)
        row = dict(label=label, command=cmd, exitCode=result.returncode, stderr=result.stderr)
        if label == 'native' or len(result.stdout) > 20000:
            p = out / ('selection.json' if label == 'native' else label + '.stdout.log')
            p.write_text(result.stdout)
            row['stdoutSha256'] = sha(p)
        else: row['stdout'] = result.stdout
        report['commands'].append(row)
        if result.returncode: raise RuntimeError(label + ': ' + result.stderr + result.stdout[-1000:])
        return result

    try:
        if args.project_build:
            build = args.project_build.resolve()
            evidence = json.loads((build / 'build.json').read_text())
            if not evidence.get('passed') or not evidence.get('httpsServer'):
                raise ValueError('Requires a successful HTTPS server project build')
            for path, digest in evidence['inputs'].items():
                if sha(path) != digest: raise ValueError('Project build input changed: ' + path)
            for name, digest in evidence['artifacts'].items():
                if sha(build / name) != digest: raise ValueError('Project build artifact changed: ' + name)
            report['projectBuildSha256'] = sha(build / 'build.json')
            report['inputsSha256'].update(evidence['inputs'])
            invocations = [('aot-project', [str(build / ('app.exe' if windows else 'app'))])]
            credential_app = build / ('app.exe' if windows else 'app')
            if args.windows_jit:
                helper_path = HERE.parent / 'jit-http/windows.py'
                helper_spec = importlib.util.spec_from_file_location('windows_jit', helper_path)
                helper = importlib.util.module_from_spec(helper_spec)
                helper_spec.loader.exec_module(helper)
                report['inputsSha256'][str(helper_path)] = sha(helper_path)
                native, link = helper.recipe(evidence, ROOT, out)
                sequence = 0
                def jit_build(cmd):
                    nonlocal sequence
                    sequence += 1
                    return run('jit-host-' + str(sequence), list(map(str, cmd)))
                host = helper.build_host(link, out, jit_build)
                dependency_output = run('jit-dependencies', ['dumpbin', '/dependents', str(host)]).stdout
                dependencies = re.findall(r'^\s+([A-Za-z0-9_.-]+\.dll)\s*$', dependency_output, re.MULTILINE | re.IGNORECASE)
                if not dependencies or set(map(str.lower, dependencies)) - set(map(str.lower, evidence['dependencies'])):
                    raise RuntimeError('JIT helper added unexpected dependencies: ' + repr(dependencies))
                report['jitHostSha256'] = sha(host)
                report['jitDependencies'] = dependencies
                jit = [native[0], '--jit-run', *native[1:]]
                jit[5] = str(host)
                invocations.append(('jit', jit))
        else:
            run('tls-build', ['cargo', 'build', '--locked', '--profile', 'native-ffi', '--manifest-path', str(ROOT / 'tools/native-tls/Cargo.toml')])
            source = command('language-source')
            source[source.index('-o') + 1] = str(out / 'server.dll'); source[-2] = str(HERE / 'Server.rvn')
            run('source', source)
            native = command('server-native-compile'); native[2], native[4] = str(out / 'server.dll'), str(out / 'server.o')
            run('native', native)
            archive = str(ROOT / 'target/native-ffi/libneoclr_native_tls.a')
            link = command('server-link')
            link = [str(out / 'server.o') if v.endswith('/server.o') else v for v in link]
            link[link.index('-o') + 1] = str(out / 'server')
            link += ['-DNEOCLR_ENTRY_ARGUMENTS', '-DNEOCLR_HTTPS_SERVER', archive]
            run('aot-link', link)
            cleartext = [item for item in link if item not in ('-DNEOCLR_HTTPS_SERVER', archive)]
            cleartext[cleartext.index('-o') + 1] = str(out / 'cleartext')
            run('cleartext-link', cleartext)
            jit_link = command('jit-host-build') + ['-DNEOCLR_HTTPS_SERVER', archive]
            jit_link[jit_link.index('-o') + 1] = str(out / 'host.dylib')
            run('jit-link', jit_link)
            jit = [native[0], '--jit-run', *native[1:]]; jit[5] = str(out / 'host.dylib')
            invocations = [('aot', [str(out / 'server')]), ('jit', jit)]
            credential_app = out / 'server'
        # Current transitive host headers and TLS engine, without any generated private keys.
        for folder in [HERE, HERE.parent / 'aot-console', ROOT / 'tools/native-tls/src']:
            for p in folder.iterdir():
                if p.suffix in ('.c', '.h', '.rs', '.rvn', '.py'): report['inputsSha256'][str(p)] = sha(p)
        report['inputsSha256'][str(ROOT / 'scripts/verify-native-http-json.py')] = sha(ROOT / 'scripts/verify-native-http-json.py')
        report['inputsSha256'][str(ROOT / 'Cargo.lock')] = sha(ROOT / 'Cargo.lock')
        with tempfile.TemporaryDirectory(prefix='nc-https-Café-🌍-') as directory:
            cert, key = Path(directory) / 'cert.pem', Path(directory) / 'key.pem'
            # Do not inherit platform OpenSSL defaults (some mark self-signed certs as CAs).
            config = Path(directory) / 'openssl.cnf'
            config.write_text('[req]\nprompt=no\ndistinguished_name=dn\n[dn]\nCN=localhost\n')
            # One-use development leaf: explicit peer trust, no global store edits.
            subprocess.run(['openssl', 'req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-sha256', '-days', '1',
                            '-config', str(config), '-subj', '/CN=localhost',
                            '-addext', 'basicConstraints=critical,CA:FALSE',
                            '-addext', 'keyUsage=critical,digitalSignature,keyEncipherment',
                            '-addext', 'extendedKeyUsage=serverAuth',
                            '-addext', 'subjectAltName=DNS:localhost,IP:127.0.0.1',
                            '-keyout', str(key), '-out', str(cert)], check=True, capture_output=True, timeout=30)
            os.chmod(key, 0o600)
            trust = ssl.create_default_context(cafile=str(cert)); trust.set_alpn_protocols(['http/1.1'])
            env = dict(os.environ, NEOCLR_HTTPS_CERTIFICATE=str(cert), NEOCLR_HTTPS_PRIVATE_KEY=str(key))
            env.pop('NEOCLR_DIAGNOSTICS_DIRECTORY', None)
            run('fragmented-transport-test', ['cargo', 'test', '--locked', '--manifest-path', str(ROOT / 'tools/native-tls/Cargo.toml'), '--', '--ignored'], env=env)

            def valid(port, version=None, fragmented=False):
                context = trust
                if version:
                    context = ssl.create_default_context(cafile=str(cert)); context.set_alpn_protocols(['http/1.1'])
                    context.minimum_version = context.maximum_version = version
                with socket.create_connection(('127.0.0.1', port), timeout=10) as tcp:
                    with context.wrap_socket(tcp, server_hostname='localhost', suppress_ragged_eofs=False) as secure:
                        wire = b'GET /greeting HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n'
                        if fragmented:
                            for i in range(0, len(wire), 3): secure.sendall(wire[i:i+3])
                        else: secure.sendall(wire)
                        response = http.client.HTTPResponse(secure); response.begin()
                        body = response.read()
                        if response.status != 200 or body != 'Café 🌍 over TLS'.encode(): raise RuntimeError('bad HTTPS response')
                        if secure.selected_alpn_protocol() != 'http/1.1': raise RuntimeError('ALPN changed')
                        protocol = secure.version()
                        # Require authenticated TLS EOF, not reciprocal shutdown after
                        # the server has already closed its socket. In strict mode an
                        # absent close_notify raises SSLEOFError rather than returning EOF.
                        if secure.recv(1) != b'': raise RuntimeError('trailing TLS application data')
                        return protocol

            for mode, invocation in invocations:
                for case in ['tls13', 'tls12', 'fragmented', 'untrusted', 'wrong-host', 'alpn', 'plaintext', 'truncated', 'abrupt-eof', 'clean-eof', 'timeout', 'cancel']:
                    scenario = 'ok' if case in ('tls13', 'tls12', 'fragmented') else case
                    process = subprocess.Popen(invocation + ([scenario] if mode != 'jit' else ['--jit-entry-argument', scenario]), env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                    try:
                        line = read_port(process.stdout, timeout=60)
                        if not line.strip().isdigit(): raise RuntimeError('startup failed: ' + line + process.stderr.read())
                        port = int(line)
                        protocol = None
                        if scenario == 'ok':
                            protocol = valid(port, ssl.TLSVersion.TLSv1_2 if case == 'tls12' else ssl.TLSVersion.TLSv1_3,
                                             case == 'fragmented')
                        elif case in ('abrupt-eof', 'clean-eof'):
                            with socket.create_connection(('127.0.0.1', port), timeout=10) as tcp:
                                with trust.wrap_socket(tcp, server_hostname='localhost') as secure:
                                    # An authenticated peer closes before finishing its HTTP headers.
                                    secure.sendall(b'GET /greeting HTTP/1.1\r\nHost: local')
                                    if case == 'clean-eof':
                                        with secure.unwrap():
                                            pass
                                    else:
                                        # Transfer the descriptor so closing bypasses TLS close_notify.
                                        with socket.socket(fileno=secure.detach()) as raw:
                                            raw.shutdown(socket.SHUT_WR)
                        elif case in ('untrusted', 'wrong-host', 'alpn'):
                            context = ssl.create_default_context() if case == 'untrusted' else ssl.create_default_context(cafile=str(cert))
                            context.set_alpn_protocols(['h2'] if case == 'alpn' else ['http/1.1'])
                            try:
                                with socket.create_connection(('127.0.0.1', port), timeout=10) as tcp:
                                    with context.wrap_socket(tcp, server_hostname='wrong.example' if case == 'wrong-host' else 'localhost'):
                                        raise RuntimeError('invalid TLS peer accepted')
                            except ssl.SSLCertVerificationError:
                                if case not in ('untrusted', 'wrong-host'): raise
                            except ssl.SSLError:
                                if case != 'alpn': raise
                        else:
                            with socket.create_connection(('127.0.0.1', port), timeout=10) as tcp:
                                if case == 'plaintext': tcp.sendall(b'GET / HTTP/1.1\r\nHost: localhost\r\n\r\n')
                                if case == 'truncated': tcp.sendall(b'\x16\x03\x03\x00'); tcp.shutdown(socket.SHUT_WR)
                                try: data = tcp.recv(4096)
                                except ConnectionResetError: data = b''
                                if data: raise RuntimeError('unexpected plaintext response')
                        valid(port)
                        stdout, stderr = process.communicate(timeout=10)
                        good = process.returncode == 0 and not stderr and stdout.strip() == 'PASS HTTPS server and recovery'
                        report['cases'].append(dict(mode=mode, case=case, passed=good, protocol=protocol,
                                                    exitCode=process.returncode, stdout=stdout, stderr=stderr))
                        print(mode, case, 'PASS' if good else 'FAIL', flush=True)
                        if not good: raise RuntimeError(str(report['cases'][-1]))
                    finally:
                        if process.poll() is None: process.kill(); process.communicate()
            # Config errors happen before binding, never a cleartext fallback.
            for name, overrides in [('missing', {'NEOCLR_HTTPS_PRIVATE_KEY': ''}),
                                    ('invalid', {'NEOCLR_HTTPS_PRIVATE_KEY': str(cert)})]:
                result = subprocess.run([str(credential_app), 'ok'], env=dict(env, **overrides), capture_output=True, text=True, timeout=5)
                good = result.returncode != 0 and not result.stdout
                report['cases'].append(dict(mode='aot', case=name + '-credentials', passed=good, exitCode=result.returncode))
                if not good: raise RuntimeError('invalid credentials accepted')
        if args.cleartext_build or not args.project_build:
            plain_app = out / 'cleartext'
            if args.cleartext_build:
                plain_build = args.cleartext_build.resolve()
                plain_evidence = json.loads((plain_build / 'build.json').read_text())
                if not plain_evidence.get('passed') or plain_evidence.get('httpsServer'):
                    raise ValueError('Requires an ordinary cleartext project build')
                for path, digest in plain_evidence['inputs'].items():
                    if sha(path) != digest: raise ValueError('Cleartext input changed: ' + path)
                for name, digest in plain_evidence['artifacts'].items():
                    if sha(plain_build / name) != digest: raise ValueError('Cleartext artifact changed: ' + name)
                report['inputsSha256'].update(plain_evidence['inputs'])
                report['cleartextBuildSha256'] = sha(plain_build / 'build.json')
                plain_app = plain_build / ('app.exe' if windows else 'app')
            plain = subprocess.Popen([str(plain_app), 'ok'], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            try:
                port = int(read_port(plain.stdout, timeout=20))
                for _ in range(2):
                    connection = http.client.HTTPConnection('127.0.0.1', port, timeout=10)
                    connection.request('GET', '/greeting'); response = connection.getresponse()
                    if response.status != 200 or response.read() != 'Café 🌍 over TLS'.encode():
                        raise RuntimeError('cleartext behavior changed')
                    connection.close()
                stdout, stderr = plain.communicate(timeout=10)
                good = plain.returncode == 0 and not stderr and 'PASS HTTPS server and recovery' in stdout
                report['cases'].append(dict(mode='aot', case='ordinary-host-stays-cleartext', passed=good))
                if not good: raise RuntimeError('cleartext shutdown failed')
            finally:
                if plain.poll() is None: plain.kill(); plain.communicate()
        for path, digest in report['inputsSha256'].items():
            if sha(path) != digest: raise RuntimeError('input changed: ' + path)
        report['passed'] = True
    finally:
        (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(out / 'validation.json')


if __name__ == '__main__': main()

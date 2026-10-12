#!/usr/bin/env python3
"""Exercise the shared provider with disposable credentials on macOS and Windows."""
import os
import json
import platform
import shlex
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def native_adapter(path, cert, key):
    manifest = ROOT / 'tools/native-tls/Cargo.toml'
    subprocess.run(['cargo', 'build', '--locked', '--release', '--manifest-path', str(manifest)],
                   check=True, timeout=600)
    metadata = json.loads(subprocess.check_output(
        ['cargo', 'metadata', '--no-deps', '--format-version', '1', '--manifest-path', str(manifest)],
        text=True))
    windows = platform.system() == 'Windows'
    archive = Path(metadata['target_directory']) / 'release' / (
        'neoclr_native_tls.lib' if windows else 'libneoclr_native_tls.a')
    base = ROOT / 'docs/experiments/aot-console'
    sources = [str(base / name) for name in
               ('tls-managed-test.c', 'tls-managed.c', 'native-gc.c', 'root-probe.c')]
    binary = path / ('tls-managed-test.exe' if windows else 'tls-managed-test')
    if windows:
        command = ['cl', '/nologo', '/W4', '/WX', '/std:c11', '/experimental:c11atomics',
                   '/O2', '/MT', '/DNEOCLR_NATIVE_GC', '/Fo' + str(path) + '/',
                   '/Fe:' + str(binary), *sources, str(archive), '/link',
                   'Bcrypt.lib', 'Ntdll.lib', 'Userenv.lib', 'Advapi32.lib', 'Ws2_32.lib']
    else:
        command = [*shlex.split(os.environ.get('CC', 'clang')), '-std=c11', '-O2',
                   '-Wall', '-Wextra', '-Werror', '-DNEOCLR_NATIVE_GC',
                   *sources, str(archive), '-o', str(binary)]
        if platform.system() == 'Darwin':
            sdk = os.environ.get('SDKROOT') or subprocess.check_output(
                ['xcrun', '--show-sdk-path'], text=True).strip()
            command += ['-isysroot', sdk]
    subprocess.run(command, check=True, timeout=180)
    subprocess.run([str(binary), str(cert), str(key)], check=True, timeout=30)
    print('Native TLS managed lifetime adapter: passed', flush=True)


def main():
    subprocess.run(["cargo", "test", "--locked", "--manifest-path",
                    str(ROOT / "tools/security-provider/Cargo.toml")], check=True, timeout=180)
    subprocess.run(["cargo", "test", "--locked", "--manifest-path",
                    str(ROOT / "tools/native-crypto/Cargo.toml")], check=True, timeout=180)
    with tempfile.TemporaryDirectory(prefix='neoclr-security-') as directory:
        path = Path(directory)
        config = path / 'openssl.cnf'
        config.write_text('[req]\nprompt=no\ndistinguished_name=dn\n[dn]\nCN=localhost\n')
        cert, key = path / 'cert.pem', path / 'key.pem'
        subprocess.run([
            'openssl', 'req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-sha256', '-days', '1',
            '-config', str(config), '-subj', '/CN=localhost',
            '-addext', 'basicConstraints=critical,CA:FALSE',
            '-addext', 'keyUsage=critical,digitalSignature,keyEncipherment',
            '-addext', 'extendedKeyUsage=serverAuth',
            '-addext', 'subjectAltName=DNS:localhost,IP:127.0.0.1',
            '-keyout', str(key), '-out', str(cert),
        ], check=True, capture_output=True, timeout=30)
        os.chmod(key, 0o600)
        env = dict(os.environ, NEOCLR_HTTPS_CERTIFICATE=str(cert), NEOCLR_HTTPS_PRIVATE_KEY=str(key))
        subprocess.run(['cargo', 'test', '--locked', '--lib', 'tls::tests', '--', '--include-ignored'],
                       cwd=ROOT, env=env, check=True, timeout=600)
        subprocess.run(['cargo', 'test', '--locked', '--manifest-path',
                        str(ROOT / 'tools/native-tls/Cargo.toml'), '--', '--include-ignored'],
                       env=env, check=True, timeout=180)
        native_adapter(path, cert, key)


if __name__ == '__main__':
    main()

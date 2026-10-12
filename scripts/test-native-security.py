#!/usr/bin/env python3
"""Exercise the shared provider with disposable credentials on macOS and Windows."""
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


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
        subprocess.run(['cargo', 'test', '--locked', '--manifest-path',
                        str(ROOT / 'tools/native-tls/Cargo.toml'), '--', '--include-ignored'],
                       env=env, check=True, timeout=180)


if __name__ == '__main__':
    main()

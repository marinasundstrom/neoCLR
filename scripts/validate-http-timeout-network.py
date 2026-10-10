#!/usr/bin/env python3
"""Verify HTTP timeout acknowledgement closes a real local socket in both modes."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import socket
import subprocess
import sys
import threading
import time

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--runtime', type=Path, default=ROOT / 'target/release' / ('neoclr.exe' if os.name == 'nt' else 'neoclr'))
    parser.add_argument('--aot', type=Path, default=ROOT / 'tools/aot-poc/target/debug' / ('neoclr-aot-poc.exe' if os.name == 'nt' else 'neoclr-aot-poc'))
    args = parser.parse_args()
    bundle, out = args.bundle.resolve(), args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    fixture = ROOT / 'docs/experiments/http-timeout/Network.rvnproj'
    report = dict(passed=False, platform=platform.platform(), cases=[],
                  sourceSha256=hashlib.sha256(fixture.with_suffix('.rvn').read_bytes()).hexdigest(),
                  bundleManifestSha256=hashlib.sha256((bundle / 'manifest.json').read_bytes()).hexdigest())
    try:
        build = out / 'build'
        result = subprocess.run([sys.executable, str(ROOT / 'scripts/build-native-project.py'), '--profile',
                                 'windows-http' if os.name == 'nt' else 'http', '--project', str(fixture),
                                 '--bundle', str(bundle), '--aot', str(args.aot.resolve()), '--output', str(build)],
                                cwd=ROOT, capture_output=True, text=True, timeout=600)
        (out / 'build.stdout.log').write_text(result.stdout)
        (out / 'build.stderr.log').write_text(result.stderr)
        if result.returncode:
            raise RuntimeError('Network consumer build failed; see logs')
        lib = bundle / 'lib'
        interpreter = [str(args.runtime.resolve()), 'run', str(build / 'app.dll'), '--system', str(lib / 'System.runtime.neox'),
                       '--object-root', str(lib / 'System.Runtime.dll')]
        for name in ('System.Runtime', 'System.Data', 'System.Networking', 'System.Web'):
            interpreter += ['--module', str(lib / (name + '.dll'))]
        native = [str(build / ('app.exe' if os.name == 'nt' else 'app'))]
        for mode, command in (('interpreter', interpreter), ('native', native)):
            for case in ('timeout', 'success'):
                peer = {}
                with socket.socket() as listener:
                    listener.bind(('127.0.0.1', 0))
                    listener.listen(1)
                    listener.settimeout(20)
                    url = 'http://127.0.0.1:' + str(listener.getsockname()[1]) + '/'
                    def serve():
                        try:
                            with listener.accept()[0] as connection:
                                connection.settimeout(5)
                                request = b''
                                while b'\r\n\r\n' not in request:
                                    part = connection.recv(4096)
                                    if not part:
                                        raise RuntimeError('Client closed before request headers')
                                    request += part
                                peer['requestBytes'] = len(request)
                                if case == 'success':
                                    time.sleep(.35)
                                    connection.sendall(b'HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}')
                                peer['closed'] = connection.recv(1) == b''
                        except Exception as error:
                            peer['error'] = str(error)
                    worker = threading.Thread(target=serve, daemon=True)
                    worker.start()
                    result = subprocess.run([*command, *(['--'] if mode == 'interpreter' else []), url, case],
                                            cwd=ROOT, capture_output=True, text=True, timeout=30)
                    worker.join(timeout=6)
                report['cases'].append(dict(case=case, mode=mode, exitCode=result.returncode,
                                            stdout=result.stdout, stderr=result.stderr, peer=peer))
                if result.returncode or 'PASS ' not in result.stdout or not peer.get('closed') or peer.get('error'):
                    raise RuntimeError(f'{mode} {case} did not finish and close its socket: {report["cases"][-1]}')
        report['passed'] = True
    finally:
        (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS real HTTP timeout and disabled-timeout response in both modes')


if __name__ == '__main__':
    main()

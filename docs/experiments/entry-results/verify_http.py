"""Build the direct-async-Main HTTP experiment and run it against a local peer."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import socket
import subprocess
import tempfile
import threading

parser = argparse.ArgumentParser(description=__doc__)
for name in ('runtime', 'bridge', 'system', 'reference'):
    parser.add_argument('--' + name, type=Path, required=True)
parser.add_argument('--evidence', type=Path)
parser.add_argument('--runner', type=Path, required=True)
args = parser.parse_args()
artifacts = {name: getattr(args, name).resolve() for name in ('runtime', 'bridge', 'system', 'reference')}
here = Path(__file__).resolve().parent
http = here.parent / 'http-client'


def run(command):
    result = subprocess.run([str(x) for x in command], capture_output=True, text=True, timeout=180)
    assert result.returncode == 0, (result.stdout, result.stderr)
    return result


with tempfile.TemporaryDirectory(prefix='neoclr-async-main-http-') as folder, socket.socket() as listener:
    root = Path(folder)
    listener.bind(('127.0.0.1', 0))
    listener.listen(1)
    listener.settimeout(120)
    port = listener.getsockname()[1]
    (root / 'Main.rvn').write_text((http / 'Main.rvn').read_text().replace('19091', str(port)))
    shutil.copyfile(http / 'Handlers.rvn', root / 'Handlers.rvn')
    shutil.copyfile(artifacts['reference'], root / 'NeoCLR.CoreProbe.dll')
    (root / 'Contracts.rvnproj').write_text((here / 'Contracts.rvnproj').read_text().replace('<Compile Include="Main.rvn" />', '<Compile Include="Main.rvn" /><Compile Include="Handlers.rvn" />'))
    run(['dotnet', artifacts['bridge'], '--project', root / 'Contracts.rvnproj', root / 'out'])
    app = root / 'out/App.neoil'
    run([artifacts['runtime'], 'verify', app, '--system', artifacts['system']])
    errors = []

    def serve():
        try:
            with listener.accept()[0] as peer:
                peer.settimeout(60)
                request = bytearray()
                while not request.endswith(b'\r\n\r\n'):
                    part = peer.recv(1)
                    assert part and len(request) < 4096
                    request.extend(part)
                assert request.startswith(b'GET /greeting HTTP/1.1\r\n'), request
                body = 'Café 🌍'.encode('utf-8')
                peer.sendall(b'HTTP/1.1 200 OK\r\nContent-Length: ' + str(len(body)).encode() + b'\r\nConnection: close\r\n\r\n' + body)
        except BaseException as error:
            errors.append(str(error))

    worker = threading.Thread(target=serve, daemon=True)
    worker.start()
    result = run([args.runner.resolve(), app, artifacts['system'], '512', '100000000'])
    worker.join(timeout=65)
    assert not worker.is_alive() and not errors, errors
    expected = 'Handler checks passed\nOther work runs while HTTP is pending\nHTTP 200\nCafé 🌍\n'
    assert result.stdout == expected and 'live=0' in result.stderr, (result.stdout, result.stderr)
    print(result.stdout, end='')
    if args.evidence:
        args.evidence.write_text(json.dumps({'source': 'docs/experiments/http-client/Main.rvn', 'passed': True, 'stdout': result.stdout, 'sourceSha256': {name: hashlib.sha256((http / name).read_bytes()).hexdigest() for name in ('Main.rvn', 'Handlers.rvn')}, 'sha256': {name: hashlib.sha256(path.read_bytes()).hexdigest() for name, path in dict(artifacts, runner=args.runner.resolve()).items()}}, indent=2) + '\n')

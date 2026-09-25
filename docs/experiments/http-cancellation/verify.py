"""Exercise HTTP tokens and text conversion with peer-controlled cancellation timing."""
import argparse
import os
from pathlib import Path
import shutil
import socket
import subprocess
import tempfile
import threading

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
parser.add_argument('--runner', type=Path, required=True)
parser.add_argument('--case', action='append', choices=['headers', 'body'])
args = parser.parse_args()
bundle = args.toolchain_root.resolve()
here = Path(__file__).resolve().parent
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
body = 'Café 🌍'.encode('utf-8')
head = b'HTTP/1.1 200 OK\r\nContent-Length: 10\r\n\r\n'

with tempfile.TemporaryDirectory(prefix='neoclr-http-cancellation-') as folder, socket.socket() as listener, socket.socket() as control:
    root = Path(folder)
    for sock in (listener, control):
        sock.bind(('127.0.0.1', 0))
        sock.listen(4)
        sock.settimeout(120)
    source = (here / 'Main.rvn').read_text().replace('19091', str(listener.getsockname()[1])).replace('19092', str(control.getsockname()[1]))
    (root / 'Main.rvn').write_text(source)
    shutil.copyfile(here / 'HttpCancellation.rvnproj', root / 'HttpCancellation.rvnproj')
    build = subprocess.run(['dotnet', 'msbuild', str(root / 'HttpCancellation.rvnproj'), '-nologo', '-v:minimal'],
                           env=env, capture_output=True, text=True, timeout=240)
    assert build.returncode == 0, build.stdout + build.stderr
    for case in args.case or ['headers', 'body']:
        errors = []
        def serve():
            peers = {}
            try:
                for _ in range(3):
                    peer = listener.accept()[0]
                    peer.settimeout(30)
                    request = bytearray()
                    while not request.endswith(b'\r\n\r\n'):
                        part = peer.recv(1)
                        assert part, 'EOF before request'
                        request.extend(part)
                        assert len(request) < 2048
                    target = request.split(b' ')[1].decode('ascii')
                    assert target in ['/cancel-response', '/cancel-text', '/keep'] and target not in peers, request
                    peers[target] = peer
                with control.accept()[0] as signal:
                    signal.settimeout(30)
                    if case == 'body':
                        for target in ('/cancel-response', '/cancel-text'):
                            peers[target].sendall(head + body[:1])
                    # Both HTTP requests reached this peer. No timing-based sleep decides cancellation.
                    signal.sendall(b'\x01')
                    assert signal.recv(1) == b'', 'Control connection was not closed'
                for target in ('/cancel-response', '/cancel-text'):
                    try:
                        assert peers[target].recv(1) == b'', 'Cancelled HTTP connection remained open'
                    except ConnectionResetError:
                        pass  # Closing with unread response bytes may reset rather than send FIN.
                # Same HttpClient, different token: cancelling the first two must not cancel this exchange.
                peers['/keep'].sendall(head + body)
                assert peers['/keep'].recv(1) == b'', 'Successful HTTP connection remained open'
            except BaseException as error:
                errors.append(error)
            finally:
                for peer in peers.values():
                    peer.close()
        worker = threading.Thread(target=serve, daemon=True)
        worker.start()
        run = subprocess.run([str(args.runner.resolve()), str(root / 'bin/neoclr/Debug/App.neoil'),
                              str(bundle / 'lib/System.neoil'), '256', '100000000'],
                             capture_output=True, text=True, timeout=180)
        assert run.returncode == 0, run.stdout + run.stderr
        worker.join(35)
        assert not worker.is_alive() and not errors, (errors, run.stdout, run.stderr)
        lines = run.stdout.splitlines()
        assert lines[0] == 'HTTP handler and text contracts passed', run.stdout
        assert sorted(lines[1:]) == sorted(['HTTP response cancelled', 'HTTP text cancelled', 'Independent HTTP text completed']), run.stdout
        assert 'live=0' in run.stderr, run.stderr
        print(case + ': peer observed both closes; independent text request completed', flush=True)
        print(run.stdout + run.stderr, flush=True)

    reference = root / 'reference'
    reference.mkdir()
    shutil.copyfile(here / 'Reference.cs', reference / 'Program.cs')
    (reference / 'Reference.csproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><ImplicitUsings>enable</ImplicitUsings></PropertyGroup></Project>')
    baseline = subprocess.run(['dotnet', 'run', '--project', str(reference / 'Reference.csproj')],
                              capture_output=True, text=True, timeout=120)
    assert baseline.returncode == 0, baseline.stdout + baseline.stderr
    print(baseline.stdout, flush=True)

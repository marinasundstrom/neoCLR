"""Cancel/close an HTTP accept with an incomplete incoming POST body."""
import argparse
import os
from pathlib import Path
import selectors
import socket
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
parser.add_argument('--runner', type=Path, required=True)
parser.add_argument('--source', type=Path, help='Override the probe source for a compiler/bridge reproduction')
args = parser.parse_args()
bundle = args.toolchain_root.resolve()
here = Path(__file__).resolve().parent
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
with tempfile.TemporaryDirectory(prefix='neoclr-http-context-cancel-') as temporary, socket.socket() as control:
    root = Path(temporary)
    control.bind(('127.0.0.1', 0))
    control.listen(2)
    control.settimeout(120)
    (root / 'Main.rvn').write_text((args.source or here / 'Cancellation.rvn').read_text().replace('19092', str(control.getsockname()[1])))
    (root / 'Probe.rvnproj').write_text((here / 'HttpContext.rvnproj').read_text().replace('<Compile Include="Sample.rvn" />', ''))
    compiled = subprocess.run(['dotnet', 'msbuild', str(root / 'Probe.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=240)
    assert compiled.returncode == 0, compiled.stdout + compiled.stderr
    for name, command in (('caller cancellation', 1), ('server shutdown', 2)):
        process = subprocess.Popen([str(args.runner.resolve()), str(root / 'bin/neoclr/Debug/App.neoil'), str(bundle / 'lib/System.neoil'), '256', '100000000', '--live-output'], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            with selectors.DefaultSelector() as selector:
                selector.register(process.stdout, selectors.EVENT_READ)
                assert selector.select(180), 'No server port'
            line = process.stdout.readline().strip()
            assert line.isdecimal(), (line, process.communicate(timeout=10))
            with socket.create_connection(('127.0.0.1', int(line)), timeout=30) as peer, control.accept()[0] as signal:
                peer.sendall(b'POST /partial HTTP/1.1\r\nHost: localhost\r\nContent-Length: 10\r\n\r\nx')
                signal.sendall(bytes([command]))
                signal.settimeout(30)
                assert signal.recv(1) == b'', 'Control connection retained'
                try:
                    assert peer.recv(1) == b'', 'Request connection retained'
                except ConnectionResetError:
                    pass
            output, errors = process.communicate(timeout=60)
            assert process.returncode == 0 and 'Pending request cancellation passed' in output and 'live=0' in errors, output + errors
            print(name + ': ' + output + errors, flush=True)
        finally:
            if process.poll() is None:
                process.kill()
                process.communicate()

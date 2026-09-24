#!/usr/bin/env python3
"""Run distinct neoCLR server/client processes on an OS-assigned loopback port."""
import argparse
import os
from pathlib import Path
import re
import selectors
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
parser.add_argument('--runner', type=Path, required=True)
args = parser.parse_args()
bundle = args.toolchain_root.resolve()
here = Path(__file__).resolve().parent
runtime = bundle / 'lib/System.neoil'
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))

def build(project):
    result = subprocess.run(['dotnet', 'msbuild', str(project), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=150)
    assert result.returncode == 0, result.stdout + result.stderr

def stats(stderr):
    result = {k: int(v) for k, v in re.findall(r'(\w+)=(\d+)', stderr)}
    assert result['live'] == 0 and result['collections'] >= 1, stderr
    return result

with tempfile.TemporaryDirectory(prefix='neoclr-two-sided-echo-') as folder:
    root = Path(folder)
    server_root = root / 'server'
    client_root = root / 'client'
    server_root.mkdir()
    client_root.mkdir()
    for name in ['Server.rvn', 'Server.rvnproj']:
        shutil.copyfile(here / name, server_root / name)
    build(server_root / 'Server.rvnproj')
    server = subprocess.Popen([str(bundle / 'bin/neoclr'), 'run', str(server_root / 'bin/neoclr/Debug/App.neoil'), '--system', str(runtime), '--gc-stats'], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        with selectors.DefaultSelector() as selector:
            selector.register(server.stdout, selectors.EVENT_READ)
            assert selector.select(60), 'Server did not report its port'
        port_line = server.stdout.readline().strip()
        assert port_line.isdecimal() and 0 < int(port_line) <= 65535, (port_line, server.poll())
        client_source = here / 'Client.rvn' if (here / 'Client.rvn').exists() else here.parent / 'socket-client/Main.rvn'
        source = client_source.read_text().replace('19090', port_line)
        (client_root / 'Main.rvn').write_text(source)
        client_project = here / 'Client.rvnproj' if (here / 'Client.rvnproj').exists() else here.parent / 'socket-client/SocketClient.rvnproj'
        (client_root / 'SocketClient.rvnproj').write_text(client_project.read_text().replace('Client.rvn', 'Main.rvn'))
        build(client_root / 'SocketClient.rvnproj')
        client = subprocess.run([str(args.runner.resolve()), str(client_root / 'bin/neoclr/Debug/App.neoil'), str(runtime), '256'], capture_output=True, text=True, timeout=120)
        assert client.returncode == 0, client.stdout + client.stderr
        output, errors = server.communicate(timeout=60)
        assert server.returncode == 0, output + errors
        assert output == 'Other work runs while accept is pending\nEchoed Hi\nServer closed\n', output
        assert client.stdout == 'Other work runs while TCP is pending\nResolved localhost\nSent Hi\nReceived Hi; peer finished sending\nSocket closed\n', client.stdout
        print('Server:', output, 'Client:', client.stdout, sep='\n')
        print('server GC:', stats(errors))
        print('client GC:', stats(client.stderr))
    finally:
        if server.poll() is None:
            server.kill()
            server.communicate()

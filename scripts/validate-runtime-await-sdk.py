#!/usr/bin/env python3
"""Qualify SDK-default runtime await through interpreter, installed JIT and AOT."""
import argparse
import hashlib
import http.server
import json
import os
from pathlib import Path
import platform
import shutil
import socket
import subprocess
import sys
import threading

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'benchmarks/framework'))
from process import Guest

BODY = 'Café 🌍'.encode()

class Peer(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    def do_GET(self):
        assert self.path == '/greeting'
        self.send_response(200)
        self.send_header('Content-Length', str(len(BODY)))
        self.end_headers()
        self.wfile.write(BODY)
    def log_message(self, *args):
        pass

def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ('bundle', 'runtime', 'aot', 'output'):
        p.add_argument('--' + name, type=Path, required=True)
    p.add_argument('--cases', nargs='+', choices=('language', 'cleanup', 'fault', 'server', 'client', 'state-machine', 'generator'), help='Run only selected cases; default is all')
    a = p.parse_args()
    out = a.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    bundle = a.bundle.resolve()
    windows = platform.system() == 'Windows'
    report = dict(passed=False, platform=platform.platform(), commands=[], cases=[], inputs={})
    def run(command, expected=0, **kwargs):
        command = list(map(str, command))
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=600, **kwargs)
        report['commands'].append(dict(command=command, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        if result.returncode != expected:
            raise RuntimeError(result.stderr or result.stdout)
        return result
    try:
        for path in [Path(__file__), *sorted((ROOT / 'runtime/raven/compiler-services').glob('*.rvn')), *sorted((ROOT / 'runtime/native').rglob('*.[ch]'))]:
            report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
        cases = [('language', 'docs/experiments/runtime-async/RuntimeAwait.rvn'),
                 ('cleanup', 'docs/experiments/runtime-async/RuntimeUse.rvn'),
                 ('fault', 'docs/experiments/runtime-async/RuntimeAwaitFault.rvn'),
                 ('server', 'docs/experiments/http-server/Server.rvn'),
                 ('client', 'docs/experiments/native-http-client/Client.rvn'),
                 ('state-machine', None),
                 ('generator', 'docs/experiments/runtime-async/RuntimeGeneratorLanguage.rvn')]
        if a.cases:
            cases = [(name, source) for name, source in cases if name in a.cases]
        installed = None
        for name, source in cases:
            project = out / name
            project.mkdir()
            if source:
                shutil.copyfile(ROOT / source, project / 'Main.rvn')
                report['inputs'][source] = hashlib.sha256((ROOT / source).read_bytes()).hexdigest()
            else:
                (project / 'Main.rvn').write_text('import System.*\nimport System.Concurrency.Tasks.*\nasync func Main() -> Task<()> {\n    await Task.CompletedTask\n    Console.WriteLine("PASS explicit state-machine fallback")\n}\n')
            fallback = '<RavenNeoClrRuntimeAsyncHelper></RavenNeoClrRuntimeAsyncHelper>' if name == 'state-machine' else ''
            (project / 'App.rvnproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><Import Project="$(NeoClrBundleRoot)/lib/NeoCLR.ClassLibrary.props"/><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><AssemblyName>SdkAwait</AssemblyName>' + fallback + '</PropertyGroup></Project>')
            profile = 'console' if name in ('cleanup', 'state-machine') else 'http'
            run([sys.executable, ROOT / 'scripts/build-native-project.py', '--profile', ('windows-' if windows else '') + profile,
                 '--project', project / 'App.rvnproj', '--bundle', bundle, '--aot', a.aot.resolve(), '--output', project / 'native'])
            assembly = project / 'native/app.dll'
            listing = project / 'app.neoil'
            run([a.runtime.resolve(), 'disassemble', assembly, listing])
            has_machine = 'AsyncStateMachine' in listing.read_text()
            if has_machine != (name == 'state-machine'):
                raise RuntimeError('Unexpected lowering: ' + name)
            if installed is None:
                run([sys.executable, ROOT / 'scripts/prepare-jit-bundle.py', '--bundle', bundle, '--runtime', a.runtime.resolve(),
                     '--jit-compiler', a.aot.resolve(), '--host-recipe', project / 'native/build.json', '--output', out / 'installed'])
                installed = out / 'installed/bundle'
            vm = installed / ('bin/neoclr.exe' if windows else 'bin/neoclr')
            catalog = json.loads((installed / 'lib/bundle.json').read_text())
            libraries = installed / 'lib'
            interpreted_context = ['--system', libraries / catalog['runtimeSeed'], '--object-root', libraries / 'System.Runtime.dll']
            for module in catalog['assemblyNames']:
                interpreted_context += ['--module', libraries / (module + '.dll')]
            invocations = {'aot': [project / ('native/app.exe' if windows else 'native/app')],
                           'interpreter': [vm, 'run', assembly, '--execution', 'interpreter', '--instructions', '100000000', *interpreted_context],
                           'jit': [vm, 'run', assembly]}
            for mode, command in invocations.items():
                command = list(map(str, command))
                if name == 'server':
                    guest = Guest(command, timeout=180)
                    try:
                        port, _ = guest.line()
                        with socket.create_connection(('127.0.0.1', int(port)), timeout=30) as peer:
                            peer.sendall(b'GET /greeting HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n')
                            response = b''
                            while True:
                                part = peer.recv(4096)
                                if not part: break
                                response += part
                                if len(response) > 8192: raise ValueError('Oversized HTTP response')
                        assert response.startswith(b'HTTP/1.1 200 OK\r\n') and response.split(b'\r\n\r\n', 1)[1] == BODY
                        guest.finish()
                        assert b'Served greeting; server closed' in b''.join(guest.output)
                    finally:
                        guest.close()
                elif name == 'client':
                    with http.server.ThreadingHTTPServer(('127.0.0.1', 0), Peer) as peer:
                        worker = threading.Thread(target=peer.serve_forever, daemon=True)
                        worker.start()
                        try:
                            result = run(command, input=f'http://localhost:{peer.server_port}/greeting\nnormal\n')
                            assert result.stdout.strip() == BODY.decode() and not result.stderr
                        finally:
                            peer.shutdown()
                            worker.join(timeout=5)
                else:
                    result = run(command, expected=1 if name == 'fault' else 0)
                    if name == 'fault':
                        assert 'UserFault: runtime async sample terminal fault' in result.stderr and not result.stdout
                    else:
                        assert result.stdout.startswith('PASS ') and not result.stderr
                report['cases'].append(dict(name=name, mode=mode, passed=True, command=command))
                print('PASS', name, mode, flush=True)
        for path, digest in report['inputs'].items():
            assert hashlib.sha256(Path(path).read_bytes()).hexdigest() == digest, path
        report['passed'] = True
    except Exception as error:
        report['error'] = str(error)
        raise
    finally:
        (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')

if __name__ == '__main__':
    main()

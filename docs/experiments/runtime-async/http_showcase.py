#!/usr/bin/env python3
"""Qualify the existing HTTP showcase sources using runtime-await application lowering."""
import argparse
import hashlib
import http.client
import http.server
import json
from pathlib import Path
import selectors
import subprocess
import threading

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--recipe', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
recipe = json.loads(args.recipe.read_text())
if not recipe['passed']:
    raise SystemExit('Requires a passing runtime-await HTTP recipe')
output = args.output.resolve()
output.mkdir(parents=True, exist_ok=True)
root = Path(__file__).resolve().parents[3]
provider = Path(__file__).with_name('RuntimeAsyncBridge.rvn').resolve()
compile_base = recipe['commands'][0]['command']
run_base = recipe['commands'][1]['command']
inputs = [provider, Path(compile_base[1]), Path(run_base[0]), *[Path(compile_base[i + 1]) for i, arg in enumerate(compile_base) if arg in ('--reference', '--core-reference', '--runtime-seed', '--bootstrap-ownership')]]
inputs += list(Path(compile_base[1]).parent.glob('*.dll'))
report = {'inputsSha256': {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, 'passed': False, 'recipe': str(args.recipe.resolve()), 'recipeSha256': hashlib.sha256(args.recipe.read_bytes()).hexdigest(), 'cases': []}
body = 'Café 🌍'.encode()


def compile_case(name, source):
    assembly = output / (name + '.dll')
    assembly.unlink(missing_ok=True)
    command = compile_base[:compile_base.index('-o')] + ['-o', str(assembly), str(source), str(provider)]
    result = subprocess.run(command, capture_output=True, text=True, timeout=120)
    case = {'name': name, 'source': str(source), 'sourceSha256': hashlib.sha256(source.read_bytes()).hexdigest(),
            'compileCommand': command, 'compileExit': result.returncode, 'compileStderr': result.stderr}
    report['cases'].append(case)
    if result.returncode:
        raise RuntimeError(result.stderr)
    case['assemblySha256'] = hashlib.sha256(assembly.read_bytes()).hexdigest()
    listing = output / (name + '.neoil')
    listing.unlink(missing_ok=True)
    subprocess.run([run_base[0], 'disassemble', str(assembly), str(listing)], check=True, capture_output=True, timeout=120)
    case['stateMachineMentions'] = listing.read_text().count('AsyncStateMachine')
    if case['stateMachineMentions']:
        raise RuntimeError('Application emitted state machines')
    command = run_base.copy()
    command[2] = str(assembly)
    case['runCommand'] = command
    return case, command


class Peer(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    paths = []
    def do_GET(self):
        self.paths.append(self.path)
        self.send_response(200)
        self.send_header('Content-Type', 'text/plain; charset=utf-8')
        self.send_header('Content-Length', str(len(body)))
        self.send_header('Connection', 'close')
        self.end_headers()
        self.wfile.write(body)
    def log_message(self, *args):
        pass


try:
    case, command = compile_case('client', root / 'docs/experiments/native-http-client/Client.rvn')
    with http.server.ThreadingHTTPServer(('127.0.0.1', 0), Peer) as peer:
        worker = threading.Thread(target=peer.serve_forever, daemon=True)
        worker.start()
        try:
            url = f'http://localhost:{peer.server_port}/greeting'
            result = subprocess.run(command, input=url + '\nnormal\n', capture_output=True, text=True, timeout=90)
        finally:
            peer.shutdown()
            worker.join(timeout=5)
    case.update(exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr, requestPaths=Peer.paths, inputUrl=url)
    case['passed'] = result.returncode == 0 and result.stdout.strip() == body.decode() and not result.stderr and Peer.paths == ['/greeting']
    if not case['passed']:
        raise RuntimeError('Client showcase failed')

    case, command = compile_case('server', root / 'docs/experiments/http-server/Server.rvn')
    process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        with selectors.DefaultSelector() as selector:
            selector.register(process.stdout, selectors.EVENT_READ)
            if not selector.select(timeout=60):
                raise TimeoutError('Server did not publish its port')
            port_line = process.stdout.readline()
        port = int(port_line.strip())
        connection = http.client.HTTPConnection('127.0.0.1', port, timeout=30)
        try:
            connection.request('GET', '/greeting')
            response = connection.getresponse()
            received = response.read()
            status = response.status
        finally:
            connection.close()
        stdout, stderr = process.communicate(timeout=60)
        case.update(exitCode=process.returncode, stdout=port_line + stdout, stderr=stderr, status=status, body=received.decode())
        case['passed'] = (process.returncode == 0 and not stderr and status == 200 and received == body
                          and 'Other work runs while HTTP accept is pending' in stdout
                          and 'Served greeting; server closed' in stdout)
        if not case['passed']:
            raise RuntimeError('Server showcase failed')
    finally:
        if process.poll() is None:
            process.kill()
            process.communicate(timeout=5)
    report['passed'] = True
except Exception as error:
    report['error'] = str(error)
finally:
    (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report, indent=2))
raise SystemExit(0 if report['passed'] else 1)

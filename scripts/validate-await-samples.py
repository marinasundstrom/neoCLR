#!/usr/bin/env python3
"""Compile and execute the website's await-first Task and HTTP client examples."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

ROOT = Path(__file__).resolve().parents[1]
def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    bundle = args.bundle.resolve(); out = args.output.resolve(); out.mkdir(parents=True, exist_ok=False)
    lib = bundle / 'lib'; compiler = bundle / 'sdk/tools/rvnc/rvnc.dll'; vm = bundle / 'bin/neoclr'
    catalog = json.loads((lib / 'bundle.json').read_text())
    libraries = [lib / (name + '.dll') for name in catalog['assemblyNames']]
    context = ['--system', lib / catalog['runtimeSeed'], *[arg for p in libraries for arg in ('--module', p)], '--object-root', lib / 'System.Runtime.dll']
    report = dict(passed=False, cases=[], inputs={})
    def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
    def run(command, name):
        r = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, timeout=120)
        (out / (name + '.stdout')).write_bytes(r.stdout); (out / (name + '.stderr')).write_bytes(r.stderr)
        if r.returncode: raise RuntimeError(name + ': ' + r.stderr.decode())
        return r.stdout.decode()
    def compile_case(name, paths, replacements=None):
        sources = []
        for rel in paths:
            p = ROOT / 'docs/experiments' / rel; report['inputs'][str(p.relative_to(ROOT))] = sha(p)
            text = p.read_text()
            for before, after in (replacements or {}).items(): text = text.replace(before, after)
            target = out / (name + '-' + p.name); target.write_text(text); sources.append(target)
        dll = out / (name + '.dll')
        run(['dotnet', compiler, 'neoclr', '--core-reference', lib / 'Core.dll', '--runtime-seed', lib / catalog['runtimeSeed'],
             *[arg for p in libraries for arg in ('--reference', p)], '--bootstrap-intrinsics', '--bootstrap-ownership', lib / 'ownership.json',
             '--object-library', 'System.Runtime', '--async-library', 'System.Runtime', '-o', dll, *sources], name + '-compile')
        return dll
    try:
        for p in [Path(__file__).resolve(), compiler, vm, lib / 'Core.dll', lib / catalog['runtimeSeed'], lib / 'ownership.json', *libraries]:
            report['inputs'][str(p)] = sha(p)
        for name, source, expected in [('producer','library-task-producer.rvn','42\n'), ('result','library-task-result.rvn','42\n'), ('cancellation','library-async-cancellation.rvn','Cancelled\n')]:
            dll = compile_case(name, ['raven-target/samples/' + source])
            output = run([vm, 'run', dll, *context], name + '-run')
            if output != expected: raise ValueError(name + ': unexpected output')
            report['cases'].append(dict(name=name, passed=True)); print(name + ': PASS', flush=True)
        for name, sources in [
            ('overview', ['http-client/Overview.rvn']),
            ('json', ['http-json/Client.rvn','http-json/Application.rvn']),
            ('mapped', ['json-object-mapping/HttpClient.rvn','json-object-mapping/HttpApplication.rvn']),
            ('routed', ['http-routing/Client.rvn','json-object-mapping/NestedHttpApplication.rvn']),
        ]:
            wire = []
            payload = {'station': {'name':'Café','description':'Sensor'}, 'readings':[21,22]} if name == 'routed' else {'station':'Café'}
            class Peer(BaseHTTPRequestHandler):
                protocol_version = "HTTP/1.1"
                def log_message(self, *args): pass
                def reply(self, status, body, kind):
                    self.send_response(status); self.send_header('Content-Length',str(len(body))); self.send_header('Content-Type',kind); self.send_header('Connection','close'); self.end_headers(); self.wfile.write(body)
                def do_GET(self):
                    wire.append(['GET',self.path])
                    body = 'Café 🌍'.encode() if name == 'overview' else json.dumps(payload,ensure_ascii=False).encode()
                    self.reply(200,body,'text/plain; charset=utf-8' if name == 'overview' else 'application/json')
                def do_POST(self):
                    body = self.rfile.read(int(self.headers['Content-Length']))
                    wire.append(['POST',self.path,json.loads(body)])
                    self.reply(201,b'{"accepted":true}','application/json')
            peer = ThreadingHTTPServer(('127.0.0.1',0),Peer); thread = threading.Thread(target=peer.serve_forever); thread.start()
            try:
                dll = compile_case(name,sources,{'19091':str(peer.server_port)})
                output = run([vm,'run',dll,*context,'--instructions','100000000',*(['--',f'http://localhost:{peer.server_port}/'] if name != 'overview' else [])],name+'-run')
                if name == 'overview':
                    if output != 'HTTP 200\nCafé 🌍\n' or wire != [['GET','/greeting']]: raise ValueError('Overview mismatch')
                else:
                    path = '/stations/42/reports' if name == 'routed' else '/reports'
                    expected = ([['GET','/stations/42/reports' if name == 'routed' else '/report']] if name != 'json' else []) + [['POST',path,payload]]
                    if json.loads(output) != {'accepted':True} or wire != expected: raise ValueError(name + ': HTTP mismatch ' + repr(wire))
            finally: peer.shutdown(); peer.server_close(); thread.join()
            report['cases'].append(dict(name=name,passed=True,requests=wire)); print(name + ': PASS',flush=True)
            if name != 'overview':
                server_sources = [sources[0].replace('Client.rvn','Server.rvn'), sources[1]]
                if name == 'routed': server_sources.append('http-routing/Routes.rvn')
                server_dll = compile_case(name + '-server', server_sources)
                command = [vm,'run',server_dll,*context,'--instructions','100000000','--','1' if name == 'json' else '2']
                process = subprocess.Popen(list(map(str,command)),stdout=subprocess.PIPE,stderr=subprocess.PIPE)
                try:
                    # The fixture reader is bounded and preserves a partial line on EOF.
                    import importlib.util
                    spec = importlib.util.spec_from_file_location('http_project',ROOT/'scripts/validate-native-http-project.py')
                    helper = importlib.util.module_from_spec(spec); spec.loader.exec_module(helper)
                    port = helper.read_port(process.stdout,timeout=20)
                    if not port.isdigit():
                        raise ValueError(name + ': server missing port: ' + process.stderr.read().decode())
                    output = run([vm,'run',dll,*context,'--instructions','100000000','--',f'http://localhost:{int(port)}/'],name+'-pair-client')
                    stdout, stderr = process.communicate(timeout=30)
                    if process.returncode or stdout != b'Reports served\n' or stderr or json.loads(output) != {'accepted':True}:
                        raise ValueError(name + ': pair failed: ' + repr((stdout,stderr,output)))
                finally:
                    if process.poll() is None: process.kill(); process.communicate()
                report['cases'].append(dict(name=name+'-pair',passed=True)); print(name+'-pair: PASS',flush=True)

        report['passed'] = len(report['cases']) == 10
    except Exception as error: report['error'] = str(error)
    finally:
        report['files'] = {p.relative_to(out).as_posix():sha(p) for p in out.rglob('*') if p.is_file()}
        (out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    if not report['passed']: print(report.get('error','Incomplete'),flush=True)
    return 0 if report['passed'] else 1
if __name__ == '__main__': raise SystemExit(main())

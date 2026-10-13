#!/usr/bin/env python3
"""Qualify explicit runtime-await bodies with the shared macOS ARM64 JIT/AOT host."""
import argparse
import hashlib
import http.client
import http.server
import json
import os
from pathlib import Path
import platform
import selectors
import shutil
import socket
import subprocess
import threading

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
BODY = 'Café 🌍'.encode()


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def receive_request(connection):
    connection.settimeout(20)
    data = b''
    while b'\r\n\r\n' not in data:
        part = connection.recv(4096)
        if not part or len(data) > 8192:
            raise RuntimeError('Incomplete/oversized request')
        data += part
    if not data.startswith(b'GET /greeting HTTP/1.1\r\n'):
        raise RuntimeError('Unexpected HTTP request')


def lifecycle_peer(listener, scenario, observed):
    try:
        with listener.accept()[0] as connection:
            receive_request(connection)
            observed['acceptedRequest'] = True
            if scenario == 'protocol':
                connection.sendall(b'not-an-http-response\r\n\r\n')
            elif scenario == 'fault':
                connection.sendall(b'HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok')
            observed['firstSocketClosed'] = connection.recv(1) == b''
        if scenario != 'fault':
            with listener.accept()[0] as connection:
                receive_request(connection)
                observed['recoveryRequest'] = True
                connection.sendall(b'HTTP/1.1 200 OK\r\nContent-Length: 9\r\nConnection: close\r\n\r\nrecovered')
                observed['recoverySocketClosed'] = connection.recv(1) == b''
    except Exception as error:
        observed['error'] = str(error)


class Peer(http.server.BaseHTTPRequestHandler):
    protocol_version = 'HTTP/1.1'

    def do_GET(self):
        self.server.paths.append(self.path)
        self.send_response(200)
        self.send_header('Content-Length', str(len(BODY)))
        self.send_header('Connection', 'close')
        self.end_headers()
        self.wfile.write(BODY)

    def log_message(self, *args):
        pass


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--recipe', type=Path, required=True, help='Passing runtime-await HTTP source validation.json')
    parser.add_argument('--native-build', type=Path, required=True, help='Passing macOS HTTP build.json, used only as a command recipe')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--backend', type=Path, default=ROOT / 'target/release/neoclr-aot-poc')
    parser.add_argument('--text-archive', type=Path, default=ROOT / 'tools/aot-native-text/target/text-only/aarch64-apple-darwin/release/libneoclr_aot_native_text.a')
    args = parser.parse_args()
    if platform.system() != 'Darwin' or platform.machine() != 'arm64':
        raise RuntimeError('The experimental stack carrier requires macOS ARM64')
    source_recipe = json.loads(args.recipe.read_text())
    native_recipe = json.loads(args.native_build.read_text())
    if not source_recipe['passed'] or not native_recipe['passed'] or native_recipe['profile'] != 'macos-arm64-http-v1':
        raise RuntimeError('Requires passing source and native HTTP command recipes')
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    report = {'passed': False, 'platform': platform.platform(), 'commands': [], 'cases': [], 'applications': {},
              'recipeSha256': sha(args.recipe), 'nativeRecipeSha256': sha(args.native_build),
              'inputsSha256': {}}

    def record_inputs(command):
        for item in command:
            path = Path(item)
            if path.is_file() and not path.is_relative_to(out):
                report['inputsSha256'][str(path.resolve())] = sha(path)

    def run(command, label, expected=0, **kwargs):
        command = list(map(str, command))
        record_inputs(command)
        result = subprocess.run(command, capture_output=True, text=True, timeout=120, cwd=ROOT, **kwargs)
        row = dict(label=label, command=command, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr)
        report['commands'].append(row)
        if result.returncode != expected:
            raise RuntimeError(label + ': ' + result.stderr)
        if label.endswith('-native-compile'):
            artifact = out / (label + '.json')
            artifact.write_text(row.pop('stdout'))
            selection = json.loads(artifact.read_text())
            row.update(stdoutFile=str(artifact), stdoutSha256=sha(artifact),
                       functionCount=len(selection['functions']), typeCount=len(selection['types']),
                       runtimeAsyncBindings=[b for b in selection['nativeBindings']
                                             if b.get('implementation', '').startswith('runtime-async-')])
        return row

    try:
        compile_source = source_recipe['commands'][0]['command']
        interpreter = source_recipe['commands'][1]['command'][0]
        bundle = Path(compile_source[compile_source.index('--core-reference') + 1]).parent.parent
        compile_native = next(c['command'] for c in native_recipe['commands'] if c.get('phase') == 'native-compilation').copy()
        compile_native[0] = str(args.backend.resolve())
        for option in ['--bind-scheduler', '--bind-runtime-async']:
            if option not in compile_native:
                compile_native.append(option)
        original_link = next(c['command'] for c in native_recipe['commands'] if c.get('phase') == 'host-compile-link')
        original_object, original_app = compile_native[4], original_link[original_link.index('-o') + 1]
        cc = original_link[:original_link.index('-I')]
        native = ROOT / 'docs/experiments/aot-console'
        for name, sources in [
            ('gc-contract', ['native-gc-test.c', 'native-gc.c', 'root-probe.c']),
            ('stack-contract', ['native-stack-test.c', 'native-stack.c']),
        ]:
            executable = out / name
            run([*cc, *[native / source for source in sources], '-o', executable], name + '-build')
            run([executable], name + '-run')

        link = [item for item in original_link if item != '-DNEOCLR_ENTRY_ARGUMENTS']
        link += ['-DNEOCLR_RUNTIME_ASYNC', '-I', str(HERE), str(HERE / 'native-runtime.c'), '-Wno-deprecated-declarations']
        unicode_source = str(ROOT / 'docs/experiments/aot-console/string-unicode.c')
        if unicode_source not in link:
            link.append(unicode_source)
        if str(args.text_archive.resolve()) not in link:
            link.append(str(args.text_archive.resolve()))
        host = out / 'host.dylib'
        jit_link = [str(ROOT / 'docs/experiments/jit-managed/host.c') if item.endswith('/http-host.c')
                    else str(host) if item == original_app else item for item in link if item != original_object]
        run([*jit_link, '-dynamiclib'], 'jit-host-build')
        for folder in [HERE, ROOT / 'docs/experiments/aot-console', ROOT / 'docs/experiments/aot-fault-details']:
            for path in folder.glob('*.h'):
                report['inputsSha256'][str(path)] = sha(path)
        for path in (ROOT / 'tools/aot-poc/src').glob('*.rs'):
            report['inputsSha256'][str(path)] = sha(path)
        report['inputsSha256'][str(Path(__file__).resolve())] = sha(__file__)
        for path in Path(compile_source[1]).parent.glob('*.dll'):
            report['inputsSha256'][str(path)] = sha(path)
        applications = [
            ('language', HERE / 'RuntimeAwait.rvn'), ('use', HERE / 'RuntimeUse.rvn'),
            ('contract', HERE / 'RuntimeNativeContract.rvn'),
            ('stress', HERE / 'RuntimeAwaitBenchmark.rvn'),
            ('stack-reuse', HERE / 'RuntimeStackReuse.rvn'),
            ('fault', HERE / 'RuntimeAwaitFault.rvn'), ('lifecycle', HERE / 'RuntimeHttpLifecycle.rvn'),
            ('client', ROOT / 'docs/experiments/native-http-client/Client.rvn'),
            ('server', ROOT / 'docs/experiments/http-server/Server.rvn'),
        ]
        for name, source in applications:
            assembly, obj, app = out / (name + '.dll'), out / (name + '.o'), out / name
            if name in ('client', 'server'):
                project = source.with_name('Native.rvnproj')
                env = os.environ.copy()
                env.update(NeoClrBundleRoot=str(bundle), RavenNeoClrRuntimeAsyncHelper='RuntimeAwaitProvider.Bridge')
                artifact = source.parent / 'bin/neoclr/HttpShowcase.dll'
                artifact.unlink(missing_ok=True)
                row = run([*compile_source[:2], 'neoclr', '--project', project], name + '-project', env=env)
                row['environment'] = {k: env[k] for k in ['NeoClrBundleRoot', 'RavenNeoClrRuntimeAsyncHelper']}
                shutil.copyfile(artifact, assembly)
            else:
                run([*compile_source[:compile_source.index('-o')], '-o', assembly, source, HERE / 'RuntimeAsyncBridge.rvn'], name + '-source')
            report['inputsSha256'][str(source)] = sha(source)
            listing = out / (name + '.neoil')
            run([interpreter, 'disassemble', assembly, listing], name + '-listing')
            if 'AsyncStateMachine' in listing.read_text():
                raise RuntimeError(name + ' emitted application state machines')
            command = compile_native.copy()
            command[2], command[4] = str(assembly), str(obj)
            run(command, name + '-native-compile')
            app_link = [str(obj) if x == original_object else str(app) if x == original_app else x for x in link]
            if name == 'lifecycle':
                app_link.append('-DNEOCLR_ENTRY_ARGUMENTS')
            run(app_link, name + '-link')
            jit = [command[0], '--jit-run', *command[1:]]
            jit[5] = str(host)
            report['applications'][name] = {'aot': [str(app)], 'jit': jit, 'assemblySha256': sha(assembly),
                                             'executableSha256': sha(app), 'stateMachineMentions': 0}
            if name == 'contract':
                interpreted = source_recipe['commands'][1]['command'].copy()
                if '--execution' not in interpreted:
                    interpreted += ['--execution', 'interpreter']
                interpreted[2] = str(assembly)
                row = run(interpreted, 'contract-interpreter')
                if row['stdout'].strip() != 'PASS runtime async operation lifetime contract' or row['stderr']:
                    raise RuntimeError('Interpreter lifetime contract differs')
            if name in ('language', 'use', 'stack-reuse'):
                repeated = out / (name + '-repeat')
                repeat_link = [str(repeated) if x == str(app) else x for x in app_link]
                run([*repeat_link, '-DNEOCLR_HTTP_INVOCATIONS=4'], name + '-repeat-link')
                row = run([repeated], name + '-same-heap-repeat')
                if row['stdout'].count('PASS ') != 4 or row['stderr']:
                    raise RuntimeError('Repeated invocation failed')
        for mode in ('aot', 'jit'):
            def invocation(name, arguments=()):
                command = report['applications'][name][mode]
                return command + (list(arguments) if mode == 'aot' else [x for a in arguments for x in ['--jit-entry-argument', a]])
            for name in ('language', 'use', 'contract', 'stress', 'stack-reuse', 'fault'):
                row = run(invocation(name), mode + '-' + name, expected=1 if name == 'fault' else 0)
                good = not row['stdout'] and not row['stderr'] if name == 'stress' else ('UserFault: runtime async sample terminal fault' in row['stderr'] and not row['stdout'] if name == 'fault' else row['stdout'].startswith('PASS ') and not row['stderr'])
                report['cases'].append(dict(mode=mode, case=name, passed=good))
                if not good:
                    raise RuntimeError(mode + '-' + name + ' output mismatch')
            with http.server.ThreadingHTTPServer(('127.0.0.1', 0), Peer) as peer:
                peer.paths = []
                worker = threading.Thread(target=peer.serve_forever, daemon=True)
                worker.start()
                try:
                    row = run(invocation('client'), mode + '-client', input=f'http://localhost:{peer.server_port}/greeting\nnormal\n')
                finally:
                    peer.shutdown()
                    worker.join(timeout=5)
            good = row['stdout'].strip() == BODY.decode() and not row['stderr'] and peer.paths == ['/greeting']
            report['cases'].append(dict(mode=mode, case='client-project', passed=good, requestPaths=peer.paths))
            if not good:
                raise RuntimeError('Client project failed')
            process = subprocess.Popen(invocation('server'), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            try:
                with selectors.DefaultSelector() as selector:
                    selector.register(process.stdout, selectors.EVENT_READ)
                    if not selector.select(timeout=30):
                        raise TimeoutError('Server did not publish its port')
                    port_line = process.stdout.readline()
                peer = http.client.HTTPConnection('127.0.0.1', int(port_line.strip()), timeout=30)
                try:
                    peer.request('GET', '/greeting')
                    response = peer.getresponse()
                    received, status = response.read(), response.status
                finally:
                    peer.close()
                stdout, stderr = process.communicate(timeout=30)
                good = process.returncode == 0 and not stderr and status == 200 and received == BODY and 'Other work runs while HTTP accept is pending' in stdout and 'Served greeting; server closed' in stdout
                report['cases'].append(dict(mode=mode, case='server-project', passed=good, exitCode=process.returncode, stdout=port_line + stdout, stderr=stderr, status=status, body=received.decode()))
                if not good:
                    raise RuntimeError('Server project failed')
            finally:
                if process.poll() is None:
                    process.kill()
                    process.communicate()
            for scenario in ('timeout', 'cancel', 'protocol', 'fault'):
                observed = {}
                with socket.socket() as listener:
                    listener.bind(('127.0.0.1', 0))
                    listener.listen()
                    listener.settimeout(30)
                    worker = threading.Thread(target=lifecycle_peer, args=(listener, scenario, observed), daemon=True)
                    worker.start()
                    url = f'http://127.0.0.1:{listener.getsockname()[1]}/greeting'
                    try:
                        row = run(invocation('lifecycle', [url, scenario, url]), mode + '-http-' + scenario, expected=1 if scenario == 'fault' else 0)
                    finally:
                        worker.join(timeout=35)
                good = not observed.get('error') and observed.get('firstSocketClosed')
                if scenario == 'fault':
                    good = good and 'UserFault: runtime HTTP continuation fault' in row['stderr'] and not row['stdout']
                else:
                    good = good and observed.get('recoverySocketClosed') and 'PASS runtime HTTP lifecycle and recovery' in row['stdout'] and not row['stderr']
                report['cases'].append(dict(mode=mode, case='http-' + scenario, passed=bool(good), observations=observed))
                if not good:
                    raise RuntimeError('HTTP lifecycle failed: ' + repr(observed))
        report['hostSha256'] = sha(host)
        report['passed'] = True
    finally:
        (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS native runtime-await: JIT and AOT language, cleanup, HTTP projects and lifecycle')


if __name__ == '__main__':
    main()

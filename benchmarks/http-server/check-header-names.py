#!/usr/bin/env python3
"""Exercise byte-matched and fallback request headers through the public server API."""
import argparse
import json
import os
from pathlib import Path
import socket
import subprocess
import benchmark

ROOT = Path(__file__).resolve().parents[2]


def cases():
    rows = []
    def add(name, fields, error=None, body=b'', fragmented=False):
        request = b'POST /headers HTTP/1.1\r\n' + fields + b'\r\n' + body
        expected = b''.join(k.lower() + b'=' + v.strip(b' \t') + b'\n'
                            for k, v in (line.split(b':', 1) for line in fields.split(b'\r\n') if line))
        rows.append(dict(name=name, request=request, expected=expected, error=error, fragmented=fragmented))
    add('mixed-known', b'hOsT: localhost\r\ncOnNeCtIoN: close\r\ncOnTeNt-LeNgTh: 0\r\nCoNtEnT-TyPe: text/plain\r\n', fragmented=True)
    add('known-uppercase', b'HOST: localhost\r\nCONNECTION: close\r\nCONTENT-LENGTH: 0\r\nCONTENT-TYPE: text/plain\r\n')
    catalog = b'Host: localhost\r\nConnection: close\r\nContent-Length: 0\r\nContent-Type: text/plain\r\nAccept: */*\r\nAccept-Encoding: identity\r\nAuthorization: Example token\r\nCookie: a=b\r\nUser-Agent: test\r\nCache-Control: no-cache\r\n'
    add('catalog-first', catalog)
    add('catalog-uppercase', b'\r\n'.join(line.split(b':', 1)[0].upper() + b':' + line.split(b':', 1)[1] for line in catalog.split(b'\r\n') if line) + b'\r\n')
    add('catalog-fragmented', catalog, fragmented=True)
    add('chunked', b'Host: localhost\r\nTrAnSfEr-EnCoDiNg: chunked\r\n', body=b'0\r\n\r\n', fragmented=True)
    add('unknown-and-empty', b'Host: localhost\r\nX-Custom:  two words \t\r\nX-Empty:\r\nX-Custom: second\r\n')
    # Exact-length lookalikes exercise failed dispatch without changing the fallback spelling.
    for original in ['host', 'connection', 'content-length', 'transfer-encoding', 'expect', 'content-encoding', 'content-type', 'accept', 'accept-encoding', 'authorization', 'cookie', 'user-agent', 'cache-control']:
        near = original[:-1] + 'x'
        add('near-' + original, b'Host: localhost\r\n' + near.upper().encode() + b': value\r\n')
        add('long-' + original, b'Host: localhost\r\n' + original.upper().encode() + b'X: value\r\n')
    for name in ['ExPeCt', 'CoNtEnT-EnCoDiNg']:
        add('unsupported-' + name, b'Host: localhost\r\n' + name.encode() + b': identity\r\n', 'Request expectation or content encoding unsupported')
    add('duplicate-host', b'Host: localhost\r\nhOsT: other\r\n', 'Duplicate Host')
    add('duplicate-length', b'Host: localhost\r\nContent-Length: 0\r\ncOnTeNt-LeNgTh: 0\r\n', 'Duplicate request Content-Length')
    add('duplicate-transfer', b'Host: localhost\r\nTransfer-Encoding: chunked\r\ntRaNsFeR-EnCoDiNg: chunked\r\n', 'Duplicate request Transfer-Encoding')
    add('ambiguous', b'Host: localhost\r\nContent-Length: 0\r\nTransfer-Encoding: chunked\r\n', 'Ambiguous request framing')
    add('invalid-length', b'Host: localhost\r\nCONTENT-LENGTH: +0\r\n', 'Invalid request Content-Length')
    add('invalid-transfer', b'Host: localhost\r\nTRANSFER-ENCODING: identity\r\n', 'Request transfer encoding unsupported')
    add('empty-host', b'HOST: \t\r\n', 'Empty Host')
    add('space-in-name', b'Host: localhost\r\nContent-Length : 0\r\n', 'Invalid request header name')
    add('underscore-not-hyphen', b'Host: localhost\r\nContent_Length: 0\r\n')
    return rows


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--bundle', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--aot', type=Path, required=True)
    a = p.parse_args()
    out = a.output.resolve(); out.mkdir(parents=True, exist_ok=False)
    bundle = a.bundle.resolve(); rows = cases()
    report = dict(passed=False, cases=[], commands=[], inputs={})
    def run(cmd):
        result = subprocess.run(list(map(str, cmd)), env=dict(os.environ, NeoClrBundleRoot=str(bundle)), capture_output=True, text=True, timeout=600)
        report['commands'].append(dict(command=result.args, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        if result.returncode:
            raise RuntimeError(result.stderr)
    try:
        project = out / 'project'; project.mkdir()
        source = Path(__file__).with_name('HeaderNames.rvn')
        (project / 'Main.rvn').write_bytes(source.read_bytes())
        (project / 'Cases.rvn').write_text(f'class HeaderCases {{ static val Count: int => {len(rows)} }}\n')
        (project / 'App.rvnproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><Import Project="$(NeoClrBundleRoot)/lib/NeoCLR.ClassLibrary.props"/><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><AssemblyName>HeaderNames</AssemblyName></PropertyGroup></Project>')
        native = out / 'native'
        run(['python3', ROOT / 'scripts/build-native-project.py', '--profile', 'windows-http' if os.name == 'nt' else 'http', '--project', project / 'App.rvnproj', '--bundle', bundle, '--aot', a.aot.resolve(), '--output', native])
        manifest = json.loads((bundle / 'manifest.json').read_text())
        lib = bundle / 'lib'; catalog = json.loads((lib / 'bundle.json').read_text())
        context = ['--system', str(lib / catalog['runtimeSeed']), '--object-root', str(lib / 'System.Runtime.dll')]
        for name in catalog['assemblyNames']: context += ['--module', str(lib / (name + '.dll'))]
        runtime = bundle / manifest['runtime']
        common = [str(runtime), 'run', str(native / 'app.dll'), *context]
        commands = {'aot': [str(native / ('app.exe' if os.name == 'nt' else 'app'))],
                    'interpreter': [*common, '--execution', 'interpreter', '--instructions', '100000000'],
                    'jit': [*common, '--jit-host', str(bundle / manifest['jit']['host']), '--jit-compiler', str(a.aot.resolve())]}
        for path in [Path(__file__), source, bundle / 'manifest.json', native / 'build.json', runtime, a.aot.resolve()]:
            report['inputs'][str(path.resolve())] = benchmark.sha(path)
        for mode, command in commands.items():
            guest = benchmark.workbench.Guest(command)
            try:
                port, _ = guest.line()
                for row in rows:
                    with socket.create_connection(('127.0.0.1', int(port)), timeout=120) as peer:
                        request = row['request']
                        if row['fragmented']:
                            for offset in range(0, len(request), 3): peer.sendall(request[offset:offset + 3])
                        else: peer.sendall(request)
                        data = b''
                        while True:
                            block = peer.recv(4096)
                            if not block: break
                            data += block
                            if len(data) > 16384: raise ValueError('Oversized test response')
                    line, _ = guest.line()
                    if row['error']:
                        if data or line != ('ERROR:' + row['error']).encode(): raise ValueError((mode, row['name'], data, line))
                    else:
                        head, body = data.split(b'\r\n\r\n', 1)
                        if not head.startswith(b'HTTP/1.1 200 OK\r\n') or body != row['expected'] or line != b'OK':
                            raise ValueError((mode, row['name'], data, line))
                    report['cases'].append(dict(mode=mode, name=row['name'], passed=True, responseHex=data.hex(), outcome=line.decode()))
                guest.line(b'DONE'); guest.finish()
            finally: guest.close()
            print(mode, 'passed', len(rows), 'cases', flush=True)
        for path, digest in report['inputs'].items():
            if benchmark.sha(Path(path)) != digest: raise ValueError('Input drift: ' + path)
        report['passed'] = True
    finally: (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__': main()

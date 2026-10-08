#!/usr/bin/env python3
"""Run one Raven HTTP artifact interpreted and AOT with matched loopback requests.
Correctness/cleanup evidence only: process-per-request timings are not benchmarks.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import select
import socket
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
p = argparse.ArgumentParser(description=__doc__)
for key in ('compiler', 'runtime', 'aot', 'bundle', 'output'):
    p.add_argument('--' + key, type=Path, required=True)
a = p.parse_args()
compiler, runtime, aot, bundle, output = (getattr(a, k).resolve() for k in ('compiler', 'runtime', 'aot', 'bundle', 'output'))
output.mkdir(parents=True, exist_ok=False)
base = ROOT / 'docs/experiments/aot-console'
library = bundle / 'lib' if (bundle / 'lib').is_dir() else bundle
seed, core, ownership = (library / n for n in ('System.runtime.neox', 'Core.dll', 'ownership.json'))
libs = [library / n for n in ('System.Runtime.dll', 'System.Web.dll', 'System.Networking.dll', 'System.Data.dll')]
context = ['--system', seed, *[x for lib in libs for x in ('--module', lib)], '--object-root', libs[0]]
flags = [*context, '--compile-system', '--bind-user-fault', '--reference-arena', '--native-gc',
         '--native-stack-budget', '--bind-int32-to-string', '--bind-utf8-text', '--bind-task-queue',
         '--bind-socket-listener', '--bind-socket-accept', '--bind-socket-transfer',
         '--bind-console-write-line', '--bind-integer-text']
source = ROOT / 'docs/experiments/http-server/Server.rvn'
adapters = [Path(__file__).with_name('http-host.c'),
            *[base / n for n in ('root-probe.c', 'native-gc.c', 'native-stack.c', 'task-queue.c',
                                'socket-listener.c', 'text-arena.c', 'console.c')],
            base.parent / 'aot-scalar/console.c', base.parent / 'aot-fault-details/render.c']
report = {'scope': 'One-request HTTP correctness, not throughput or release qualification',
          'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
          'SDKROOT': os.environ.get('SDKROOT'), 'managedHeapCapacity': 1048576,
          'inputs': {}, 'commands': [], 'cases': {}}

def save():
    (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def run(command):
    result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True, timeout=300)
    report['commands'].append({'command': result.args, 'exit': result.returncode,
                              'stdout': result.stdout, 'stderr': result.stderr})
    save()
    if result.returncode:
        raise RuntimeError(f'{result.args}: {result.stderr}')
    return result

for f in [Path(__file__), source, compiler, runtime, aot, core, seed, ownership, *libs, *adapters,
          *base.glob('*.h'), *base.parent.joinpath('aot-fault-details').glob('*.h'),
          *ROOT.joinpath('tools/aot-poc/src').glob('*.rs'), *compiler.parent.glob('*.dll')]:
    report['inputs'][str(f)] = digest(f)
assembly, obj = output / 'Server.dll', output / 'Server.o'
run(['dotnet', compiler, 'neoclr', '--core-reference', core, '--runtime-seed', seed,
     *[x for lib in libs for x in ('--reference', lib)], '--bootstrap-intrinsics', '--bootstrap-ownership',
     ownership, '--object-library', 'System.Runtime', '-o', assembly, source])
run([aot, '--closed-world', assembly, '@entry', obj, *flags])
report['artifact'] = {'assemblySha256': digest(assembly), 'objectSha256': digest(obj)}
commands = {'interpreter': [runtime, 'run', assembly, *context, '--instructions', '100000000']}
for name, sanitizer in [('native-sanitized', ['-fsanitize=undefined,bounds']), ('native-standalone', [])]:
    binary = output / name
    run(['clang', '-arch', 'arm64', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror',
         *sanitizer, '-DNEOCLR_NATIVE_GC', '-I', base, *adapters, obj, '-o', binary])
    commands[name] = [binary]
    if not sanitizer:
        dependencies = run(['otool', '-L', binary]).stdout.splitlines()[1:]
        if [line.split()[0] for line in dependencies] != ['/usr/lib/libSystem.B.dylib']:
            raise RuntimeError(dependencies)
        report['standalone'] = {'bytes': binary.stat().st_size, 'sha256': digest(binary), 'dependencies': dependencies}

valid = b'GET /greeting HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n'
cases = [('greeting', valid, False, None), ('fragmented', valid, True, None),
         ('duplicate-length', b'GET /greeting HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\nContent-Length: 0\r\n\r\n', False, 'Duplicate request Content-Length'),
         ('handler-error', valid.replace(b'/greeting', b'/other'), False, 'Unexpected request')]
expected_response = (b'HTTP/1.1 200 OK\r\nContent-Length: 10\r\nConnection: close\r\n'
                     b'Content-Type: text/plain; charset=utf-8\r\n\r\n' + 'Café 🌍'.encode())

def serve(command, request, fragmented):
    process = subprocess.Popen(list(map(str, command)), cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    response = b''
    try:
        if not select.select([process.stdout], [], [], 30)[0]:
            raise RuntimeError('Server did not report its port')
        line = process.stdout.readline()
        if not line.strip().isdigit():
            raise RuntimeError(f'Expected port, received {line!r}')
        with socket.create_connection(('127.0.0.1', int(line)), timeout=10) as peer:
            if fragmented:
                for at in range(0, len(request), 3):
                    peer.sendall(request[at:at + 3])
                    time.sleep(0.002)
            else:
                peer.sendall(request)
            peer.shutdown(socket.SHUT_WR)
            try:
                while True:
                    part = peer.recv(4096)
                    if not part:
                        break
                    response += part
                    if len(response) > 8192:
                        raise RuntimeError('Unexpected response size')
            except ConnectionResetError:
                pass
        stdout, stderr = process.communicate(timeout=20)
        return {'exit': process.returncode, 'stdout': stdout.decode(), 'stderr': stderr.decode(),
                'responseHex': response.hex()}
    finally:
        if process.poll() is None:
            process.kill()
            process.communicate()

for case, request, fragmented, error in cases:
    results = report['cases'][case] = {}
    for mode, command in commands.items():
        result = results[mode] = serve(command, request, fragmented)
        save()
        expected_output = 'Other work runs while HTTP accept is pending\n'
        expected_output += 'Served greeting; server closed\n' if error is None else 'Server error: ' + error + '\n'
        assert result['exit'] == 0 and result['stderr'] == '', result
        assert result['stdout'] == expected_output, result
        assert bytes.fromhex(result['responseHex']) == (expected_response if error is None else b''), result
        assert result == results['interpreter'], (case, mode, result, results['interpreter'])
        print(case, mode, 'passed', flush=True)
if digest(aot) != report['inputs'][str(aot)]:
    raise RuntimeError('AOT executable changed during verification')
report['passed'] = True
save()
print('All matched one-request HTTP cases passed; no throughput claim.', flush=True)

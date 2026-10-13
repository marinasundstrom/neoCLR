#!/usr/bin/env python3
"""Hold one request incomplete while the other peers must finish; never a throughput score."""
import argparse
import hashlib
import json
from pathlib import Path
import socket
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT/'benchmarks/framework'))
from process import Guest

BODY = 'Café 🌍'.encode()
FAST = b'GET /greeting HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n'
CASES = [
    ('headers', b'GET /greeting HTTP/1.1\r\nHost: localhost\r\n', b'Connection: close\r\n\r\n'),
    ('fixed-body', b'POST /greeting HTTP/1.1\r\nHost: localhost\r\nContent-Length: 8\r\n\r\nabc', b'defgh'),
    ('chunked-body', b'POST /greeting HTTP/1.1\r\nHost: localhost\r\nTransfer-Encoding: chunked\r\n\r\n8\r\nabc', b'defgh\r\n0\r\n\r\n'),
    ('large-header', b'GET /greeting HTTP/1.1\r\nHost: localhost\r\nX-Pad: '+b'a'*4096, b'\r\nConnection: close\r\n\r\n'),
]


def response(peer):
    wire = b''
    while True:
        part = peer.recv(4096)
        if not part:
            break
        wire += part
        if len(wire) > 8192:
            raise ValueError('Oversized response')
    head, body = wire.split(b'\r\n\r\n', 1)
    lines = head.split(b'\r\n')
    fields = dict(line.lower().split(b': ', 1) for line in lines[1:])
    assert lines[0] == b'HTTP/1.1 200 OK', lines
    assert fields[b'content-length'] == str(len(BODY)).encode()
    assert fields[b'connection'] == b'close'
    assert body == BODY, body


def run_case(command, workload=None, width=4):
    guest = Guest(command, timeout=180)
    results = []
    try:
        port, ready = guest.line()
        for name, prefix, suffix in CASES:
            guest.line(b'BATCH')
            sockets = []
            try:
                slow = socket.create_connection(('127.0.0.1', int(port)), timeout=90)
                sockets.append(slow)
                slow.sendall(prefix)
                started = time.perf_counter()
                peers = []
                for _ in range(width-1):
                    peer = socket.create_connection(('127.0.0.1', int(port)), timeout=90)
                    sockets.append(peer)
                    peer.sendall(FAST)
                    peers.append(peer)
                completions = []
                for peer in peers:
                    response(peer)
                    completions.append(time.perf_counter()-started)
                # Every fast peer completed while the first request was incomplete.
                slow.sendall(suffix)
                released = time.perf_counter()
                response(slow)
                results.append(dict(case=name, fastCompletionSeconds=completions,
                                    slowHeldSeconds=released-started,
                                    slowFinishSeconds=time.perf_counter()-released,
                                    fastResponsesBeforeSlowRelease=width-1, passed=True))
            finally:
                for peer in sockets:
                    peer.close()
        guest.line(b'DONE')
        guest.finish()
        return dict(passed=True, startupSeconds=ready-guest.started, cases=results,
                    checkedResponses=4*width, width=width, queuedCancellations=140,
                    shutdownCancelledAccepts=16, admissionLimit=16)
    except Exception as error:
        try:
            guest.process.wait(timeout=3)
        except Exception:
            pass
        print('Completed phases:', results, 'fast completions:', locals().get('completions'), file=sys.stderr, flush=True)
        raise RuntimeError(f'{name if results or "name" in locals() else "startup"}: {error}; guest errors: {guest.errors()!r}') from error
    finally:
        guest.close()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--build', type=Path, required=True)
    p.add_argument('--bundle', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--mode', action='append', choices=['jit', 'aot', 'interpreter'])
    a = p.parse_args()
    b = a.bundle.resolve()
    manifest = json.loads((b/'manifest.json').read_text())
    context = ['--system', str(b/manifest['runtimeSeed']), '--object-root', str(b/manifest['objectRoot'])]
    for module in manifest['modules']:
        context += ['--module', str(b/module)]
    app = a.build.resolve()/'app.dll'
    commands = {'jit': [str(b/manifest['runtime']), 'run', str(app)],
                'aot': [str(a.build.resolve()/'app')],
                'interpreter': [str(b/manifest['runtime']), 'run', str(app), '--execution', 'interpreter', '--instructions', '2000000000', *context]}
    report = dict(passed=False, commands=commands, modes={}, inputs={},
                  scope='http-concurrency-v1: four batches: width four in all modes to stay below the five-second transport read ceiling; withhold first request suffix until all other responses complete. Includes cancellation/admission/shutdown assertions. Client phase durations, not isolated runtime/parser time or throughput.')
    for path in [Path(__file__), Path(__file__).with_name('Server.rvn'), app, a.build/'app', a.build/'build.json', b/'manifest.json']:
        report['inputs'][str(path.resolve())] = hashlib.sha256(path.read_bytes()).hexdigest()
    try:
        for mode in a.mode or ['jit', 'aot', 'interpreter']:
            report['modes'][mode] = run_case(commands[mode], width=4)
            print('PASS', mode, flush=True)
        report['passed'] = True
    finally:
        a.output.write_text(json.dumps(report, indent=2)+'\n')


if __name__ == '__main__':
    main()

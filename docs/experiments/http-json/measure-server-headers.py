"""Measure sample header-size cost and its recovery after an over-limit request on macOS."""
import argparse
import json, subprocess, socket, time, selectors
from pathlib import Path
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bundle', required=True, type=Path)
parser.add_argument('--output', required=True, type=Path)
args = parser.parse_args()
root = args.bundle.resolve()
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
p = subprocess.Popen([str(root / 'tools/http-runner'), str(root / 'editable-samples/http-json/server/bin/neoclr/Debug/App.neoil'), str(root / 'lib/System.neoil'), '1024', '100000000', '--live-output', '--', '4'], stdout=subprocess.PIPE, stderr=(out / 'headers.stderr').open('w'), text=True)

def cpu():
    s = subprocess.check_output(['ps', '-p', str(p.pid), '-o', 'time='], text=True).strip()
    parts = s.split(':')
    return sum((float(value) * 60 ** index for (index, value) in enumerate(reversed(parts))))
try:
    with selectors.DefaultSelector() as sel:
        sel.register(p.stdout, selectors.EVENT_READ)
        assert sel.select(30)
    port = int(p.stdout.readline())
    results = []
    for length in (128, 1024, 2049, 128):
        prefix = b'GET /report HTTP/1.1\r\nHost: localhost\r\nX-Padding: '
        suffix = b'\r\n\r\n'
        request = prefix + b'x' * (length - len(prefix) - len(suffix)) + suffix
        start = cpu()
        t = time.monotonic()
        response = b''
        with socket.create_connection(('127.0.0.1', port), timeout=30) as s:
            s.sendall(request)
            try:
                while (part := s.recv(4096)):
                    response += part
            except ConnectionResetError:
                pass
        assert response.startswith(b'HTTP/1.1 200 OK') if length <= 2048 else response == b''
        entry = {'header_bytes': len(request), 'wall_seconds': time.monotonic() - t, 'response': response.decode()}
        if p.poll() is None:
            entry['cpu_seconds'] = cpu() - start
        results.append(entry)
    p.wait(timeout=20)
    assert p.returncode == 0, 'Server must continue after rejecting the oversized request'
    assert 'live=0' in (out / 'headers.stderr').read_text()
    data = {'requests': results, 'exit_code': p.returncode}
    (out / 'headers.json').write_text(json.dumps(data, indent=2) + '\n')
    print(json.dumps(data, indent=2))
finally:
    if p.poll() is None:
        p.terminate()
        p.wait(timeout=10)

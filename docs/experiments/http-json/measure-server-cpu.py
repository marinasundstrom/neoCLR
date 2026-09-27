"""Measure a locally installed sample server; macOS ps/sample, no compiler rebuild."""
import argparse
import json, subprocess, time, selectors
from pathlib import Path
from urllib.request import urlopen, Request
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bundle', required=True, type=Path)
parser.add_argument('--output', required=True, type=Path)
parser.add_argument('--idle-seconds', type=float, default=10)
args = parser.parse_args()
if args.idle_seconds <= 0:
    parser.error('--idle-seconds must be positive')
root = args.bundle.resolve()
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
cmd = [str(root / 'tools/http-runner'), str(root / 'editable-samples/http-json/server/bin/neoclr/Debug/App.neoil'), str(root / 'lib/System.neoil'), '1024', '100000000', '--live-output', '--', '7']

def cpu(pid):
    s = subprocess.check_output(['ps', '-p', str(pid), '-o', 'time='], text=True).strip()
    parts = s.split(':')
    return sum((float(value) * 60 ** index for (index, value) in enumerate(reversed(parts))))

def phase(p, seconds):
    c = cpu(p.pid)
    t = time.monotonic()
    time.sleep(seconds)
    elapsed = time.monotonic() - t
    return {'wall_seconds': elapsed, 'cpu_seconds': cpu(p.pid) - c}
p = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=(out / 'server.stderr').open('w'), text=True)
try:
    t = time.monotonic()
    with selectors.DefaultSelector() as sel:
        sel.register(p.stdout, selectors.EVENT_READ)
        assert sel.select(30), 'no port'
    port = int(p.stdout.readline())
    data = {'pid': p.pid, 'port': port, 'startup_wall_seconds': time.monotonic() - t, 'startup_cpu_seconds': cpu(p.pid)}
    subprocess.run(['sample', str(p.pid), '3', '1', '-file', str(out / 'idle.sample.txt')], capture_output=True)
    data['idle_before'] = phase(p, args.idle_seconds)
    reqs = []
    sampler = subprocess.Popen(['sample', str(p.pid), '5', '1', '-file', str(out / 'requests.sample.txt')], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    for i in range(6):
        method = 'GET' if i % 2 == 0 else 'POST'
        path = '/report' if method == 'GET' else '/reports'
        body = None if method == 'GET' else b'{"station":"Cafe"}'
        c = cpu(p.pid)
        t = time.monotonic()
        with urlopen(Request(f'http://127.0.0.1:{port}' + path, data=body, headers={'Content-Type': 'application/json'}), timeout=30) as r:
            payload = r.read().decode()
            status = r.status
        assert status == (200 if method == 'GET' else 201)
        assert json.loads(payload) == ({'station': 'Café'} if method == 'GET' else {'accepted': True})
        reqs.append({'method': method, 'status': status, 'body': payload, 'wall_seconds': time.monotonic() - t, 'cpu_seconds': cpu(p.pid) - c})
    sampler.wait()
    data['requests'] = reqs
    data['idle_after'] = phase(p, args.idle_seconds)
    with urlopen(f'http://127.0.0.1:{port}/report', timeout=30) as r:
        r.read()
    p.wait(timeout=30)
    data['exit_code'] = p.returncode
    assert p.returncode == 0
    (out / 'measurements.json').write_text(json.dumps(data, indent=2) + '\n')
    print(json.dumps(data, indent=2))
finally:
    if p.poll() is None:
        p.terminate()
        p.wait(timeout=10)

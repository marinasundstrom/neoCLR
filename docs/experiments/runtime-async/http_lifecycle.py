#!/usr/bin/env python3
"""Check runtime-await HTTP terminal outcomes, socket closure and recovery."""
import argparse
import hashlib
import json
from pathlib import Path
import socket
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
source = Path(__file__).with_name('RuntimeHttpLifecycle.rvn').resolve()
provider = Path(__file__).resolve().parents[3] / 'runtime/compiler-services/RuntimeAwait.rvn'
assembly = output / 'lifecycle.dll'
assembly.unlink(missing_ok=True)
base = recipe['commands'][0]['command']
compile_command = base[:base.index('-o')] + ['-o', str(assembly), str(source), str(provider)]
run_base = recipe['commands'][1]['command'].copy()
run_base[2] = str(assembly)
inputs = [source, provider, Path(__file__).resolve(), Path(base[1]), Path(run_base[0])]
inputs += list(Path(base[1]).parent.glob('*.dll'))
report = {'passed': False, 'recipeSha256': hashlib.sha256(args.recipe.read_bytes()).hexdigest(),
          'inputsSha256': {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, 'cases': []}


def request(connection):
    connection.settimeout(30)
    data = b''
    while b'\r\n\r\n' not in data:
        chunk = connection.recv(4096)
        if not chunk or len(data) > 8192:
            raise RuntimeError('Incomplete/oversized request')
        data += chunk
    if not data.startswith(b'GET /greeting HTTP/1.1\r\n'):
        raise RuntimeError('Unexpected request')


def peer(listener, mode, observations):
    try:
        with listener.accept()[0] as connection:
            request(connection)
            observations['acceptedRequest'] = True
            if mode == 'protocol':
                connection.sendall(b'not-an-http-response\r\n\r\n')
            elif mode == 'fault':
                connection.sendall(b'HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok')
            observations['firstSocketClosed'] = connection.recv(1) == b''
        if mode != 'fault':
            with listener.accept()[0] as connection:
                request(connection)
                observations['recoveryRequest'] = True
                connection.sendall(b'HTTP/1.1 200 OK\r\nContent-Length: 9\r\nConnection: close\r\n\r\nrecovered')
                observations['recoverySocketClosed'] = connection.recv(1) == b''
    except Exception as error:
        observations['error'] = str(error)


try:
    result = subprocess.run(compile_command, capture_output=True, text=True, timeout=120)
    report['compile'] = {'command': compile_command, 'exitCode': result.returncode, 'stderr': result.stderr}
    if result.returncode:
        raise RuntimeError(result.stderr)
    report['assemblySha256'] = hashlib.sha256(assembly.read_bytes()).hexdigest()
    listing = output / 'lifecycle.neoil'
    listing.unlink(missing_ok=True)
    subprocess.run([run_base[0], 'disassemble', str(assembly), str(listing)], check=True, capture_output=True, timeout=120)
    report['stateMachineMentions'] = listing.read_text().count('AsyncStateMachine')
    if report['stateMachineMentions']:
        raise RuntimeError('Unexpected state machine')
    for mode in ['timeout', 'cancel', 'protocol', 'fault']:
        observed = {}
        with socket.socket() as listener:
            listener.bind(('127.0.0.1', 0))
            listener.listen(2)
            listener.settimeout(30)
            url = f'http://127.0.0.1:{listener.getsockname()[1]}/greeting'
            worker = threading.Thread(target=peer, args=(listener, mode, observed), daemon=True)
            worker.start()
            command = run_base + ['--', url, mode, url]
            result = subprocess.run(command, capture_output=True, text=True, timeout=60)
            worker.join(timeout=35)
        passed = observed.get('acceptedRequest') and observed.get('firstSocketClosed') and not observed.get('error') and not worker.is_alive()
        if mode == 'fault':
            passed = passed and result.returncode != 0 and 'UserFault: runtime HTTP continuation fault' in result.stderr and not result.stdout
        else:
            passed = (passed and result.returncode == 0 and not result.stderr
                      and result.stdout.strip() == 'PASS runtime HTTP lifecycle and recovery'
                      and observed.get('recoveryRequest') and observed.get('recoverySocketClosed'))
        report['cases'].append({'mode': mode, 'passed': bool(passed), 'command': command,
            'exitCode': result.returncode, 'stdout': result.stdout, 'stderr': result.stderr, 'peer': observed})
        if not passed:
            raise RuntimeError('Failed lifecycle mode: ' + mode)
    report['passed'] = True
except Exception as error:
    report['error'] = str(error)
finally:
    (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report, indent=2))
raise SystemExit(0 if report['passed'] else 1)

"""Check native-only System namespace completion.

Usage: python3 scripts/check-native-editor.py SERVER_DLL PROJECT_DIR OUTPUT_JSON
Run NeoClrMetadataProbe --native-core-project CORE LIBRARY PROJECT_DIR first.
This checks semantic editor requests, not VS Code build/run or reference refresh.
"""
import hashlib
import json
from pathlib import Path
import queue
import subprocess
import sys
import threading
import time

server, project, output = map(lambda p: Path(p).resolve(), sys.argv[1:])
messages = queue.Queue()
output.parent.mkdir(parents=True, exist_ok=True)
log = output.with_suffix('.stderr.log').open('wb')
process = subprocess.Popen(['dotnet', str(server)], stdin=subprocess.PIPE,
    stdout=subprocess.PIPE, stderr=log, cwd=project)
def read():
    try:
        while True:
            headers = {}
            while (line := process.stdout.readline()) not in (b'\r\n', b'\n'):
                if not line:
                    raise EOFError('Language server ended its output stream')
                key, value = line.decode().split(':', 1)
                headers[key.lower()] = value.strip()
            messages.put(json.loads(process.stdout.read(int(headers['content-length']))))
    except Exception as error:
        messages.put(error)

threading.Thread(target=read, daemon=True).start()
sequence = 0
transcript = []
def send(method, params, request=False):
    global sequence
    message = {'jsonrpc': '2.0', 'method': method, 'params': params}
    if request:
        sequence += 1
        message['id'] = sequence
    data = json.dumps(message).encode()
    process.stdin.write(f'Content-Length: {len(data)}\r\n\r\n'.encode()+data)
    process.stdin.flush()
    transcript.append(message)
    return sequence

def receive(identifier):
    import time
    deadline = time.monotonic()+45
    while True:
        item = messages.get(timeout=max(0.01, deadline-time.monotonic()))
        if isinstance(item, Exception):
            raise item
        transcript.append(item)
        if item.get('id') == identifier and ('result' in item or 'error' in item):
            if 'error' in item:
                raise AssertionError(item['error'])
            return item['result']
        if time.monotonic() >= deadline:
            raise TimeoutError('Language-server response timed out')

try:
    receive(send('initialize', {'processId': None, 'rootUri': project.as_uri(), 'capabilities': {},
        'workspaceFolders': [{'uri': project.as_uri(), 'name': 'native core'}]}, True))
    send('initialized', {})
    uri = (project / 'Main.rvn').as_uri()
    source = 'func Main() -> int { System. }'
    send('textDocument/didOpen', {'textDocument': {'uri': uri, 'version': 1, 'languageId': 'raven', 'text': source}})
    result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
        'position': {'line': 0, 'character': source.index('System.') + len('System.')},
        'context': {'triggerKind': 1}}, True))
    items = result if isinstance(result, list) else result['items']
    labels = [item['label'] for item in items]
    assert 'Int32' in labels, labels
    assert not any(label.startswith(('Action', 'Func<')) for label in labels), labels
    receive(send('shutdown', None, True))
    send('exit', None)
    process.wait(timeout=10)
    assert process.returncode == 0
    output.write_text(json.dumps({'passed': True, 'labels': labels,
        'serverSha256': hashlib.sha256(server.read_bytes()).hexdigest(),
        'project': str(project)}, indent=2) + '\n')
    print('PASS native-only System completion without CLI delegate transport types')
finally:
    if process.poll() is None:
        process.kill()
        process.wait(timeout=10)
    log.close()

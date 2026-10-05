"""Check native-project LSP symbols using the fixture from --native-project.

Usage: python3 scripts/check-native-editor.py SERVER_DLL PROJECT_DIR OUTPUT_JSON
Run NeoClrMetadataProbe --native-project CORE SEED ROOT first; PROJECT_DIR is ROOT/project.
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
        'workspaceFolders': [{'uri': project.as_uri(), 'name': 'native project'}]}, True))
    send('initialized', {})
    uri = (project / 'Main.rvn').as_uri()
    text = (project / 'Main.rvn').read_text()
    send('textDocument/didOpen', {'textDocument': {'uri': uri, 'version': 1, 'languageId': 'raven', 'text': text}})
    hover = receive(send('textDocument/hover', {'textDocument': {'uri': uri},
        'position': {'line': 0, 'character': text.index('Updated') + 2}}, True))
    assert hover and 'Updated' in json.dumps(hover), hover
    partial = 'func Main() -> int => Library.Api.'
    send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': 2}, 'contentChanges': [{'text': partial}]})
    result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
        'position': {'line': 0, 'character': len(partial)}, 'context': {'triggerKind': 1}}, True))
    items = result if isinstance(result, list) else result['items']
    labels = [item['label'] for item in items]
    assert any(label.startswith('Updated') for label in labels), labels
    assert not any(label.startswith('Value') for label in labels), labels
    send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': 3},
        'contentChanges': [{'text': text.replace('Updated', 'Missing')}]})
    deadline = time.monotonic() + 45
    while True:
        item = messages.get(timeout=max(.01, deadline - time.monotonic()))
        if isinstance(item, Exception): raise item
        transcript.append(item)
        if item.get('method') == 'textDocument/publishDiagnostics':
            diagnostics = item['params']['diagnostics']
            if any('Missing' in d['message'] and d.get('severity') == 1 for d in diagnostics): break
        if time.monotonic() >= deadline: raise TimeoutError('No missing-member semantic diagnostic')
    send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': 4}, 'contentChanges': [{'text': text}]})
    receive(send('shutdown', None, True))
    send('exit', None)
    process.wait(timeout=10)
    assert process.returncode == 0, process.returncode
    artifacts = [server, project / 'App.rvnproj', project / 'Main.rvn', project.parent / 'dependencies/Library.dll']
    output.write_text(json.dumps({'passed': True, 'checks': ['native hover', 'native completion', 'missing-member diagnostics'],
        'artifacts': {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in artifacts},
        'transcript': transcript}, indent=2) + '\n')
    print('PASS native-project LSP hover, completion and semantic diagnostics')
finally:
    if process.poll() is None:
        process.kill()
        process.wait(timeout=10)
    log.close()

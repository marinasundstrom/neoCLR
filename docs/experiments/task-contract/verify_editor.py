"""Check the installed Task API through Raven Language Server stdio."""
import json
from pathlib import Path
import queue
import subprocess
import sys
import threading

project = Path(sys.argv[1]).resolve()
server = json.loads((project / '.vscode/settings.json').read_text())['raven.languageServerPath']
messages = queue.Queue()
log = (project / 'lsp-stderr.log').open('wb')
process = subprocess.Popen(['dotnet', server], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=log, cwd=project)

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
        'workspaceFolders': [{'uri': project.as_uri(), 'name': 'Task demo'}]}, True))
    send('initialized', {})
    uri = (project / 'Main.rvn').as_uri()
    results = {}
    probes = [
        ('Task', 'import System.*\nimport System.Tasks.*\nfunc Inspect(task: Task<Result<int, string>>) {\n    task.\n}', '    task.', ['State', 'Outcome', 'Map', 'Then', 'MapResult']),
        ('Promise', 'import System.*\nimport System.Tasks.*\nfunc Inspect(promise: Promise<int>) {\n    promise.\n}', '    promise.', ['Task', 'Complete', 'Cancel']),
        ('Queue', 'import System.Tasks.*\nfunc Inspect() {\n    TaskQueue.\n}', '    TaskQueue.', ['Default']),
    ]
    for index, (label, text, access, expected) in enumerate(probes):
        lines = text.splitlines()
        position = {'line': next(i for i,l in enumerate(lines) if l == access), 'character':len(access)}
        if index == 0:
            send('textDocument/didOpen', {'textDocument': {'uri':uri,'languageId':'raven','version':1,'text':text}})
        else:
            send('textDocument/didChange', {'textDocument': {'uri':uri,'version':index+1},'contentChanges':[{'text':text}]})
        result = receive(send('textDocument/completion', {'textDocument': {'uri':uri}, 'position':position, 'context':{'triggerKind':1}}, True))
        items = result if isinstance(result,list) else result['items']
        labels = sorted({i['label'] for i in items})
        for name in expected:
            assert any(l == name or l.startswith(name+'(') or l.startswith(name+'<') for l in labels), (label, name, labels)
        if label == 'Queue':
            default = next(i for i in items if i['label'] == 'Default')
            assert default['kind'] == 10, default
            assert 'Current' not in labels, labels
        results[label] = labels
    receive(send('shutdown', None, True))
    send('exit', None)
    print(json.dumps(results,indent=2))
finally:
    (project/'lsp-transcript.json').write_text(json.dumps(transcript,indent=2)+'\n')
    if process.poll() is None: process.terminate()
    process.wait(timeout=10)
    log.close()

"""Exercise the same stdio LSP server used by VS Code, with bounded request waits."""
import json
from pathlib import Path
import queue
import subprocess
import sys
import threading

project = Path(sys.argv[1]).resolve()
collections = '--collections' in sys.argv[2:]
files = '--files' in sys.argv[2:]
strings = '--strings' in sys.argv[2:]
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
        'workspaceFolders': [{'uri': project.as_uri(), 'name': 'neoCLR demo'}]}, True))
    send('initialized', {})
    uri = (project / 'Main.rvn').as_uri()
    results = {}
    for version, owner in enumerate(('Math', 'Console', ''), 1):
        access = 'System.' + (owner + '.' if owner else '')
        text = f'import System.*\nfunc Main() {{\n    {access}\n}}'
        document = {'uri': uri, 'version': version}
        if version == 1:
            send('textDocument/didOpen', {'textDocument': {**document, 'languageId': 'raven', 'text': text}})
        else:
            send('textDocument/didChange', {'textDocument': document, 'contentChanges': [{'text': text}]})
        result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
            'position': {'line': 2, 'character': len('    '+access)},
            'context': {'triggerKind': 1}}, True))
        items = result if isinstance(result, list) else result['items']
        labels = sorted({item['label'] for item in items})
        expected = ('Abs', 'Min', 'Max', 'Sign') if owner == 'Math' else (('WriteLine',) if owner else (('Collections', 'Option', 'Result', 'Console', 'Math') if collections else ('Option', 'Result', 'Console', 'Math')))
        if any(not any(label == name or label.startswith(name+'(') for label in labels) for name in expected):
            raise AssertionError(f'{owner}: missing target completions: {labels}')
        if any(label == name or label.startswith(name+'(') for label in labels for name in ('ReadLine', 'Clamp', 'Sin', 'Sqrt')):
            raise AssertionError(f'{owner}: unexpected host API: {labels}')
        if not owner and 'IO' in labels and not files:
            raise AssertionError('Host System.IO leaked into target namespace')
        results[owner or 'System'] = labels
    if collections:
        text = 'import System.Collections.*\nfunc Inspect(values: List<int>) {\n    values.\n}'
        send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': 4}, 'contentChanges': [{'text': text}]})
        result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
            'position': {'line': 2, 'character': len('    values.')}, 'context': {'triggerKind': 1}}, True))
        items = result if isinstance(result, list) else result['items']
        labels = sorted({item['label'] for item in items})
        for expected in ('Add', 'Count', 'GetIterator'):
            if not any(label == expected or label.startswith(expected + '(') for label in labels):
                raise AssertionError('Missing collection completion: ' + str(labels))
        results['List'] = labels
        text = 'import System.Collections.*\nfunc Inspect(values: List<int>) {\n    for item in values {\n        System.Console.WriteLine(item)\n    }\n}'
        send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': 5}, 'contentChanges': [{'text': text}]})
        hover = receive(send('textDocument/hover', {'textDocument': {'uri': uri},
            'position': {'line': 2, 'character': 9}}, True))
        if hover is None or not any(name in json.dumps(hover) for name in ('int', 'Int32')):
            raise AssertionError('Loop element type was not inferred from the project target: ' + str(hover))
        results['LoopElementHover'] = hover
        text = 'import System.*\nfunc Read(value: Option<int>) -> Option<int> {\n    let amount = value?\n    return Option<int>(Option.Some<int>(amount))\n}'
        send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': 6}, 'contentChanges': [{'text': text}]})
        hover = receive(send('textDocument/hover', {'textDocument': {'uri': uri},
            'position': {'line': 2, 'character': 9}}, True))
        if hover is None or not any(name in json.dumps(hover) for name in ('int', 'Int32')):
            raise AssertionError('Option propagation output type was not inferred: ' + str(hover))
        results['PropagationOutputHover'] = hover
        text = 'import System.Collections.*\nfunc Main() {\n    let values = ArrayList<int>(2)\n    values.\n}'
        send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': 7}, 'contentChanges': [{'text': text}]})
        result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
            'position': {'line': 3, 'character': 11}, 'context': {'triggerKind': 1}}, True))
        items = result if isinstance(result, list) else result['items']
        labels = sorted({item['label'] for item in items})
        if 'Capacity' not in labels or any(label.startswith('Allocate') for label in labels):
            raise AssertionError('Unexpected ArrayList constructor surface: ' + str(labels))
        results['ArrayList'] = labels
    if files:
        for version, owner, expected, forbidden in (
            (8, 'IO', ('File', 'FileReadError', 'FileWriteError'), ('Directory', 'Stream')),
            (9, 'IO.File', ('ReadAllText', 'WriteAllText'), ('Delete', 'ReadAllBytes', 'Open'))):
            access = 'System.' + owner + '.'
            text = f'import System.*\nfunc Main() {{\n    {access}\n}}'
            send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': version}, 'contentChanges': [{'text': text}]})
            result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
                'position': {'line': 2, 'character': len('    ' + access)}, 'context': {'triggerKind': 1}}, True))
            items = result if isinstance(result, list) else result['items']
            labels = sorted({item['label'] for item in items})
            if any(not any(label == name or label.startswith(name + '(') for label in labels) for name in expected):
                raise AssertionError('Missing target file API: ' + str(labels))
            if any(label == name or label.startswith(name + '(') for label in labels for name in forbidden):
                raise AssertionError('Host file API leaked: ' + str(labels))
            results[owner] = labels
    if strings:
        for version, access, prefix, expected, forbidden in (
            (10, 'text.', '    let text = "hello"\n',
             ('Equals', 'ContainsOrdinal', 'StartsWithOrdinal', 'EndsWithOrdinal', 'GetUtf8ByteCount', 'IsEmpty'), ('Substring', 'Contains')),
            (11, 'System.String.', '', ('Concat', 'CompareOrdinal'), ('IsNullOrEmpty', 'Join', 'Format'))):
            text = 'import System.*\nfunc Main() {\n' + prefix + '    ' + access + '\n}'
            send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': version}, 'contentChanges': [{'text': text}]})
            result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
                'position': {'line': 3 if prefix else 2, 'character': len('    ' + access)}, 'context': {'triggerKind': 1}}, True))
            items = result if isinstance(result, list) else result['items']
            labels = sorted({item['label'] for item in items})
            if any(not any(label == name or label.startswith(name + '(') for label in labels) for name in expected):
                raise AssertionError('Missing target String API: ' + str(labels))
            if any(label == name or label.startswith(name + '(') for label in labels for name in forbidden):
                raise AssertionError('Host String API leaked: ' + str(labels))
            results[access] = labels
    receive(send('shutdown', None, True))
    send('exit', None)
    print(json.dumps(results, indent=2))
finally:
    (project / 'lsp-transcript.json').write_text(json.dumps(transcript, indent=2)+'\n')
    if process.poll() is None:
        process.terminate()
    process.wait(timeout=10)
    log.close()

"""Exercise the same stdio LSP server used by VS Code, with bounded request waits."""
import json
from pathlib import Path
import queue
import subprocess
import sys
import threading

project = Path(sys.argv[1]).resolve()
collections = '--collections' in sys.argv[2:]
booleans = '--booleans' in sys.argv[2:]
reflection = '--reflection' in sys.argv[2:]
process_apis = '--process' in sys.argv[2:]
unions = '--unions' in sys.argv[2:]
errors = '--errors' in sys.argv[2:]
calendar = '--calendar' in sys.argv[2:]
primitives = '--primitives' in sys.argv[2:]
parsing = '--parsing' in sys.argv[2:]
files = '--files' in sys.argv[2:]
strings = '--strings' in sys.argv[2:]
patterns = '--patterns' in sys.argv[2:]
extensions = '--extensions' in sys.argv[2:]
queries = '--queries' in sys.argv[2:]
array_invariance = '--array-invariance' in sys.argv[2:]
array_shape = '--array-shape' in sys.argv[2:]
collection_capabilities = '--collection-capabilities' in sys.argv[2:]
maps = '--maps' in sys.argv[2:]
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
        expected = ('Abs', 'Min', 'Max', 'Sign', 'Clamp', 'Sqrt', 'Floor', 'Ceiling', 'Truncate', 'Round', 'Exp', 'Log', 'Log10', 'Sin', 'Cos', 'Tan', 'Pow') if owner == 'Math' else (('WriteLine',) if owner else (('Collections', 'Option', 'Result', 'Console', 'Math') if collections else ('Option', 'Result', 'Console', 'Math')))
        if any(not any(label == name or label.startswith(name+'(') for label in labels) for name in expected):
            raise AssertionError(f'{owner}: missing target completions: {labels}')
        if any(label == name or label.startswith(name+'(') for label in labels for name in ('ReadLine', 'Atan', 'Cosh')):
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
        if 'Capacity' not in labels or any(name not in labels for name in ('Copy', 'Find', 'FindLast', 'FindIndex', 'FindLastIndex', 'FindAll', 'Exists', 'TrueForAll')) or any(label.startswith('Allocate') for label in labels):
            raise AssertionError('Unexpected ArrayList constructor surface: ' + str(labels))
        results['ArrayList'] = labels
    if files:
        for version, owner, expected, forbidden in (
            (8, 'IO', ('Path', 'File', 'FileReadError', 'FileWriteError'), ('Directory', 'Stream')),
            (9, 'IO.File', ('ReadAllText', 'WriteAllText'), ('Delete', 'ReadAllBytes', 'Open')),
            (10, 'IO.Path', ('Combine', 'GetFileName'), ('GetFullPath', 'GetExtension'))):
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
            (11, 'text.', '    let text = "hello"\n',
             ('Equals', 'ContainsOrdinal', 'StartsWithOrdinal', 'EndsWithOrdinal', 'GetUtf8ByteCount', 'IsEmpty', 'SliceUtf8'), ('Substring', 'Contains')),
            (12, 'System.String.', '', ('Concat', 'CompareOrdinal'), ('IsNullOrEmpty', 'Join', 'Format'))):
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
    if parsing:
        access = 'System.Int32.'
        text = 'func Main() {\n    ' + access + '\n}'
        send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': 13}, 'contentChanges': [{'text': text}]})
        result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
            'position': {'line': 1, 'character': len('    ' + access)}, 'context': {'triggerKind': 1}}, True))
        items = result if isinstance(result, list) else result['items']
        labels = sorted({item['label'] for item in items})
        if any(not any(label == name or label.startswith(name + '(') for label in labels) for name in ('Parse', 'Divide')):
            raise AssertionError('Missing target Parse API: ' + str(labels))
        if any(label == 'TryParse' or label.startswith('TryParse(') for label in labels):
            raise AssertionError('Host TryParse leaked: ' + str(labels))
        results['Int32'] = labels
    if parsing:
        text = 'func Main() {\n    let number = 42\n    number.\n}'
        send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': 14}, 'contentChanges': [{'text': text}]})
        result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
            'position': {'line': 2, 'character': 11}, 'context': {'triggerKind': 1}}, True))
        items = result if isinstance(result, list) else result['items']
        labels = sorted({item['label'] for item in items})
        if any(not any(label == name or label.startswith(name + '(') for label in labels) for name in ('Equals', 'CompareTo', 'ToString')):
            raise AssertionError('Missing Int32 instance API: ' + str(labels))
        results['Int32Instance'] = labels
    if primitives:
        text = 'func Main() {\n    System.Char.\n}'
        send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': 15}, 'contentChanges': [{'text': text}]})
        result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
            'position': {'line': 1, 'character': 16}, 'context': {'triggerKind': 1}}, True))
        items = result if isinstance(result, list) else result['items']
        labels = sorted({item['label'] for item in items})
        expected = ('IsDigit', 'IsNumber', 'IsLetter', 'IsUpper', 'IsLower', 'IsSeparator', 'IsControl', 'IsPunctuation', 'IsSymbol', 'IsSurrogate', 'IsHighSurrogate', 'IsLowSurrogate', 'IsAscii', 'IsAsciiDigit', 'IsLetterOrDigit', 'IsWhiteSpace')
        if any(not any(label == name or label.startswith(name + '(') for label in labels) for name in expected):
            raise AssertionError('Missing character API: ' + str(labels))
        results['Char'] = labels
    if calendar:
        for version, expression, expected in (
            (16, 'System.Date.', ('Create', 'FromDayNumber')),
            (17, 'System.Time.', ('Create', 'FromTicks')),
            (18, 'System.Clock.', ('GetLocalNow',)),
            (19, 'now.', ('Date', 'Time', 'UtcOffsetSeconds'))):
            text = 'func Main() {\n    let now = System.Clock.GetLocalNow()\n    ' + expression + '\n}'
            send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': version}, 'contentChanges': [{'text': text}]})
            result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
                'position': {'line': 2, 'character': len('    ' + expression)}, 'context': {'triggerKind': 1}}, True))
            items = result if isinstance(result, list) else result['items']
            labels = sorted({item['label'] for item in items})
            if any(not any(label == name or label.startswith(name + '(') for label in labels) for name in expected):
                raise AssertionError('Missing calendar API: ' + str(labels))
            results[expression] = labels
    if errors:
        for version, expression, expected in (
            (20, 'System.IO.FileReadError.', ('NotFound', 'AccessDenied')),
            (21, 'error.', ('IsNotFound', 'GetNotFound', 'ToString')),
            (22, 'System.Error.', ('FromMessage',))):
            text = 'func Main() {\n    let error = System.IO.FileReadError(System.IO.FileReadError.NotFound())\n    ' + expression + '\n}'
            send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': version}, 'contentChanges': [{'text': text}]})
            result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
                'position': {'line': 2, 'character': len('    ' + expression)}, 'context': {'triggerKind': 1}}, True))
            items = result if isinstance(result, list) else result['items']
            labels = sorted({item['label'] for item in items})
            if any(not any(label == name or label.startswith(name + '(') for label in labels) for name in expected):
                raise AssertionError('Missing error API: ' + str(labels))
            results[expression] = labels
    if unions:
        for version, expression, expected in (
            (23, 'result.', ('IsOk', 'IsErr', 'GetOkCase', 'GetErrorCase', 'TryGet', 'TryGetOutput', 'TryGetResidual')),
            (24, 'option.', ('IsSome', 'IsNone', 'GetSomeCase', 'GetNoneCase', 'TryGet'))):
            text = 'import System.*\nfunc Main() {\n    let result = Result<long, Error>.Ok(42L)\n    let option = Option<string>(Option.None())\n    ' + expression + '\n}'
            send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': version}, 'contentChanges': [{'text': text}]})
            result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
                'position': {'line': 4, 'character': len('    ' + expression)}, 'context': {'triggerKind': 1}}, True))
            items = result if isinstance(result, list) else result['items']
            labels = sorted({item['label'] for item in items})
            if any(not any(label == name or label.startswith(name + '(') for label in labels) for name in expected):
                raise AssertionError('Missing union API: ' + str(labels))
            results[expression] = labels
    if process_apis:
        for version, expression, expected in (
            (25, 'System.Environment.', ('GetCommandLineArgs', 'GetCurrentDirectory', 'GetEnvironmentVariable')),
            (26, 'System.Console.', ('WriteLine', 'ReadByte'))):
            text = 'func Main() {\n    ' + expression + '\n}'
            send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': version}, 'contentChanges': [{'text': text}]})
            result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
                'position': {'line': 1, 'character': len('    ' + expression)}, 'context': {'triggerKind': 1}}, True))
            items = result if isinstance(result, list) else result['items']
            labels = sorted({item['label'] for item in items})
            if any(not any(label == name or label.startswith(name + '(') for label in labels) for name in expected):
                raise AssertionError('Missing process API: ' + str(labels))
            results[expression] = labels
    if booleans:
        text = 'func Main() {\n    let value = true\n    value.\n}'
        send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': 27}, 'contentChanges': [{'text': text}]})
        result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
            'position': {'line': 2, 'character': 10}, 'context': {'triggerKind': 1}}, True))
        items = result if isinstance(result, list) else result['items']
        labels = sorted({item['label'] for item in items})
        if not any(label == 'CompareTo' or label.startswith('CompareTo(') for label in labels):
            raise AssertionError('Missing Boolean CompareTo: ' + str(labels))
        results['Boolean'] = labels
    if reflection:
        for version, expression, expected in (
            (28, 'info.', ('Name', 'FullName', 'GetFields', 'GetMethods', 'GetProperties', 'GetElementType', 'GetGenericArguments')),
            (29, 'method.', ('Name', 'DeclaringType', 'ReturnType', 'GetParameters', 'IsPublic')),
            (30, 'property.', ('Name', 'CanRead', 'GetGetMethod', 'GetIndexParameters')),
            (31, 'System.Reflection.BindingFlags.', ('Public', 'NonPublic', 'Instance', 'Static', 'DeclaredOnly')),
            (32, 'System.Runtime.InteropServices.NativeMemory.', ('Alloc', 'Free'))):
            text = 'import System.*\nfunc Main() {\n    let info = typeof(int)\n    let method = info.GetMethods()[0]\n    let property = typeof(Date).GetProperties()[0]\n    ' + expression + '\n}'
            send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': version}, 'contentChanges': [{'text': text}]})
            result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
                'position': {'line': 5, 'character': len('    ' + expression)}, 'context': {'triggerKind': 1}}, True))
            items = result if isinstance(result, list) else result['items']
            labels = sorted({item['label'] for item in items})
            if any(not any(label == name or label.startswith(name + '(') for label in labels) for name in expected):
                raise AssertionError('Missing reflection API: ' + str(labels))
            results[expression] = labels
    if collections:
        for version, contract, expected in ((33, 'Comparable<int>', 'CompareTo'), (34, 'Equatable<Type>', 'Equals'), (35, 'Clonable<int>', 'Clone'), (36, 'Closable<OverflowError>', 'Close')):
            text = 'import System.*\nfunc Check(value: ' + contract + ') {\n    value.\n}'
            send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': version}, 'contentChanges': [{'text': text}]})
            result = receive(send('textDocument/completion', {'textDocument': {'uri': uri}, 'position': {'line': 2, 'character': 10}, 'context': {'triggerKind': 1}}, True))
            items = result if isinstance(result, list) else result['items']
            labels = sorted({item['label'] for item in items})
            if not any(label == expected or label.startswith(expected + '(') for label in labels):
                raise AssertionError('Missing interface member: ' + str(labels))
            results[contract] = labels
    if parsing:
        for version, declaration in ((37, 'init()'), (38, 'init(value: int)')):
            text = 'class Example {\n    ' + declaration + ' {\n        System.Int32.\n    }\n}'
            send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': version}, 'contentChanges': [{'text': text}]})
            result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
                'position': {'line': 2, 'character': len('        System.Int32.')},
                'context': {'triggerKind': 2, 'triggerCharacter': '.'}}, True))
            items = result if isinstance(result, list) else result['items']
            labels = sorted({item['label'] for item in items})
            assert 'Parse' in labels and 'Divide' in labels, labels
            assert 'CompareTo' not in labels, labels
            results['Int32 in ' + declaration] = labels
    if patterns:
        for version, case_head in ((39, 'Ok(let text)'), (40, '.Ok(let text)')):
            text = ('import System.*\nimport System.Result.*\n'
                    'func Inspect(value: Result<string, OverflowError>) {\n'
                    '    match value {\n        ' + case_head + ' => {\n'
                    '            text.\n        }\n        .Error(_) => {}\n    }\n}')
            send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': version}, 'contentChanges': [{'text': text}]})
            result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
                'position': {'line': 5, 'character': len('            text.')},
                'context': {'triggerKind': 2, 'triggerCharacter': '.'}}, True))
            items = result if isinstance(result, list) else result['items']
            labels = sorted({item['label'] for item in items})
            assert {'GetUtf8ByteCount', 'IsEmpty', 'ContainsOrdinal'}.issubset(labels), (case_head, labels)
            results[case_head] = labels
            hover = receive(send('textDocument/hover', {'textDocument': {'uri': uri},
                'position': {'line': 5, 'character': 14}}, True))
            assert hover is not None and any(name in json.dumps(hover) for name in ('string', 'String')), hover
            results[case_head + ' payload hover'] = hover
    if extensions:
        text = ('extension NumberOperations for int {\n'
                '    func Next() -> int { return self + 1 }\n}\n'
                'func Main() {\n    let value = 41\n    value.\n}')
        send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': 41}, 'contentChanges': [{'text': text}]})
        result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
            'position': {'line': 5, 'character': len('    value.')},
            'context': {'triggerKind': 2, 'triggerCharacter': '.'}}, True))
        items = result if isinstance(result, list) else result['items']
        labels = sorted({item['label'] for item in items})
        assert any(label == 'Next' or label.startswith('Next(') for label in labels), labels
        results['Extension receiver'] = labels
    if queries:
        for version, declaration in ((42, 'let values = ArrayList<int>()'),
                                     (43, 'let values = ArrayList<int>().Where((value: int) -> bool => true)'),
                                     (44, 'let values: int[] = [1, 2]'),
                                     (45, 'let values = typeof(int).GetMethods()')):
            text = ('import System.Collections.*\nimport System.Linq.*\n'
                    'func Main() {\n    ' + declaration + '\n    values.\n}')
            send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': version}, 'contentChanges': [{'text': text}]})
            result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
                'position': {'line': 4, 'character': len('    values.')},
                'context': {'triggerKind': 2, 'triggerCharacter': '.'}}, True))
            items = result if isinstance(result, list) else result['items']
            labels = sorted({item['label'] for item in items})
            assert {'Where', 'Select', 'ToList', 'First', 'Last', 'Single'}.issubset(labels), labels
            results['Query extensions ' + str(version)] = labels
    if array_invariance:
        import time
        for version, expression, expected_code in (
                (46, 'let members: MemberInfo[] = typeof(int).GetMethods()', 'RAV1504'),
                (47, 'let members = (MemberInfo[])typeof(int).GetMethods()', 'RAV1503'),
                (48, 'let members: MethodInfo[] = typeof(int).GetMethods()', None)):
            text = 'import System.*\nimport System.Reflection.*\nfunc Main() {\n    ' + expression + '\n}\n'
            send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': version},
                'contentChanges': [{'text': text}]})
            deadline = time.monotonic() + 90
            while True:
                item = messages.get(timeout=max(0.01, deadline - time.monotonic()))
                if isinstance(item, Exception):
                    raise item
                transcript.append(item)
                params = item.get('params', {})
                if item.get('method') == 'textDocument/publishDiagnostics' and params.get('uri') == uri and params.get('version') == version:
                    errors = [d for d in params['diagnostics'] if d.get('severity') == 1]
                    # Syntax-only updates may clear or carry earlier diagnostics while
                    # semantic analysis runs. Wait for this edit's expected outcome.
                    if expected_code is None:
                        if errors:
                            continue
                    elif not any(d.get('code') == expected_code and
                                 'MemberInfo' in d['message'] and 'MethodInfo' in d['message']
                                 for d in errors):
                        continue
                    results['Array invariance diagnostics ' + str(version)] = params['diagnostics']
                    break
                if time.monotonic() >= deadline:
                    raise TimeoutError('Array invariance diagnostics timed out')
    if array_shape:
        text = ('import System.*\nimport System.Linq.*\n'
                'func Inspect(values: Array<int>) {\n    values.\n}')
        send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': 49},
            'contentChanges': [{'text': text}]})
        result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
            'position': {'line': 3, 'character': len('    values.')},
            'context': {'triggerKind': 2, 'triggerCharacter': '.'}}, True))
        items = result if isinstance(result, list) else result['items']
        labels = sorted({item['label'] for item in items})
        assert {'Length', 'GetIterator', 'Where', 'ToList'}.issubset(labels), labels
        results['Generic array members'] = labels
        text = ('import System.*\nfunc Inspect(values: Array<int>) -> int {\n'
                '    let vector: int[] = values\n    return vector[0]\n}')
        send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': 50},
            'contentChanges': [{'text': text}]})
        hover = receive(send('textDocument/hover', {'textDocument': {'uri': uri},
            'position': {'line': 2, 'character': 9}}, True))
        assert hover is not None and any(t in json.dumps(hover) for t in ('int[]', 'Int32[]')), hover
        results['Generic array alias hover'] = hover
    if collection_capabilities:
        for version, shape, can_add in ((51, 'Sequence<int>', False),
                                        (52, 'MutableSequence<int>', False),
                                        (53, 'List<int>', True),
                                        (54, 'Array<int>', False)):
            text = ('import System.*\nimport System.Collections.*\nimport System.Linq.*\n'
                    'func Inspect(values: ' + shape + ') {\n    values.\n}')
            send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': version},
                'contentChanges': [{'text': text}]})
            result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
                'position': {'line': 4, 'character': len('    values.')},
                'context': {'triggerKind': 2, 'triggerCharacter': '.'}}, True))
            items = result if isinstance(result, list) else result['items']
            labels = sorted({item['label'] for item in items})
            assert {'Count', 'GetIterator', 'ToList'}.issubset(labels), labels
            assert ('Add' in labels) == can_add, labels
            results['Collection capabilities ' + shape] = labels
    if maps:
        for version, shape, mutable in ((55, 'Map<int, string>', False),
                                        (56, 'MutableMap<int, string>', True),
                                        (57, 'HashMap<int, string>', True)):
            text = ('import System.*\nimport System.Collections.*\n'
                    'func Inspect(values: ' + shape + ') {\n    values.\n}')
            send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': version},
                'contentChanges': [{'text': text}]})
            result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
                'position': {'line': 3, 'character': len('    values.')},
                'context': {'triggerKind': 2, 'triggerCharacter': '.'}}, True))
            items = result if isinstance(result, list) else result['items']
            labels = sorted({item['label'] for item in items})
            assert {'Count', 'Keys', 'Find', 'ContainsKey'}.issubset(labels), labels
            assert ('TryAdd' in labels) == mutable and ('Set' in labels) == mutable, labels
            results['Map capabilities ' + shape] = labels
        for version, label, body in (
            (58, 'Order map payload', 'match index.Find(101) {\nSome(let order) => order.\nNone => ()\n}'),
            (59, 'Order filtered element', 'let order = values.FindAll((order: Order) -> bool => true)[0]\norder.')):
            text = ('import System.*\nimport System.Collections.*\nimport System.Option.*\n'
                    'class Order { var Number: int }\n'
                    'func Inspect(index: Map<int, Order>, values: ArrayList<Order>) {\n' + body + '\n}')
            lines = text.splitlines()
            line = next(i for i, value in enumerate(lines) if value.endswith('order.'))
            send('textDocument/didChange', {'textDocument': {'uri': uri, 'version': version},
                'contentChanges': [{'text': text}]})
            result = receive(send('textDocument/completion', {'textDocument': {'uri': uri},
                'position': {'line': line, 'character': len(lines[line])},
                'context': {'triggerKind': 2, 'triggerCharacter': '.'}}, True))
            items = result if isinstance(result, list) else result['items']
            labels = sorted({item['label'] for item in items})
            assert 'Number' in labels, labels
            results[label] = labels
    receive(send('shutdown', None, True))
    send('exit', None)
    print(json.dumps(results, indent=2))
finally:
    (project / 'lsp-transcript.json').write_text(json.dumps(transcript, indent=2)+'\n')
    if process.poll() is None:
        process.terminate()
    process.wait(timeout=10)
    log.close()

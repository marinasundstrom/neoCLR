"""Build bounded Raven-authored runtime implementations and its bootstrap snapshots."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]
SLICES = {
    'JsonError': 'System.Data.Json.JsonError',
    'JsonValue': 'System.Data.Json.JsonValue',
    "Cancellation": "System.Concurrency.CancellationTokenSource",
    "IPAddress": "System.Networking.IPAddress",
    "IPAddressError": "System.Networking.IPAddressError",
    "HttpError": "System.Web.Http.HttpError",
    "Uri": "System.Uri",
    "UriError": "System.UriError",
    "HttpClient": "System.Web.Http.HttpClient",
    "DnsError": "System.Networking.DnsError",
    "Dns": "System.Networking.Dns",
    "SocketError": "System.Networking.Sockets.SocketError",
    "Socket": "System.Networking.Sockets.Socket",
    'TextReadError': 'System.IO.TextReadError',
    'TextReader': 'System.IO.TextReader',
    'TextWriter': 'System.IO.TextWriter',
    'StreamWriter': 'System.IO.StreamWriter',
    'MemoryStream': 'System.IO.MemoryStream',
    'StreamReader': 'System.IO.StreamReader',
    'SeekableStream': 'System.IO.SeekableStream',

    'FileSystem': 'System.Storage.FileSystem',
    'StorageItems': 'System.Storage.StorageItem',
    'StorageProvider': 'System.Storage.StorageProvider',
    'InputStream': 'System.IO.InputStream',
    'OutputStream': 'System.IO.OutputStream',
    "InvalidPathError": "System.Storage.InvalidPathError",
    "EntryKind": "System.Storage.EntryKind",
    "HttpStatusCode": "System.Web.Http.HttpStatusCode",
    "StorageLookupError": "System.Storage.StorageLookupError",
    "StorageMetadata": "System.Storage.Metadata",
    'StreamError': 'System.IO.StreamError',
    'FileInputStream': 'System.IO.FileInputStream',
    'FileOutputStream': 'System.IO.FileOutputStream',

    'TaskOutcome': 'System.Tasks.TaskOutcome',
    'TaskState': 'System.Tasks.TaskState',
    'Tasks': 'System.Tasks.Task',
    'Workers': 'System.Concurrency.Thread',

    'Array': 'System.Array',
    'Object': 'System.Object',
    'HashCode': 'System.HashCode',
    'UnionAttribute': 'System.Runtime.CompilerServices.UnionAttribute',
    'BindingFlags': 'System.Introspection.BindingFlags',
    'Func': 'System.Func',
    'Fault': 'System',
    'NativeMemory': 'System.Runtime.InteropServices.NativeMemory',
    'Descriptors': 'System.Introspection.MemberInfo',
    'Option': 'System.Option',
    'Result': 'System.Result',
    'Propagatable': 'System.Propagatable',
    'IntegerDivisionError': 'System.IntegerDivisionError',
    'SingleError': 'System.Linq.SingleError',
    'Int32ParseError': 'System.Int32ParseError',
    'Utf8SliceError': 'System.Text.Utf8SliceError',
    'ConsoleReadError': 'System.ConsoleReadError',
    'FileWriteError': 'System.Storage.FileWriteError',
    'FileReadError': 'System.Storage.FileReadError',
    'InvalidRangeError': 'System.InvalidRangeError',
    'InvalidDateError': 'System.InvalidDateError',
    'InvalidTimeError': 'System.InvalidTimeError',
    'OverflowError': 'System.OverflowError',
    'EnvironmentError': 'System.EnvironmentError',
    'Void': 'System.Void',

    'String': 'System.String',
    'Utf8': 'System.Text.Utf8',
    'UnicodeScalar': 'System.Text.UnicodeScalar',
    'InvalidUtf8Error': 'System.Text.InvalidUtf8Error',
    'Environment': 'System.Environment',
    'Console': 'System.Console',
    'IntPtr': 'System.IntPtr',
    'UIntPtr': 'System.UIntPtr',
    'Iterable': 'System.Collections.Iterable',
    'Iterator': 'System.Collections.Iterator',
    'Collection': 'System.Collections.Collection',
    'Sequence': 'System.Collections.Sequence',
    'MutableSequence': 'System.Collections.MutableSequence',
    'List': 'System.Collections.List',
    'Map': 'System.Collections.Map',
    'MutableMap': 'System.Collections.MutableMap',

    'SystemClock': 'System.SystemClock',
    'LocalDateTime': 'System.LocalDateTime',
    'Disposable': 'System.Disposable',
    'Equatable': 'System.Equatable',
    'Comparable': 'System.Comparable',
    'Clonable': 'System.Clonable',
    'Closable': 'System.Closable',

    'Clock': 'System.Clock',
    'AssemblyInfo': 'System.Introspection.AssemblyInfo',
    'ModuleInfo': 'System.Introspection.ModuleInfo',

    'ParameterInfo': 'System.Introspection.ParameterInfo',
    'Duration': 'System.Duration',
    'Instant': 'System.Instant',
    'RuntimeContext': 'System.Runtime.RuntimeContext',
    'RuntimeTypeHandle': "System.RuntimeTypeHandle",
    'Value': "System.Value",
    'Math': 'System.Math',
    'Linq': 'System.Linq.Operators',
    'OptionOperators': 'System.OptionOperators',
    'OptionNestedOperators': 'System.OptionNestedOperators',
    'ResultOperators': 'System.ResultOperators',
    'Int32': 'System.Int32',
    'Char': 'System.Char',
    'ArrayList': 'System.Collections.ArrayList',
    'HashMap': 'System.Collections.HashMap',
    'Time': 'System.Time',
    'Date': 'System.Date',
    'Path': 'System.Storage.Path',
    'FileText': 'System.Storage.FileText',
    'Int64': 'System.Int64',
    'SByte': 'System.SByte',
    'Byte': 'System.Byte',
    'Int16': 'System.Int16',
    'UInt16': 'System.UInt16',
    'UInt32': 'System.UInt32',
    'UInt64': 'System.UInt64',
    'Single': 'System.Single',
    'Double': 'System.Double',
    'Boolean': 'System.Boolean',
}
SOURCES = {
    'JsonValue': 'runtime/raven/src/System/Data/Json/JsonValue.rvn',
    'JsonError': 'runtime/raven/src/System/Data/Json/JsonError.rvn',
    'JsonDocument': 'runtime/raven/src/System/Data/Json/JsonDocument.rvn',
    'JsonSyntax': 'runtime/raven/src/System/Data/Json/JsonSyntax.rvn',
    'JsonSerializer': 'runtime/raven/src/System/Data/Json/JsonSerializer.rvn',

    "Cancellation": "runtime/raven/src/System/Concurrency/Cancellation.rvn",
    "IPAddress": "runtime/raven/src/System/Networking/IPAddress.rvn",
    "IPAddressError": "runtime/raven/src/System/Networking/IPAddressError.rvn",
    "Uri": "runtime/raven/src/System/Uri.rvn",
    "UriError": "runtime/raven/src/System/UriError.rvn",
    "HttpServer": "runtime/raven/src/System/Web/Http/HttpServer.rvn",
    "HttpClient": "runtime/raven/src/System/Web/Http/HttpClient.rvn",
    "HttpError": "runtime/raven/src/System/Web/Http/HttpError.rvn",
    "DnsError": "runtime/raven/src/System/Networking/DnsError.rvn",
    "Dns": "runtime/raven/src/System/Networking/Dns.rvn",
    "SocketError": "runtime/raven/src/System/Networking/Sockets/SocketError.rvn",
    "Socket": "runtime/raven/src/System/Networking/Sockets/Socket.rvn",
    'TextReadError': 'runtime/raven/src/System/IO/TextReadError.rvn',
    'TextReader': 'runtime/raven/src/System/IO/TextReader.rvn',
    'TextWriter': 'runtime/raven/src/System/IO/TextWriter.rvn',
    'StreamWriter': 'runtime/raven/src/System/IO/StreamWriter.rvn',
    'MemoryStream': 'runtime/raven/src/System/IO/MemoryStream.rvn',
    'StreamReader': 'runtime/raven/src/System/IO/StreamReader.rvn',
    'SeekableStream': 'runtime/raven/src/System/IO/SeekableStream.rvn',
    'FileSystem': 'runtime/raven/src/System/Storage/FileSystem.rvn',
    'StorageItems': 'runtime/raven/src/System/Storage/StorageItems.rvn',
    'StorageProvider': 'runtime/raven/src/System/Storage/StorageProvider.rvn',
    'InputStream': 'runtime/raven/src/System/IO/InputStream.rvn',
    'OutputStream': 'runtime/raven/src/System/IO/OutputStream.rvn',
    "InvalidPathError": "runtime/raven/src/System/Storage/InvalidPathError.rvn",
    "EntryKind": "runtime/raven/src/System/Storage/EntryKind.rvn",
    "HttpStatusCode": "runtime/raven/src/System/Web/Http/HttpStatusCode.rvn",
    "StorageLookupError": "runtime/raven/src/System/Storage/StorageLookupError.rvn",
    "StorageMetadata": "runtime/raven/src/System/Storage/Metadata.rvn",
    'StreamError': 'runtime/raven/src/System/IO/StreamError.rvn',
    'FileInputStream': 'runtime/raven/src/System/IO/FileInputStream.rvn',
    'FileOutputStream': 'runtime/raven/src/System/IO/FileOutputStream.rvn',

    'TaskOutcome': 'runtime/raven/src/System/Tasks/TaskOutcome.rvn',
    'TaskState': 'runtime/raven/src/System/Tasks/TaskState.rvn',
    'Tasks': 'runtime/raven/src/System/Tasks/Tasks.rvn',
    'Workers': 'runtime/raven/src/System/Concurrency/Workers.rvn',

    'Array': 'runtime/raven/src/System/Array.rvn',
    'Object': 'runtime/raven/src/System/Object.rvn',
    'HashCode': 'runtime/raven/src/System/HashCode.rvn',
    'UnionAttribute': 'runtime/raven/src/System/Runtime/CompilerServices/UnionAttribute.rvn',
    'BindingFlags': 'runtime/raven/src/System/Introspection/BindingFlags.rvn',
    'Func': 'runtime/raven/src/System/Func.rvn',
    'Fault': 'runtime/raven/src/System/Functions.rvn',
    'NativeMemory': 'runtime/raven/src/System/Runtime/InteropServices/NativeMemory/Functions.rvn',
    'Descriptors': 'runtime/raven/src/System/Introspection/Descriptors.rvn',
    'Option': 'runtime/raven/src/System/Option.rvn',
    'Result': 'runtime/raven/src/System/Result.rvn',
    'Propagatable': 'runtime/raven/src/System/Propagatable.rvn',
    'IntegerDivisionError': 'runtime/raven/src/System/IntegerDivisionError.rvn',
    'SingleError': 'runtime/raven/src/System/Linq/SingleError.rvn',
    'Int32ParseError': 'runtime/raven/src/System/Int32ParseError.rvn',
    'Utf8SliceError': 'runtime/raven/src/System/Text/Utf8SliceError.rvn',
    'ConsoleReadError': 'runtime/raven/src/System/ConsoleReadError.rvn',
    'FileWriteError': 'runtime/raven/src/System/Storage/FileWriteError.rvn',
    'FileReadError': 'runtime/raven/src/System/Storage/FileReadError.rvn',
    'InvalidRangeError': 'runtime/raven/src/System/InvalidRangeError.rvn',
    'InvalidDateError': 'runtime/raven/src/System/InvalidDateError.rvn',
    'InvalidTimeError': 'runtime/raven/src/System/InvalidTimeError.rvn',
    'OverflowError': 'runtime/raven/src/System/OverflowError.rvn',
    'EnvironmentError': 'runtime/raven/src/System/EnvironmentError.rvn',
    'Void': 'runtime/raven/src/System/Void.rvn',

    'String': 'runtime/raven/src/System/String.rvn',
    'Utf8': 'runtime/raven/src/System/Text/Utf8.rvn',
    'UnicodeScalar': 'runtime/raven/src/System/Text/UnicodeScalar.rvn',
    'InvalidUtf8Error': 'runtime/raven/src/System/Text/InvalidUtf8Error.rvn',
    'Environment': 'runtime/raven/src/System/Environment/Functions.rvn',
    'Console': 'runtime/raven/src/System/Console/Functions.rvn',
    'IntPtr': 'runtime/raven/src/System/IntPtr.rvn',
    'UIntPtr': 'runtime/raven/src/System/UIntPtr.rvn',
    'Iterable': 'runtime/raven/src/System/Collections/Iterable.rvn',
    'Iterator': 'runtime/raven/src/System/Collections/Iterator.rvn',
    'Collection': 'runtime/raven/src/System/Collections/Collection.rvn',
    'Sequence': 'runtime/raven/src/System/Collections/Sequence.rvn',
    'MutableSequence': 'runtime/raven/src/System/Collections/MutableSequence.rvn',
    'List': 'runtime/raven/src/System/Collections/List.rvn',
    'Map': 'runtime/raven/src/System/Collections/Map.rvn',
    'MutableMap': 'runtime/raven/src/System/Collections/MutableMap.rvn',

    'SystemClock': 'runtime/raven/src/System/SystemClock.rvn',
    'LocalDateTime': 'runtime/raven/src/System/LocalDateTime.rvn',
    'Disposable': 'runtime/raven/src/System/Disposable.rvn',
    'Equatable': 'runtime/raven/src/System/Equatable.rvn',
    'Comparable': 'runtime/raven/src/System/Comparable.rvn',
    'Clonable': 'runtime/raven/src/System/Clonable.rvn',
    'Closable': 'runtime/raven/src/System/Closable.rvn',

    'Clock': 'runtime/raven/src/System/Clock.rvn',
    'AssemblyInfo': 'runtime/raven/src/System/Introspection/AssemblyInfo.rvn',
    'ModuleInfo': 'runtime/raven/src/System/Introspection/ModuleInfo.rvn',

    'ParameterInfo': 'runtime/raven/src/System/Introspection/ParameterInfo.rvn',
    'Duration': 'runtime/raven/src/System/Duration.rvn',
    'Instant': 'runtime/raven/src/System/Instant.rvn',
    'RuntimeContext': 'runtime/raven/src/System/Runtime/RuntimeContext.rvn',
    'RuntimeTypeHandle': "runtime/raven/src/System/RuntimeTypeHandle.rvn",
    'Value': "runtime/raven/src/System/Value.rvn",
    'Math': 'runtime/raven/src/System/Math/Functions.rvn',
    'Int32': 'runtime/raven/src/System/Int32.rvn',
    'Char': 'runtime/raven/src/System/Char.rvn',
    'Linq': 'runtime/raven/src/System/Linq/Operators.rvn',
    'OptionOperators': 'runtime/raven/src/System/OptionOperators.rvn',
    'OptionNestedOperators': 'runtime/raven/src/System/OptionOperators.rvn',
    'ResultOperators': 'runtime/raven/src/System/ResultOperators.rvn',
    'ArrayList': 'runtime/raven/src/System/Collections/ArrayList.rvn',
    'HashMap': 'runtime/raven/src/System/Collections/HashMap.rvn',
    'Time': 'runtime/raven/src/System/Time.rvn',
    'Date': 'runtime/raven/src/System/Date.rvn',
    'Path': 'runtime/raven/src/System/Storage/Path.rvn',
    'FileText': 'runtime/raven/src/System/Storage/FileText.rvn',
    'Int64': 'runtime/raven/src/System/Int64.rvn',
    'SByte': 'runtime/raven/src/System/SByte.rvn',
    'Byte': 'runtime/raven/src/System/Byte.rvn',
    'Int16': 'runtime/raven/src/System/Int16.rvn',
    'UInt16': 'runtime/raven/src/System/UInt16.rvn',
    'UInt32': 'runtime/raven/src/System/UInt32.rvn',
    'UInt64': 'runtime/raven/src/System/UInt64.rvn',
    'Single': 'runtime/raven/src/System/Single.rvn',
    'Double': 'runtime/raven/src/System/Double.rvn',
    'Boolean': 'runtime/raven/src/System/Boolean.rvn',
}
PROJECT = ROOT / 'runtime/raven/System.Runtime.rvnproj'
GENERATED = ROOT / 'runtime/raven/generated'

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def fragments(text, name="Math", owner="System.Math", bootstrap=False):
    if name == 'Path' and bootstrap:
        # The archived Neo profile has no Object or HashCode class contract.
        # Keep its existing lexical surface; Raven uses the complete implementation.
        text = text.replace('.extends System.Object\n', '').replace('.implements System.Equatable<System.Storage.Path>\n', '')
        text = text.replace('call instance System.Object::.ctor()', 'pop')
        text = re.sub(r'(?ms)^\.method instance override (?:Equals\(System.Object other\)|GetHashCode\(\))[^\n]*\n.*?^\.end\n', '', text)
        text = text.replace('.method instance override ToString()', '.method instance ToString()')
    lines = text.splitlines(keepends=True)
    methods, helpers, types = [], {}, []
    task_operators = {}
    while lines:
        if not lines[0].strip():
            lines.pop(0)
            continue
        if lines[0].startswith(('.type ', '.interface ', '.delegate ')):
            depth = 0
            for index, line in enumerate(lines):
                token = line.strip().split(' ', 1)[0]
                if token in ('.type', '.interface', '.delegate', '.method', '.property'):
                    depth += 1
                elif token == '.end':
                    depth -= 1
                    if depth == 0:
                        break
            else:
                raise ValueError('Unclosed private implementation type')
            body = ''.join(lines[:index + 1])
            if lines[0].startswith(('.delegate ' + owner + '<', '.type class ' + owner + '<', '.type ' + owner + '<', '.interface ' + owner + '<')) or lines[0].strip() in ('.type ' + owner, '.interface ' + owner):
                methods.append(body)
            else:
                types.append(body)
            lines = lines[index + 1:]
            continue
        match = re.match(r'\.function (?:internal )?([^(]+)\(', lines[0])
        assert match, lines[0]
        end = lines.index('.end\n')
        body = ''.join(lines[:end + 1])
        if name == 'Tasks' and match[1].startswith(('System.Tasks.TaskOperators.', 'System.Tasks.TaskResultOperators.')):
            operator_owner = match[1].rsplit('.', 1)[0]
            task_operators.setdefault(operator_owner, []).append(body.replace('.function ' + operator_owner + '.', '.method static ', 1))
        elif match[1].startswith(owner + '.'):
            # Retain the bootstrap owner used by direct IL and the archived Neo frontend.
            # Namespace functions use marked containers; static APIs retain their owner.
            if owner == "System":
                methods.append(body)
            elif body.startswith('.function internal '):
                methods.append(body.replace('.function internal ' + owner + '.', '.method internal static ', 1))
            else:
                methods.append(body.replace('.function ' + owner + '.', '.method static ', 1))
        else:
            assert match[1] not in helpers
            helpers[match[1]] = body
        lines = lines[end + 1:]
    for operator_owner, bodies in task_operators.items():
        types.append('.type ' + operator_owner + '\n' + ''.join(bodies) + '.end\n')
    # Nongeneric classes can also own static factories. Merge their function roots
    # into the emitted class rather than leaving top-level method fragments.
    for body in types[:]:
        if body.startswith(('.type class ' + owner + '\n', '.type class abstract ' + owner + '\n')):
            types.remove(body)
            methods = [body[:-len('.end\n')] + ''.join(methods) + '.end\n']
    if name == 'String' and bootstrap:
        # The archived Neo profile uses borrowed collection interfaces. Keep its
        # scalar-independent String operations; Raven uses the complete source.
        methods = [re.sub(r'(?ms)^\.method (?:private )?(?:internal static |instance (?:readonly byref )?)(?:GetIterator|GetScalars|CreateFromCharacters|CollectionCount)\(.*?^\.end\n', '', body)
                   for body in methods]
        methods = [re.sub(r'(?m)^\.implements System\.Collections\.(?:Iterable|Collection|Sequence)<Char>\n', '', body)
                   for body in methods]
        methods = [re.sub(r'(?ms)^\.property instance Metadata_[^\n]+\n\.get instance System\.String::CollectionCount\(\)\n\.end\n', '', body)
                   for body in methods]
    # Retain only transitively called adapters; no application entry-point shim.
    used = set()
    pending = re.findall(r'(?m)^(?:call|ldftn) ([^(]+)\(', ''.join(methods + types))
    while pending:
        helper = pending.pop()
        if helper in used or helper not in helpers:
            continue
        used.add(helper)
        pending.extend(re.findall(r'(?m)^(?:call|ldftn) ([^(]+)\(', helpers[helper]))
    banner = f'; Generated from {SOURCES[name]}. Regenerate with build_runtime_library.py.\n'
    prefix = name + ('.bootstrap' if bootstrap else '')
    result = {prefix + '.methods.neoil': banner + ''.join(methods),
              prefix + '.helpers.neoil': banner + ''.join(body for name, body in helpers.items() if name in used) + ''.join(types)}
    if name in ('String', 'Path') and not bootstrap:
        result.update(fragments(text, name, owner, bootstrap=True))
    return result

def check_snapshot():
    for name in SLICES:
        data = json.loads((GENERATED / (name + '.json')).read_text())
        for path, expected in data['inputs'].items():
            if digest(ROOT / path) != expected:
                raise SystemExit('Stale Raven library input: ' + path)
        for path, expected in data['outputs'].items():
            if digest(GENERATED / path) != expected:
                raise SystemExit('Modified generated library: ' + path)
    print('Raven library source and bootstrap snapshot hashes match')

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path)
    parser.add_argument('--bridge', type=Path)
    parser.add_argument('--slice', action='append', choices=tuple(SLICES), help='Regenerate only selected implementation slices; omitted means all')
    parser.add_argument('--check', action='store_true', help='Regenerate and compare without modifying the snapshot')
    parser.add_argument('--check-snapshot', action='store_true', help='Check source/artifact hashes without an SDK')
    args = parser.parse_args()
    if args.check_snapshot:
        check_snapshot()
        return
    if not args.compiler or not args.bridge:
        parser.error('--compiler and --bridge are required for regeneration')
    with tempfile.TemporaryDirectory(prefix='neoclr-runtime-library-') as temporary:
        root = Path(temporary)
        (root / 'demo').mkdir()
        core = root / 'demo/NeoCLR.CoreProbe.dll'
        bridge = ['dotnet', str(args.bridge.resolve())]
        subprocess.run([*bridge, '--reference-library-core', str(core)], check=True)
        subprocess.run(['dotnet', str(args.compiler.resolve()), str(PROJECT), '--no-project-restore',
                        '-o', str(root / 'compiled')], env={**os.environ, 'NeoCLRBootstrapRoot': str(root), 'NeoCLRLibrarySlice': ''}, check=True)
        if args.check:
            check_snapshot()
        generated = {}
        for name, owner in SLICES.items():
            if args.slice and name not in args.slice:
                continue
            compiled = root / 'compiled'
            if name not in ('Math', 'Linq', 'OptionOperators', 'OptionNestedOperators', 'ResultOperators'):
                compiled = root / ('compiled-' + name)
                subprocess.run(['dotnet', str(args.compiler.resolve()), str(PROJECT), '--no-project-restore',
                                '-o', str(compiled)], env={**os.environ, 'NeoCLRBootstrapRoot': str(root),
                                'NeoCLRLibrarySlice': name}, check=True)
            imported = root / ('imported-' + name)
            subprocess.run([*bridge, '--library-implementation', str(compiled / 'System.Runtime.dll'),
                            str(core), owner, str(imported)], check=True)
            outputs = fragments((imported / 'Implementation.neoil').read_text(), name, owner)
            if args.check:
                for output, text in outputs.items():
                    if (GENERATED / output).read_text() != text:
                        raise SystemExit('Regenerated library differs: ' + output)
                continue
            inputs = [ROOT / path for path in SOURCES.values()]
            inputs += [ROOT / "runtime/raven/src/System/Tasks/TaskOperators.rvn", ROOT / "runtime/raven/src/System/Console/Streams.rvn"]
            inputs += [PROJECT, ROOT / 'build/NeoCLR.Raven.props']
            data = {'format': 'raven-library-bootstrap-v1', 'owner': owner,
                    'inputs': {str(p.relative_to(ROOT)): digest(p) for p in inputs},
                    'outputs': {output: hashlib.sha256(text.encode()).hexdigest() for output, text in outputs.items()},
                    'compilerSha256': digest(args.compiler), 'coreSha256': digest(core),
                    'exports': re.findall(r'(?m)^\.method (?:static |instance )(.+)', outputs[name + '.methods.neoil'])}
            generated.update(outputs)
            generated[name + '.json'] = json.dumps(data, indent=2) + '\n'
        # Do not publish a partial snapshot when a later implementation fails admission.
        GENERATED.mkdir(exist_ok=True)
        for output, text in generated.items():
            (GENERATED / output).write_text(text)
        print('Clean bootstrap regeneration matches checked-in implementation' if args.check else 'Generated Raven library bootstrap fragments')

if __name__ == '__main__':
    main()

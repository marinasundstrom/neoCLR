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
    'Tasks': 'System.Tasks.Task',

    'Array': 'System.Array',
    'Object': 'System.Object',
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
    'ConsoleReadError': 'System.IO.ConsoleReadError',
    'FileWriteError': 'System.IO.FileWriteError',
    'FileReadError': 'System.IO.FileReadError',
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
    'Path': 'System.IO.Path',
    'File': 'System.IO.File',
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
    'Tasks': 'runtime/raven/src/System/Tasks/Tasks.rvn',

    'Array': 'runtime/raven/src/System/Array.rvn',
    'Object': 'runtime/raven/src/System/Object.rvn',
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
    'ConsoleReadError': 'runtime/raven/src/System/IO/ConsoleReadError.rvn',
    'FileWriteError': 'runtime/raven/src/System/IO/FileWriteError.rvn',
    'FileReadError': 'runtime/raven/src/System/IO/FileReadError.rvn',
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
    'Path': 'runtime/raven/src/System/IO/Path/Functions.rvn',
    'File': 'runtime/raven/src/System/IO/File/Functions.rvn',
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
    lines = text.splitlines(keepends=True)
    methods, helpers, types = [], {}, []
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
        if match[1].startswith(owner + '.'):
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
    # Nongeneric classes can also own static factories. Merge their function roots
    # into the emitted class rather than leaving top-level method fragments.
    for body in types[:]:
        if body.startswith('.type class ' + owner + '\n'):
            types.remove(body)
            methods = [body[:-len('.end\n')] + ''.join(methods) + '.end\n']
    if name == 'String' and bootstrap:
        # The archived Neo profile uses borrowed collection interfaces. Keep its
        # scalar-independent String operations; Raven uses the complete source.
        methods = [re.sub(r'(?ms)^\.method instance (?:readonly byref )?(?:GetIterator|GetScalars)\(\).*?^\.end\n', '', body)
                   .replace('.implements System.Collections.Iterable<Char>\n', '') for body in methods]
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
    if name == 'String' and not bootstrap:
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
            compiled = root / 'compiled'
            if name not in ('Math', 'Linq', 'OptionOperators', 'OptionNestedOperators', 'ResultOperators', 'Path', 'File'):
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

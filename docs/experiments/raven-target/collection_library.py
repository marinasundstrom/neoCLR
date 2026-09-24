"""Generate the bounded Raven collection profile from the existing System sources.

No declaration-stub bodies are used. Raven-authored implementations replace migrated
types; remaining IL definitions adapt storage and receiver contracts for this target.
"""
import argparse
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[3]
COLLECTIONS = {'ArrayIterator', 'List', 'Iterable', 'Iterator'}


def adapt(text: str, name: str) -> str:
    text = text.replace('.type internal ', '.type internal class ')
    text = text.replace('instance readonly byref ', 'instance ').replace('instance byref ', 'instance ')
    text = text.replace('T[]&', 'arrayref<T>')
    for ty in COLLECTIONS:
        text = text.replace(f'System.Collections.{ty}<T>&', f'System.Collections.{ty}<T>')
    # Capacity is not a sequence of readable values. Reserve checked slots until Add writes them.
    text = text.replace('array.alloc T', 'array.reserve T')
    text = text.replace('-> Void', '-> noresult')
    text = re.sub(r'^\s*ldvoid\n', '\n', text, flags=re.M)
    text = re.sub(r'^\s*heap.new\n', '\n', text, flags=re.M)
    text = text.replace('interface.borrow ', 'castclass ')
    # Ordinary class stfld and no-result calls leave nothing to pop.
    text = re.sub(r'(stfld System\.Collections\.[^\n]+\n)\s*pop\n', r'\1', text)
    text = re.sub(r'(callvirt instance System\.Disposable::Dispose\(\)\n)\s*pop\n', r'\1', text)
    return text


def build(path: Path) -> str:
    if path.resolve().parent == ROOT / 'runtime/legacy':
        return (build(ROOT / 'runtime/raven/generated' / (path.stem + '.methods.neoil'))
                + build(ROOT / 'runtime/raven/generated' / (path.stem + '.helpers.neoil')))
    if path.parent == ROOT / 'runtime/System' and path.stem in {'Environment', 'Console', 'Clonable', 'LocalDateTime', 'Disposable', 'Equatable', 'Comparable', 'SystemClock', 'Closable'}:
        return build(ROOT / 'runtime/raven' / path.name)
    if path == ROOT / 'runtime/System/Storage/File.neoil':
        return build(ROOT / 'runtime/raven/FileText.neoil')
    if path == ROOT / 'runtime/System/Storage/Path.neoil':
        return build(ROOT / 'runtime/raven/Path.neoil')
    if path == ROOT / 'runtime/System/String.neoil':
        return build(ROOT / 'runtime/raven/String.neoil')
    if path == ROOT / 'runtime/System/Clock.neoil':
        return build(ROOT / 'runtime/raven/Clock.neoil')
    if path == ROOT / 'runtime/System/TypeInfo.neoil':
        return ''  # TypeInfo is part of the closed MemberInfo authoring slice.
    if path == ROOT / 'runtime/System/Type.neoil':
        return build(ROOT / 'runtime/raven/RuntimeContext.neoil')
    if path == ROOT / 'runtime/System/Date.neoil':
        return build(ROOT / 'runtime/raven/Date.neoil')
    if path == ROOT / 'runtime/System/Time.neoil':
        return build(ROOT / 'runtime/raven/Time.neoil')
    if path == ROOT / 'runtime/System/Collections/ArrayList.neoil':
        return build(ROOT / 'runtime/raven/ArrayList.neoil')
    if path == ROOT / 'runtime/System/Collections/Iterable.neoil':
        return build(ROOT / 'runtime/raven/Iterable.neoil')
    text = path.read_text()
    if path == ROOT / 'runtime/System/Collections/Iterator.neoil':
        return build(ROOT / 'runtime/raven/Iterator.neoil')
    if path.stem in COLLECTIONS | {'Disposable'}:
        text = adapt(text, path.stem)
    if path.stem in {'Equatable', 'Comparable', 'Clonable', 'Closable'}:
        text = text.replace('instance readonly byref ', 'instance ').replace('instance byref ', 'instance ')
    if path == ROOT / 'runtime/System/Reflection.neoil':
        # Raven owns descriptor bodies; legacy Neo retains its value-based profile.
        return (build(ROOT / 'runtime/raven/ParameterInfo.neoil')
                + build(ROOT / 'runtime/raven/Descriptors.neoil')
                + build(ROOT / 'runtime/raven/BindingFlags.neoil'))
    if path == ROOT / 'runtime/System/Collections/List.neoil':
        return build(ROOT / 'runtime/raven/CollectionContracts.neoil') + build(ROOT / 'runtime/raven/List.neoil')
    if path == ROOT / 'runtime/System/Array.neoil':
        return build(ROOT / 'runtime/raven/Array.neoil') + build(ROOT / 'runtime/raven/NativeMemory.neoil')
    if path.stem == 'Func':
        # The Raven profile declares instance ForEach on the managed Array<T> shape.
        text = text[:text.index('; Managed-array callback consumer')]
        text = text.replace('-> Void', '-> noresult')
        text = re.sub(r'^\s*ldvoid\n', '\n', text, flags=re.M)
    if path.parent == ROOT / 'runtime/neoCLR/Runtime':
        text = re.sub(r'System\.Type\b', 'System.Introspection.TypeInfo', text)
    lines = []
    for line in text.splitlines(keepends=True):
        include = re.fullmatch(r'\s*\.include "([^"]+)"\s*', line)
        lines.append(build(path.parent / include[1]) if include else line)
    result = ''.join(lines)
    if path.name == 'System.neoil':
        result = result.replace('.module System\n', '.module System\n.assembly {"name":"System.Runtime","full_name":"System.Runtime","modules":["System"],"references":[]}\n', 1)
        result += build(ROOT / 'runtime/raven/AssemblyInfo.neoil')
        result += build(ROOT / 'runtime/raven/ModuleInfo.neoil')
        result += build(ROOT / 'runtime/neoCLR/Runtime/AssemblyInfo.neoil')
        result += build(ROOT / 'runtime/raven/Object.neoil')
        result += build(ROOT / 'runtime/neoCLR/Runtime/EnumInspection.neoil')
        result += build(ROOT / 'runtime/raven/HashCode.neoil')
        result += build(ROOT / 'runtime/neoCLR/Runtime/ObjectTypeHandle.neoil')
        result += build(ROOT / 'runtime/neoCLR/Runtime/ObjectIdentity.neoil')
        result += build(ROOT / 'runtime/raven/SingleError.neoil')
        result += build(ROOT / 'runtime/raven/Linq.neoil')
        result += build(ROOT / 'runtime/raven/OptionOperators.neoil')
        result += build(ROOT / 'runtime/raven/OptionNestedOperators.neoil')
        result += build(ROOT / 'runtime/raven/ResultOperators.neoil')
        result += build(ROOT / 'runtime/raven/Map.neoil')
        result += build(ROOT / 'runtime/raven/Utf8.neoil')
        result += build(ROOT / 'runtime/raven/EntryKind.neoil')
        result += build(ROOT / 'runtime/raven/StorageLookupError.neoil')
        result += build(ROOT / 'runtime/raven/StorageMetadata.neoil')
        # Provider contracts require the Raven StorageItem/File/Directory hierarchy.
        # Keep them out of the legacy bootstrap manifest, which has static File helpers.
        result += build(ROOT / 'runtime/raven/StorageItems.neoil')
        result += build(ROOT / 'runtime/raven/StorageProvider.neoil')
        result += build(ROOT / 'runtime/raven/FileSystem.neoil')
        result += build(ROOT / 'runtime/raven/TextReadError.neoil')
        result += build(ROOT / 'runtime/raven/TextReader.neoil')
        result += build(ROOT / 'runtime/raven/TextWriter.neoil')
        result += build(ROOT / 'runtime/raven/StreamWriter.neoil')
        result += build(ROOT / 'runtime/neoCLR/Runtime/ConsoleStreams.neoil')
        result += build(ROOT / 'runtime/raven/StreamReader.neoil')
        result += build(ROOT / 'runtime/raven/SeekableStream.neoil')
        result += build(ROOT / 'runtime/raven/FileInputStream.neoil')
        result += build(ROOT / 'runtime/raven/FileOutputStream.neoil')
        result += build(ROOT / 'runtime/neoCLR/Runtime/FileStreams.neoil')
        result += build(ROOT / 'runtime/raven/DnsError.neoil')
        result += build(ROOT / 'runtime/raven/Dns.neoil')
        result += build(ROOT / 'runtime/neoCLR/Runtime/Dns.neoil')
        result += build(ROOT / 'runtime/raven/UnionProtocol.neoil')
        result += build(ROOT / 'runtime/raven/SocketError.neoil')
        result += build(ROOT / 'runtime/raven/Socket.neoil')
        result += build(ROOT / 'runtime/raven/UriError.neoil')
        result += build(ROOT / 'runtime/raven/Uri.neoil')
        result += build(ROOT / 'runtime/raven/HttpError.neoil')
        result += build(ROOT / 'runtime/raven/HttpClient.neoil')
        result += build(ROOT / 'runtime/neoCLR/Runtime/Sockets.neoil')
        result += build(ROOT / 'runtime/raven/Workers.neoil')
        result += build(ROOT / 'runtime/neoCLR/Runtime/Workers.neoil')
        result += build(ROOT / 'runtime/raven/TaskState.neoil')
        result += build(ROOT / 'runtime/raven/TaskOutcome.neoil')
        result += build(ROOT / 'runtime/raven/Tasks.neoil')
        result += build(ROOT / 'runtime/neoCLR/Runtime/Tasks.neoil')
        result += build(ROOT / 'runtime/raven/InvalidUtf8Error.neoil')
        result += build(ROOT / 'runtime/neoCLR/Runtime/Utf8.neoil')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    # Refuse to overwrite a library that may already be in use by a saved demo.
    source = build(ROOT / 'runtime/System.neoil')
    with args.output.open('x') as stream:
        stream.write('; Generated Raven collection profile; do not edit.\n' + source)

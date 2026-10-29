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
    text = text.replace('-> Void', '-> void')
    text = re.sub(r'^\s*ldvoid\n', '\n', text, flags=re.M)
    text = re.sub(r'^\s*heap.new\n', '\n', text, flags=re.M)
    text = text.replace('interface.borrow ', 'castclass ')
    # Ordinary class stfld and no-result calls leave nothing to pop.
    text = re.sub(r'(stfld System\.Collections\.[^\n]+\n)\s*pop\n', r'\1', text)
    text = re.sub(r'(callvirt instance System\.Disposable::Dispose\(\)\n)\s*pop\n', r'\1', text)
    return text


def build(path: Path) -> str:
    if path == ROOT / 'runtime/legacy/bootstrap/System/Int32ParseError.neoil':
        return ''  # Raven parsing uses the shared NumberParseError union.
    if path.resolve().parent == ROOT / 'runtime/legacy':
        return (build(ROOT / 'runtime/legacy/raven/generated' / (path.stem + '.methods.neoil'))
                + build(ROOT / 'runtime/legacy/raven/generated' / (path.stem + '.helpers.neoil')))
    if path.parent == ROOT / 'runtime/legacy/bootstrap/System' and path.stem in {'Environment', 'Console', 'Clonable', 'LocalDateTime', 'Disposable', 'EquatableTo', 'ComparableTo', 'ConvertibleInto', 'SystemClock', 'Closable'}:
        return build(ROOT / 'runtime/legacy/raven' / path.name)
    if path == ROOT / 'runtime/legacy/bootstrap/System/Storage/File.neoil':
        return build(ROOT / 'runtime/legacy/raven/FileText.neoil')
    if path == ROOT / 'runtime/legacy/bootstrap/System/Storage/Path.neoil':
        return build(ROOT / 'runtime/legacy/raven/Path.neoil')
    if path.parent == ROOT / 'runtime/legacy/bootstrap/System' and path.stem in {'Int16', 'Boolean', 'Double', 'UInt32', 'Int32', 'Single', 'Int64', 'UInt64', 'UInt16', 'SByte', 'Byte'}:
        return build(ROOT / ('runtime/legacy/raven/generated/' + path.stem + '.methods.neoil')) + build(ROOT / ('runtime/legacy/raven/generated/' + path.stem + '.helpers.neoil'))
    if path == ROOT / 'runtime/legacy/bootstrap/System/String.neoil':
        return build(ROOT / 'runtime/legacy/raven/String.neoil')
    if path == ROOT / 'runtime/legacy/bootstrap/System/Clock.neoil':
        return build(ROOT / 'runtime/legacy/raven/Clock.neoil')
    if path == ROOT / 'runtime/legacy/bootstrap/System/TypeInfo.neoil':
        return ''  # TypeInfo is part of the closed MemberInfo authoring slice.
    if path == ROOT / 'runtime/legacy/bootstrap/System/Type.neoil':
        return build(ROOT / 'runtime/legacy/raven/RuntimeContext.neoil')
    if path == ROOT / 'runtime/legacy/bootstrap/System/Date.neoil':
        return build(ROOT / 'runtime/legacy/raven/Date.neoil')
    if path == ROOT / 'runtime/legacy/bootstrap/System/Time.neoil':
        return build(ROOT / 'runtime/legacy/raven/Time.neoil')
    if path == ROOT / 'runtime/legacy/bootstrap/System/Collections/ArrayList.neoil':
        return build(ROOT / 'runtime/legacy/raven/ArrayList.neoil')
    if path == ROOT / 'runtime/legacy/bootstrap/System/Collections/Iterable.neoil':
        return build(ROOT / 'runtime/legacy/raven/Iterable.neoil')
    text = path.read_text()
    if path == ROOT / 'runtime/legacy/raven/generated/Descriptors.helpers.neoil':
        # Archived CLI snapshots predate the native field-reflection payload.
        # Adapt this test-only profile's storage without rewriting its generated
        # artifacts or reintroducing the bridge into production library builds.
        anchor = '.field private DefinitionIndex Int32\n.property instance FieldType()'
        if text.count(anchor) != 1:
            raise ValueError('Revisit archived RuntimeFieldInfo layout adaptation')
        text = text.replace(anchor,
            '.field private DefinitionIndex Int32\n'
            '.field private StoredIsInitOnly Boolean\n'
            '.field private StoredIsLiteral Boolean\n'
            '.field private StoredConstantValue System.Object\n'
            '.property instance FieldType()', 1)
        # Generic method flags precede Identity in the current Raven provider.
        # Keep this archived test profile aligned with the runtime snapshot ABI.
        anchor = '.field private IsAbstract Boolean\n.field private Identity String'
        if text.count(anchor) != 1:
            raise ValueError('Revisit archived RuntimeMethodInfo layout adaptation')
        text = text.replace(anchor,
            '.field private IsAbstract Boolean\n'
            '.field private StoredIsGenericMethod Boolean\n'
            '.field private StoredIsGenericMethodDefinition Boolean\n'
            '.field private StoredContainsGenericParameters Boolean\n'
            '.field private Identity String', 1)
    if path == ROOT / 'runtime/legacy/bootstrap/System/Collections/Iterator.neoil':
        return build(ROOT / 'runtime/legacy/raven/Iterator.neoil')
    if path.stem in COLLECTIONS | {'Disposable'}:
        text = adapt(text, path.stem)
    if path.stem in {'EquatableTo', 'ComparableTo', 'ConvertibleInto', 'Clonable', 'Closable'}:
        text = text.replace('instance readonly byref ', 'instance ').replace('instance byref ', 'instance ')
    if path == ROOT / 'runtime/legacy/bootstrap/System/Reflection.neoil':
        # Raven owns descriptor bodies; legacy Neo retains its value-based profile.
        return (build(ROOT / 'runtime/legacy/raven/ParameterInfo.neoil')
                + build(ROOT / 'runtime/legacy/raven/Descriptors.neoil')
                + build(ROOT / 'runtime/legacy/raven/ReflectionError.neoil')
                + build(ROOT / 'runtime/legacy/raven/BindingFlags.neoil'))
    if path == ROOT / 'runtime/legacy/bootstrap/System/Collections/List.neoil':
        return build(ROOT / 'runtime/legacy/raven/CollectionContracts.neoil') + build(ROOT / 'runtime/legacy/raven/List.neoil')
    if path == ROOT / 'runtime/legacy/bootstrap/System/Array.neoil':
        return build(ROOT / 'runtime/legacy/raven/Array.neoil') + build(ROOT / 'runtime/legacy/raven/NativeMemory.neoil')
    if path.stem == 'Function':
        # Array<T> owns ForEach in the Raven profile. There is no nominal callable declaration.
        return ''
    if path.parent == ROOT / 'runtime/legacy/bootstrap/neoCLR/Runtime':
        text = re.sub(r'System\.Type\b', 'System.Introspection.TypeInfo', text)
    lines = []
    for line in text.splitlines(keepends=True):
        include = re.fullmatch(r'\s*\.include "([^"]+)"\s*', line)
        lines.append(build(path.parent / include[1]) if include else line)
    result = ''.join(lines)
    if path.name == 'System.neoil':
        result = result.replace('.module System\n', '.module System\n.assembly {"name":"System.Runtime","full_name":"System.Runtime","modules":["System"],"references":[]}\n', 1)
        result += build(ROOT / 'runtime/legacy/raven/AssemblyInfo.neoil')
        result += build(ROOT / 'runtime/legacy/raven/ModuleInfo.neoil')
        result += build(ROOT / 'runtime/legacy/bootstrap/neoCLR/Runtime/AssemblyInfo.neoil')
        result += build(ROOT / 'runtime/legacy/raven/Object.neoil')
        result += build(ROOT / 'runtime/legacy/bootstrap/neoCLR/Runtime/EnumInspection.neoil')
        result += build(ROOT / 'runtime/legacy/raven/HashCode.neoil')
        result += build(ROOT / 'runtime/legacy/raven/GC.neoil')
        result += build(ROOT / 'runtime/legacy/bootstrap/neoCLR/Runtime/GC.neoil')
        result += build(ROOT / 'runtime/legacy/bootstrap/neoCLR/Runtime/ObjectTypeHandle.neoil')
        result += build(ROOT / 'runtime/legacy/bootstrap/neoCLR/Runtime/ReflectionExecution.neoil')
        result += build(ROOT / 'runtime/legacy/bootstrap/neoCLR/Runtime/ReflectionMembers.neoil')
        result += build(ROOT / 'runtime/legacy/bootstrap/neoCLR/Runtime/ObjectIdentity.neoil')
        result += build(ROOT / 'runtime/legacy/raven/SingleError.neoil')
        result += build(ROOT / 'runtime/legacy/raven/Linq.neoil')
        result += build(ROOT / 'runtime/legacy/raven/OptionalOperators.neoil')
        result += build(ROOT / 'runtime/legacy/raven/OptionalNestedOperators.neoil')
        result += build(ROOT / 'runtime/legacy/raven/ResultOperators.neoil')
        result += build(ROOT / 'runtime/legacy/raven/StringComparison.neoil')
        result += build(ROOT / 'runtime/legacy/raven/Comparers.neoil')
        result += build(ROOT / 'runtime/legacy/raven/TimeOffset.neoil')
        result += build(ROOT / 'runtime/legacy/raven/ZonedDateTime.neoil')
        result += build(ROOT / 'runtime/legacy/raven/DateTime.neoil')
        result += build(ROOT / 'runtime/legacy/raven/TimeZone.neoil')
        result += build(ROOT / 'runtime/legacy/raven/TimeZoneError.neoil')
        result += build(ROOT / 'runtime/legacy/raven/LocalTimeMapping.neoil')
        result += build(ROOT / 'runtime/legacy/raven/Calendar.neoil')
        result += build(ROOT / 'runtime/legacy/raven/Culture.neoil')
        result += build(ROOT / 'runtime/legacy/bootstrap/neoCLR/Runtime/TimeZones.neoil')
        result += build(ROOT / 'runtime/legacy/bootstrap/neoCLR/Runtime/SystemCultureName.neoil')
        result += build(ROOT / 'runtime/legacy/raven/Language.neoil')
        result += build(ROOT / 'runtime/legacy/raven/CultureProvider.neoil')
        result += build(ROOT / 'runtime/legacy/raven/FixedCultureProvider.neoil')
        result += build(ROOT / 'runtime/legacy/raven/SystemCultureProvider.neoil')

        result += build(ROOT / 'runtime/legacy/raven/DateTimeFormat.neoil')

        result += build(ROOT / 'runtime/legacy/raven/Map.neoil')
        result += build(ROOT / 'runtime/legacy/raven/Number.neoil')
        result += build(ROOT / 'runtime/legacy/raven/NumberParseError.neoil')
        result += build(ROOT / 'runtime/legacy/raven/BooleanParseError.neoil')
        result += build(ROOT / 'runtime/legacy/raven/Encoder.neoil')
        result += build(ROOT / 'runtime/legacy/raven/EncoderProgress.neoil')
        result += build(ROOT / 'runtime/legacy/raven/EncoderState.neoil')
        result += build(ROOT / 'runtime/legacy/raven/Encoding.neoil')
        result += build(ROOT / 'runtime/legacy/raven/Decoder.neoil')
        result += build(ROOT / 'runtime/legacy/raven/EncodingError.neoil')
        result += build(ROOT / 'runtime/legacy/raven/Encodings.neoil')
        result += build(ROOT / 'runtime/legacy/raven/Utf8.neoil')
        result += build(ROOT / 'runtime/legacy/raven/StringBuilder.neoil')
        result += build(ROOT / 'runtime/legacy/raven/EntryKind.neoil')
        result += build(ROOT / 'runtime/legacy/raven/StorageLookupError.neoil')
        result += build(ROOT / 'runtime/legacy/raven/StorageMetadata.neoil')
        # Provider contracts require the Raven StorageItem/File/Directory hierarchy.
        # Keep them out of the legacy bootstrap manifest, which has static File helpers.
        result += build(ROOT / 'runtime/legacy/raven/StorageItems.neoil')
        result += build(ROOT / 'runtime/legacy/raven/StorageProvider.neoil')
        result += build(ROOT / 'runtime/legacy/raven/FileSystem.neoil')
        result += build(ROOT / 'runtime/legacy/raven/TextReadError.neoil')
        result += build(ROOT / 'runtime/legacy/raven/TextReader.neoil')
        result += build(ROOT / 'runtime/legacy/raven/TextWriter.neoil')
        result += build(ROOT / 'runtime/legacy/raven/StreamWriter.neoil')
        result += build(ROOT / 'runtime/legacy/bootstrap/neoCLR/Runtime/ConsoleStreams.neoil')
        result += build(ROOT / 'runtime/legacy/raven/JsonError.neoil')
        result += build(ROOT / 'runtime/legacy/raven/HttpJsonError.neoil')
        result += build(ROOT / 'runtime/legacy/raven/JsonValue.neoil')
        result += build(ROOT / 'runtime/legacy/raven/MemoryStream.neoil')
        result += build(ROOT / 'runtime/legacy/raven/StreamReader.neoil')
        result += build(ROOT / 'runtime/legacy/raven/SeekableStream.neoil')
        result += build(ROOT / 'runtime/legacy/raven/FileInputStream.neoil')
        result += build(ROOT / 'runtime/legacy/raven/FileOutputStream.neoil')
        result += build(ROOT / 'runtime/legacy/bootstrap/neoCLR/Runtime/FileStreams.neoil')
        result += build(ROOT / 'runtime/legacy/raven/DnsError.neoil')
        result += build(ROOT / 'runtime/legacy/raven/Dns.neoil')
        result += build(ROOT / 'runtime/legacy/bootstrap/neoCLR/Runtime/Dns.neoil')
        result += build(ROOT / 'runtime/legacy/raven/UnionProtocol.neoil')
        result += build(ROOT / 'runtime/legacy/raven/SocketError.neoil')
        result += build(ROOT / 'runtime/legacy/raven/Socket.neoil')
        result += build(ROOT / 'runtime/legacy/raven/UriError.neoil')
        result += build(ROOT / 'runtime/legacy/raven/Uri.neoil')
        result += build(ROOT / 'runtime/legacy/raven/IPAddressError.neoil')
        result += build(ROOT / 'runtime/legacy/raven/IPAddress.neoil')
        result += build(ROOT / 'runtime/legacy/raven/Cancellation.neoil')
        result += build(ROOT / 'runtime/legacy/raven/HttpStatusCode.neoil')
        result += build(ROOT / 'runtime/legacy/raven/HttpError.neoil')
        result += build(ROOT / 'runtime/legacy/raven/HttpClient.neoil')
        result += build(ROOT / 'runtime/legacy/bootstrap/neoCLR/Runtime/Sockets.neoil')
        result += build(ROOT / 'runtime/legacy/raven/Workers.neoil')
        result += build(ROOT / 'runtime/legacy/bootstrap/neoCLR/Runtime/Workers.neoil')
        result += build(ROOT / 'runtime/legacy/raven/TaskState.neoil')
        result += build(ROOT / 'runtime/legacy/raven/TaskOutcome.neoil')
        result += build(ROOT / 'runtime/legacy/raven/Tasks.neoil')
        result += build(ROOT / 'runtime/legacy/bootstrap/neoCLR/Runtime/Tasks.neoil')
        result += build(ROOT / 'runtime/legacy/raven/InvalidUtf8Error.neoil')
        result += build(ROOT / 'runtime/legacy/bootstrap/neoCLR/Runtime/Utf8.neoil')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    # Refuse to overwrite a library that may already be in use by a saved demo.
    source = build(ROOT / 'runtime/legacy/bootstrap/System.neoil')
    with args.output.open('x') as stream:
        stream.write('; Generated Raven collection profile; do not edit.\n' + source)

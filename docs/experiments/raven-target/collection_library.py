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
    if path.parent == ROOT / 'runtime/System' and path.stem in {'Environment', 'Console', 'Clonable', 'LocalDateTime', 'Disposable', 'Equatable', 'Comparable', 'SystemClock', 'Closable'}:
        return build(ROOT / 'runtime/raven' / path.name)
    if path == ROOT / 'runtime/System/Clock.neoil':
        return build(ROOT / 'runtime/raven/Clock.neoil')
    if path == ROOT / 'runtime/System/TypeInfo.neoil':
        return build(ROOT / 'runtime/raven/TypeInfo.neoil')
    if path == ROOT / 'runtime/System/Type.neoil':
        return build(ROOT / 'runtime/raven/Type.neoil')
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
        text = build(ROOT / 'runtime/raven/Iterator.neoil') + text[text.index('; Retains the original managed buffer'):]
    if path.stem in COLLECTIONS | {'Disposable'}:
        text = adapt(text, path.stem)
    if path.stem in {'Equatable', 'Comparable', 'Clonable', 'Closable'}:
        text = text.replace('instance readonly byref ', 'instance ').replace('instance byref ', 'instance ')
    if path.stem in {'TypeInfo', 'Reflection'}:
        if path.stem == 'Reflection':
            # Replace the complete descriptor declaration with checked Raven bodies.
            start = text.index('.type System.Introspection.ParameterInfo\n')
            end = text.index('; Internal construction contract:', start)
            text = text[:start] + build(ROOT / 'runtime/raven/ParameterInfo.neoil') + '\n' + text[end:]
        # Immutable reflection snapshots use ordinary class identity in the target.
        text = re.sub(r'^\.type (abstract )?(System\.(?:Type|Introspection\.(?:TypeInfo|MemberInfo|FieldInfo|MethodInfo|PropertyInfo|ParameterInfo)))$',
                      lambda m: '.type class ' + (m[1] or '') + m[2], text, flags=re.M)
        # These private construction helpers are replaced by trusted snapshot factories.
        text = re.sub(r'    \.method internal instance byref \.ctor[^\n]*\n.*?    \.end\n', '', text, flags=re.S)
        text = text.replace('instance readonly byref ', 'instance ').replace('instance byref ', 'instance ')
        text = text.replace('ldloca method', 'ldloc method')
    if path == ROOT / 'runtime/System/Collections/List.neoil':
        return build(ROOT / 'runtime/raven/CollectionContracts.neoil') + build(ROOT / 'runtime/raven/List.neoil')
    if path.stem == 'Array':
        return (ROOT / 'runtime/raven/Array.neoil').read_text() + (ROOT / 'runtime/raven/NativeMemory.neoil').read_text()
    if path.stem == 'Func':
        # The Raven profile declares instance ForEach on the managed Array<T> shape.
        text = text[:text.index('; Managed-array callback consumer')]
        text = text.replace('-> Void', '-> noresult')
        text = re.sub(r'^\s*ldvoid\n', '\n', text, flags=re.M)
    lines = []
    for line in text.splitlines(keepends=True):
        include = re.fullmatch(r'\s*\.include "([^"]+)"\s*', line)
        lines.append(build(path.parent / include[1]) if include else line)
    result = ''.join(lines)
    if path.name == 'System.neoil':
        result += '\n.type class System.Object\n.end\n'
        result += (ROOT / 'runtime/raven/SingleError.neoil').read_text()
        result += build(ROOT / 'runtime/raven/Linq.neoil')
        result += (ROOT / 'runtime/raven/ArrayEnumerable.neoil').read_text()
        result += build(ROOT / 'runtime/raven/Map.neoil')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    # Refuse to overwrite a library that may already be in use by a saved demo.
    source = build(ROOT / 'runtime/System.neoil')
    with args.output.open('x') as stream:
        stream.write('; Generated Raven collection profile; do not edit.\n' + source)

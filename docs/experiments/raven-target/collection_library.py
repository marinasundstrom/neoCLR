"""Generate the bounded Raven collection profile from the existing System sources.

No declaration-stub bodies are used. Keep the collection algorithms shared with the
legacy library while adapting their storage and receiver contracts for this target.
"""
import argparse
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[3]
COLLECTIONS = {'ArrayList', 'ArrayListState', 'ArrayIterator', 'List', 'Iterable', 'Iterator'}


def adapt(text: str, name: str) -> str:
    if name == 'ArrayList':
        text = text.replace('; Value wrapper with coherent shared managed state. Copy() duplicates the sequence.',
                            '; Nominal class with shared managed state. Copy() duplicates the sequence.')
    if name == 'ArrayList':
        # Class constructors initialize the allocated receiver, never replace its identity.
        for signature, capacity in [('()', 'ldc.i4 0'), ('(Int32 capacity)', 'ldarg capacity')]:
            pattern = r'    \.method instance \.ctor' + re.escape(signature) + r' -> Void.*?    \.end'
            body = f"""    .method instance .ctor{signature} -> Void
        {capacity}
        ldc.i4 0
        blt Invalid
        ldarg this
        {capacity}
        array.alloc T
        ldc.i4 0
        newobj System.Collections.ArrayListState<T>
        stfld System.Collections.ArrayList<T>::State
        ret
    Invalid:
        fault \"ArrayList capacity must be non-negative\"
    .end"""
            text, count = re.subn(pattern, lambda _: body, text, flags=re.S)
            if count != 1:
                raise ValueError('ArrayList constructor boundary changed')
    text = text.replace('.type internal ', '.type internal class ')
    text = re.sub(r'^\.type (System\.Collections\.ArrayList<T>)$', r'.type class \1', text, flags=re.M)
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
    text = text.replace('ldloca copy', 'ldloc copy')
    # Ordinary class stfld and no-result calls leave nothing to pop.
    text = re.sub(r'(stfld System\.Collections\.[^\n]+\n)\s*pop\n', r'\1', text)
    text = re.sub(r'(call instance System\.Collections\.ArrayList<T>::(?:CheckIndex|Add)\([^\n]+\n)\s*pop\n', r'\1', text)
    text = re.sub(r'(callvirt instance System\.Disposable::Dispose\(\)\n)\s*pop\n', r'\1', text)
    return text


def build(path: Path) -> str:
    text = path.read_text()
    if path.stem in COLLECTIONS | {'Disposable'}:
        text = adapt(text, path.stem)
    if path.stem in {'Equatable', 'Comparable', 'Clonable', 'Closable'}:
        text = text.replace('instance readonly byref ', 'instance ').replace('instance byref ', 'instance ')
    if path.stem in {'Type', 'Reflection'}:
        # Immutable reflection snapshots use ordinary class identity in the target.
        text = re.sub(r'^\.type (abstract )?(System\.(?:Type|Reflection\.(?:MemberInfo|FieldInfo|MethodInfo|PropertyInfo|ParameterInfo)))$',
                      lambda m: '.type class ' + (m[1] or '') + m[2], text, flags=re.M)
        # These private construction helpers are replaced by trusted snapshot factories.
        text = re.sub(r'    \.method internal instance byref \.ctor[^\n]*\n.*?    \.end\n', '', text, flags=re.S)
        text = text.replace('instance readonly byref ', 'instance ').replace('instance byref ', 'instance ')
        text = text.replace('ldloca method', 'ldloc method')
    if path.stem == 'List':
        return (ROOT / 'runtime/raven/CollectionContracts.neoil').read_text() + (ROOT / 'runtime/raven/List.neoil').read_text()
    if path.stem == 'Array':
        return (ROOT / 'runtime/raven/Array.neoil').read_text() + (ROOT / 'runtime/raven/NativeMemory.neoil').read_text()
    if path.stem == 'Func':
        text = text.replace('readonly T[]& array', 'arrayref<T> array').replace('-> Void', '-> noresult')
        text = re.sub(r'^\s*ldvoid\n', '\n', text, flags=re.M)
    lines = []
    for line in text.splitlines(keepends=True):
        include = re.fullmatch(r'\s*\.include "([^"]+)"\s*', line)
        lines.append(build(path.parent / include[1]) if include else line)
    result = ''.join(lines)
    if path.name == 'System.neoil':
        result += '\n.type class System.Object\n.end\n'
        result += (ROOT / 'runtime/raven/SingleError.neoil').read_text()
        result += (ROOT / 'runtime/raven/Linq.neoil').read_text()
        result += (ROOT / 'runtime/raven/ArrayEnumerable.neoil').read_text()
        result += (ROOT / 'runtime/raven/Map.neoil').read_text()
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    # Refuse to overwrite a library that may already be in use by a saved demo.
    source = build(ROOT / 'runtime/System.neoil')
    with args.output.open('x') as stream:
        stream.write('; Generated Raven collection profile; do not edit.\n' + source)

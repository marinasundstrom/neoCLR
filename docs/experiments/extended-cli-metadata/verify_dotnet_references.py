"""Shared table and catalog-scoped equality/rejection vectors for independent codecs."""
import base64
from dataclasses import replace
from itertools import combinations
import json
import struct
from uuid import UUID

from codec import FormatError, decode
from references import Bindings, Definition, decode_bindings, encode_bindings, read_profile, resolve
from signatures import Context, Node, encode_signature
from test_references import CATALOG, TYPE, CONTRACT, METHOD, OTHER_METHOD, fixture
from verify_pe import HERE, OUTPUT, ROOT, command, require


def b64(value):
    return base64.b64encode(value).decode('ascii')


def reference(value):
    return [str(value.assembly), str(value.module), value.token]


def spec(root, context, bindings, catalog):
    return dict(signature=b64(encode_signature(root, context, references=True)),
                bindings=b64(encode_bindings(bindings)), catalog=[
                    dict(reference=reference(ref), kind=value.kind, arity=value.arity,
                         owner=None if value.owner is None else reference(value.owner))
                    for ref, value in catalog.items()])


def main():
    vectors = []
    root, context, bindings = read_profile(decode(fixture(), {2: 1, 3: 1}))
    baseline = (root, context, bindings, CATALOG)

    def compare(name, left, right):
        result = 'equal' if resolve(*left) == resolve(*right) else 'different'
        vectors.append(dict(name=name, left=spec(*left), right=spec(*right), result=result))

    def reject(name, value):
        try:
            resolve(*value)
        except FormatError:
            vectors.append(dict(name=name, left=spec(*value), result='reject'))
        else:
            raise RuntimeError('Python accepted invalid ' + name)

    compare('reference-renumbering', baseline,
            (*read_profile(decode(fixture(True), {2: 1, 3: 1})), CATALOG))
    for name, old, new in [('method-owner', METHOD, OTHER_METHOD),
                           ('assembly-scope', TYPE, replace(TYPE, assembly=UUID(int=10))),
                           ('module-scope', TYPE, replace(TYPE, module=UUID(int=11))),
                           ('self-contract', CONTRACT, replace(CONTRACT, token=0x02000003))]:
        catalog = dict(CATALOG)
        catalog[new] = catalog[old]
        if old == TYPE:
            catalog[METHOD] = replace(catalog[METHOD], owner=new)
        other_bindings = replace(bindings, references=tuple(new if ref == old else ref for ref in bindings.references))
        compare(name, baseline, (root, context, other_bindings, catalog))

    def local(node):
        return node, Context(), Bindings(()), {}

    i, s, u = Node('int32'), Node('string'), Node('unit')
    for kind in ('union', 'intersection'):
        compare(kind + '-normalization', local(Node(kind, (i, Node(kind, (s, i))))), local(Node(kind, (s, i))))
        compare(kind + '-singleton-wrapper', local(Node(kind, (i, i))), local(i))
    compare('tuple-order', local(Node('tuple', (i, s))), local(Node('tuple', (s, i))))
    compare('array-storage', local(Node('array', (i,))), local(Node('array_ref', (i,))))
    compare('unit-no-result', local(Node('function', (u,))), local(Node('function', (u,), no_result=True)))
    for first, second in combinations(('value', 'ref', 'readonly_ref', 'out', 'out_when_true'), 2):
        compare('mode-' + first + '-' + second,
                local(Node('function', (i, Node('bool')), modes=(first,))),
                local(Node('function', (i, Node('bool')), modes=(second,))))
    for name, catalog in [
        ('missing-catalog', {}), ('binder-arity', {**CATALOG, TYPE: Definition('type', 2)}),
        ('self-not-interface', {**CATALOG, CONTRACT: Definition('type')}),
        ('self-generic', {**CATALOG, CONTRACT: Definition('interface', 1)}),
        ('method-owner-mismatch', {**CATALOG, METHOD: Definition('method', 1, CONTRACT)}),
        ('wrong-definition-kind', {**CATALOG, TYPE: Definition('method', 1, TYPE)}),
        ('negative-arity', {**CATALOG, TYPE: Definition('type', -1)}),
        ('excessive-arity', {**CATALOG, TYPE: Definition('type', 257)}),
        ('method-no-owner', {**CATALOG, METHOD: Definition('method', 1)}),
        ('type-with-owner', {**CATALOG, TYPE: Definition('type', 1, CONTRACT)})
    ]:
        reject(name, (root, context, bindings, catalog))
    for name, other in [('missing-type-owner', replace(bindings, type_owner=0)),
                        ('missing-method-owner', replace(bindings, method_owner=0)),
                        ('missing-self-owner', replace(bindings, self_owner=0))]:
        reject(name, (root, context, other, CATALOG))
    for index in (1, 3, 4):
        reject('bad-nominal-' + str(index), (Node('nominal', index=index), context, bindings, CATALOG))

    tables = {'empty': encode_bindings(Bindings(())), 'fixture': encode_bindings(bindings),
              'mixed-uuid': encode_bindings(Bindings((replace(TYPE,
                  assembly=UUID('01234567-89ab-cdef-0123-456789abcdef'),
                  module=UUID('fedcba98-7654-3210-fedc-ba9876543210')),), type_owner=1)),
              'maximum': encode_bindings(Bindings(tuple(replace(TYPE, token=0x02000001+n) for n in range(256))))}
    for name, payload in tables.items():
        require(encode_bindings(decode_bindings(payload)) == payload, 'Python table roundtrip')
        vectors.append(dict(name='table-' + name, table=b64(payload), result='valid'))
    raw = tables['mixed-uuid']
    malformed = {'truncated-' + str(n): raw[:n] for n in range(len(raw))}
    malformed.update({'trailing': raw + b'x', 'zero-assembly': raw[:8] + bytes(16) + raw[24:],
                      'zero-module': raw[:24] + bytes(16) + raw[40:],
                      'duplicate': struct.pack('<HHHH', 2, 0, 0, 0) + raw[8:] * 2})
    for name, offset, code, value in [('count', 0, 'H', 257), ('owner-range', 2, 'H', 2),
                                    ('owner-kind', 4, 'H', 1), ('token-kind', 40, 'I', 0x01000001),
                                    ('token-row', 40, 'I', 0x02000000)]:
        broken = bytearray(raw)
        struct.pack_into('<' + code, broken, offset, value)
        malformed[name] = bytes(broken)
    for name, payload in malformed.items():
        try:
            decode_bindings(payload)
        except FormatError:
            vectors.append(dict(name='table-' + name, table=b64(payload), result='reject'))
        else:
            raise RuntimeError('Python accepted invalid table ' + name)

    out = OUTPUT / 'dotnet-library'
    print(command('dotnet', 'build', ROOT / 'tools/metadata/MetadataConformance/MetadataConformance.csproj',
                  '-o', out, '--nologo', '-v', 'minimal'))
    path = out / 'reference-vectors.json'
    path.write_text(json.dumps(vectors))
    print(command('dotnet', out / 'MetadataConformance.dll', 'reference-vectors', path).strip())
    report = dict(date='2026-09-30', scope='explicit host catalog; no PE declaration loading or execution',
                  dotnet_sdk=command('dotnet', '--version').strip(), target_framework='net10.0',
                  shared_vectors=len(vectors), equality_vectors=sum('right' in v for v in vectors),
                  rejection_vectors=sum(v['result'] == 'reject' for v in vectors),
                  independent_network_uuid_emission_and_ownership_checks='passed',
                  cases=[dict(name=v['name'], result=v['result']) for v in vectors])
    (HERE / 'dotnet-references-validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k: v for k, v in report.items() if k != 'cases'}, indent=2))


if __name__ == '__main__':
    main()

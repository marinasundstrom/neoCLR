"""Independent .NET synthesized-member tables, contracts and identity conformance."""
import json
import struct

from codec import FormatError, decode
from members import MemberRef, decode_members, encode_members, resolve_members
from references import Bindings, read_profile, resolve
from signatures import Context, Node
from test_references import CATALOG, fixture
from verify_dotnet_references import b64, spec
from verify_pe import HERE, OUTPUT, ROOT, command, require


def main():
    vectors = []
    i, s, u, b = [Node(kind) for kind in ('int32', 'string', 'unit', 'bool')]

    def local(root):
        return root, Context(), Bindings(()), {}

    def valid(name, profile, members):
        descriptors = resolve_members(members, *profile)
        root, context, bindings, catalog = profile
        contracts = []
        for member, descriptor in zip(members, descriptors):
            if member.operation == 'array_length':
                parameters, result = (), 'native_uint'
                require(descriptor.result == ('intrinsic', 'native_uint'), 'Python array result')
            elif member.operation == 'tuple_element':
                parameters, result = (), root.children[member.element]
            elif member.operation == 'tuple_deconstruct':
                parameters, result = root.children, u
            else:
                parameters, result = root.children[:-1], root.children[-1]
            def child(node):
                return spec(node, context, bindings, catalog)
            require(tuple(resolve(node, context, bindings, catalog) for node in parameters) == descriptor.parameters,
                    'Python parameter contract')
            if not isinstance(result, str):
                require(resolve(result, context, bindings, catalog) == descriptor.result, 'Python result contract')
            contracts.append(dict(parameters=[child(node) for node in parameters], modes=descriptor.modes,
                                  result=result if isinstance(result, str) else child(result), no_result=descriptor.no_result))
        vectors.append(dict(name=name, profile=spec(*profile), table=b64(encode_members(members)),
                            contracts=contracts, result='valid'))

    def compare(name, left, right, members):
        first, second = resolve_members(members, *left), resolve_members(members, *right)
        equal = tuple(d.identity for d in first) == tuple(d.identity for d in second)
        if equal:
            require(first == second, 'Python equal member contracts')
        vectors.append(dict(name=name, profile=spec(*left), right=spec(*right),
                            table=b64(encode_members(members)), result='equal' if equal else 'different'))

    valid('empty', local(i), [])
    for kind in ('array', 'array_ref'):
        valid(kind, local(Node(kind, (i,))), [MemberRef('array_length')])
    valid('tuple', local(Node('tuple', (i, s))), [MemberRef('tuple_element'),
          MemberRef('tuple_element', element=1), MemberRef('tuple_deconstruct')])
    valid('maximum-tuple', local(Node('tuple', (i,) * 256)),
          [MemberRef('tuple_element', element=n) for n in range(256)])
    valid('maximum-deconstruct', local(Node('tuple', (i,) * 256)), [MemberRef('tuple_deconstruct')])
    invoke = [MemberRef('function_invoke')]
    for mode in ('value', 'ref', 'readonly_ref', 'out', 'out_when_true'):
        valid('function-' + mode, local(Node('function', (i, b), modes=(mode,))), invoke)
    for flag in (False, True):
        valid('no-result-' + str(flag), local(Node('function', (u,), no_result=flag)), invoke)
    first = (*read_profile(decode(fixture(), {2: 1, 3: 1})), CATALOG)
    second = (*read_profile(decode(fixture(True), {2: 1, 3: 1})), CATALOG)
    valid('reference-owner', first, invoke)
    compare('reference-renumbering', first, second, invoke)
    compare('array-storage', local(Node('array', (i,))), local(Node('array_ref', (i,))), [MemberRef('array_length')])
    compare('no-result-identity', local(Node('function', (u,))), local(Node('function', (u,), no_result=True)), invoke)
    compare('function-mode-identity', local(Node('function', (i, b), modes=('ref',))),
            local(Node('function', (i, b), modes=('out',))), invoke)
    compare('normalized-owner', local(Node('tuple', (Node('union', (i, s)),))),
            local(Node('tuple', (Node('union', (s, i, i)),))), [MemberRef('tuple_element')])

    for name, profile, members in [
        ('length-scalar', local(i), [MemberRef('array_length')]),
        ('element-array', local(Node('array', (i,))), [MemberRef('tuple_element')]),
        ('element-range', local(Node('tuple', (i,))), [MemberRef('tuple_element', element=1)]),
        ('invoke-tuple', local(Node('tuple', (i,))), invoke),
        ('deconstruct-function', local(Node('function', (u,))), [MemberRef('tuple_deconstruct')]),
        ('unresolved-owner', (*first[:3], {}), invoke),
        ('empty-unresolved-owner', (*first[:3], {}), [])
    ]:
        try:
            resolve_members(members, *profile)
        except FormatError:
            vectors.append(dict(name=name, profile=spec(*profile), table=b64(encode_members(members)), result='reject'))
        else:
            raise RuntimeError('Python accepted invalid contract ' + name)
    golden = encode_members([MemberRef('tuple_element', element=1), MemberRef('tuple_deconstruct')])
    malformed = {'truncated-' + str(n): golden[:n] for n in range(len(golden))}
    malformed.update({'trailing': golden + b'x', 'duplicate': b'\2\0' + golden[2:8] * 2})
    for name, offset, code, value in [('count', 0, 'H', 257), ('owner-zero', 2, 'H', 0),
                                    ('owner-range', 2, 'H', 2), ('operation', 4, 'B', 255),
                                    ('flags', 5, 'B', 1), ('operand', 12, 'H', 1),
                                    ('element-limit', 6, 'H', 256)]:
        broken = bytearray(golden)
        struct.pack_into('<' + code, broken, offset, value)
        malformed[name] = bytes(broken)
    for name, payload in malformed.items():
        try:
            decode_members(payload)
        except FormatError:
            vectors.append(dict(name=name, table=b64(payload), result='reject'))
        else:
            raise RuntimeError('Python accepted malformed table ' + name)
    out = OUTPUT / 'dotnet-library'
    print(command('dotnet', 'build', ROOT / 'tools/metadata/MetadataConformance/MetadataConformance.csproj',
                  '-o', out, '--nologo', '-v', 'minimal'))
    path = out / 'member-vectors.json'
    path.write_text(json.dumps(vectors))
    print(command('dotnet', out / 'MetadataConformance.dll', 'member-vectors', path).strip())
    report = dict(date='2026-09-30', scope='section-4 tables and catalog-scoped derived contracts; no invocation or assembly loading',
                  dotnet_sdk=command('dotnet', '--version').strip(), target_framework='net10.0',
                  shared_vectors=len(vectors), contract_vectors=sum('contracts' in v for v in vectors),
                  identity_vectors=sum('right' in v for v in vectors),
                  rejection_vectors=sum(v['result'] == 'reject' for v in vectors),
                  independent_golden_emission_writer_and_ownership_checks='passed',
                  cases=[dict(name=v['name'], result=v['result']) for v in vectors])
    (HERE / 'dotnet-members-validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k: v for k, v in report.items() if k != 'cases'}, indent=2))


if __name__ == '__main__':
    main()

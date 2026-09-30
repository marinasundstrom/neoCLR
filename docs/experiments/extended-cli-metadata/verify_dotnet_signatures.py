"""Independent Python/.NET structural-signature conformance (no nominal resolution)."""
import hashlib
import json
import struct

from codec import FormatError, decode
from signatures import Context, Node, decode_signature, encode_signature
from test_signatures import nested
from verify_pe import HERE, OUTPUT, ROOT, command, require


def raw_node(tag, payload=b''):
    return struct.pack('<BI', tag, len(payload)) + payload


def main():
    out = OUTPUT / 'dotnet-library'
    print(command('dotnet', 'build', ROOT / 'tools/metadata/MetadataConformance/MetadataConformance.csproj',
                  '-o', out, '--nologo', '-v', 'minimal'))
    consumer = out / 'MetadataConformance.dll'
    print(command('dotnet', consumer, 'signature-selftest').strip())
    vectors = {}
    for fixture in sorted((HERE / 'fixtures').glob('*.neox')):
        for section in decode(fixture.read_bytes(), {1: 1, 2: 1, 3: 1, 4: 1}):
            if section.kind in (1, 3):
                vectors[fixture.stem] = (section.payload, section.kind == 3)
    i, s, b, u = [Node(name) for name in ('int32', 'string', 'bool', 'unit')]
    for mode in ('value', 'ref', 'readonly_ref', 'out', 'out_when_true'):
        vectors[mode] = (encode_signature(Node('function', (i, b), modes=(mode,))), False)
    for name, node in [('no-result', Node('function', (u,), no_result=True)),
                       ('unit-result', Node('function', (u,))), ('array-ref', Node('array_ref', (i,))),
                       ('union-duplicates', Node('union', (s, i, i))),
                       ('nominal', Node('nominal', (i,), index=256))]:
        vectors[name] = (encode_signature(node, references=True), name == 'nominal')
    hashes = {}
    incoming, outgoing = out / 'signature.input', out / 'signature.output'
    for name, (payload, references) in vectors.items():
        incoming.write_bytes(payload)
        flags = ['references'] if references else []
        command('dotnet', consumer, 'signature-roundtrip', incoming, outgoing, *flags)
        require(outgoing.read_bytes() == payload, 'signature byte roundtrip differs: ' + name)
        require(decode_signature(outgoing.read_bytes(), references=references) ==
                decode_signature(payload, references=references), 'signature meaning differs: ' + name)
        hashes[name] = hashlib.sha256(payload).hexdigest()
    command('dotnet', consumer, 'signature-emit', outgoing)
    require(outgoing.read_bytes() == encode_signature(nested(), Context(1, 1, True)), 'independent writers disagree')

    valid = encode_signature(nested(), Context(1, 1, True))
    cases = {f'truncated-{size}': (valid[:size], False) for size in range(len(valid))}
    zero = b'\0' * 5
    cases.update({
        'trailing': (valid + b'x', False),
        'unknown-node': (zero + raw_node(255), False),
        'oversized-node': (zero + struct.pack('<BI', 1, 0xffffffff), False),
        'bad-self-flag': (b'\0\0\0\0\2' + raw_node(1), False),
        'bad-context-arity': (struct.pack('<HHB', 257, 0, 0) + raw_node(1), False),
        'unbound-type': (zero + raw_node(5, b'\0\0'), False),
        'unbound-method': (zero + raw_node(6, b'\0\0'), False),
        'unbound-self': (zero + raw_node(7), False),
        'empty-tuple': (zero + raw_node(9, b'\0\0'), False),
        'singleton-union': (zero + raw_node(11, b'\1\0' + raw_node(1)), False),
        'bad-arity': (zero + raw_node(9, b'\1\1'), False),
        'leaf-payload': (zero + raw_node(1, b'x'), False),
        'nominal-disallowed': (vectors['nominal'][0], False),
        'nominal-zero': (zero + raw_node(15, struct.pack('<HH', 0, 0)), True),
        'nominal-out-of-range': (zero + raw_node(15, struct.pack('<HH', 257, 0)), True),
    })
    for name, convention, flags, mode, result in [
        ('bad-convention', 1, 0, 0, 3), ('bad-flags', 0, 2, 0, 3),
        ('bad-mode', 0, 0, 255, 3), ('bad-no-result', 0, 1, 0, 1),
        ('bad-conditional-output', 0, 0, 4, 1)
    ]:
        cases[name] = (zero + raw_node(10, struct.pack('<BBHB', convention, flags, 1, mode) + raw_node(1) + raw_node(result)), False)
    deep = raw_node(1)
    for _ in range(33):
        deep = raw_node(8, deep)
    cases['depth-limit'] = (zero + deep, False)
    wide = raw_node(9, struct.pack('<H', 256) + raw_node(1) * 256)
    cases['node-limit'] = (zero + raw_node(9, struct.pack('<H', 16) + wide * 16), False)
    for name, (payload, references) in cases.items():
        try:
            decode_signature(payload, references=references)
        except FormatError:
            pass
        else:
            raise RuntimeError('Python accepted malformed ' + name)
        incoming.write_bytes(payload)
        flags = ['references'] if references else []
        require(command('dotnet', consumer, 'signature-reject', incoming, *flags).strip() == 'rejected',
                '.NET accepted malformed ' + name)
    report = {
        'date': '2026-09-30', 'scope': 'local/reference-profile signature syntax only; no resolution or execution',
        'dotnet_sdk': command('dotnet', '--version').strip(), 'target_framework': 'net10.0',
        'positive_cross_reader_vectors': len(vectors),
        'independent_dotnet_nested_emission_matches_python': True,
        'writer_ownership_context_and_boundary_checks': 'passed',
        'negative_cross_reader_vectors': len(cases),
        'negative_cases': list(cases), 'positive_sha256': hashes,
    }
    (HERE / 'dotnet-signatures-validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k: v for k, v in report.items() if k not in ('negative_cases', 'positive_sha256')}, indent=2))


if __name__ == '__main__':
    main()

"""Reference-profile composition and typed document conformance."""
from dataclasses import replace
import json

from codec import FormatError, Section, decode, encode
from members import MemberRef, encode_members, read_members
from references import Bindings, encode_bindings, read_profile
from signatures import Node, encode_signature
from test_references import CATALOG, fixture
from verify_dotnet_references import b64, spec
from verify_pe import HERE, OUTPUT, ROOT, command, require


def main():
    vectors = []
    schemas = {2: 1, 3: 1, 4: 1}
    sections = decode(fixture(), schemas)
    tuple_image = (HERE / 'fixtures/tuple-members.neox').read_bytes()
    tuple_sections = decode(tuple_image, schemas)

    def check(name, image, invalid=False):
        try:
            decoded = decode(image, schemas)
            profile = read_profile(decoded)
            if profile is None:
                raise FormatError('expected reference profile')
            members = read_members(decoded, profile)
        except FormatError:
            require(invalid, 'unexpected Python rejection ' + name)
            vectors.append(dict(name=name, image=b64(image), reject=True))
            return
        require(not invalid, 'Python accepted invalid ' + name)
        vectors.append(dict(name=name, image=b64(image), reject=False, has_members=members is not None,
                            unknown=sum(s.kind > 4 or (s.kind == 4 and s.version != 1) for s in decoded),
                            profile=spec(*profile, CATALOG)))

    def image(parts):
        # Test-only permissive writer enables construction of unsupported required schemas.
        return encode(parts, {s.kind: s.version for s in parts})

    check('references-a', fixture())
    check('references-b', fixture(True))
    check('tuple-fixture', tuple_image)
    check('reverse-directory', image(tuple_sections[::-1]))
    check('empty-members', image(sections + [Section(4, 1, True, encode_members([]))]))
    check('opaque-extension', image([Section(60000, 99, False, b'opaque\0\xff')] + sections))
    check('opaque-future-member-schema', image(sections + [Section(4, 99, False, b'future')]))
    for name, parts in [
        ('empty', []), ('missing-signature', sections[:1]), ('missing-references', sections[1:]),
        ('local-only', [Section(1, 1, False, encode_signature(Node('int32')))]),
        ('mixed-local', sections + [Section(1, 1, False, encode_signature(Node('int32')))]),
        ('member-only', tuple_sections[-1:]),
        ('optional-references', [replace(sections[0], required=False), sections[1]]),
        ('optional-signature', [sections[0], replace(sections[1], required=False)]),
        ('optional-members', tuple_sections[:-1] + [replace(tuple_sections[-1], required=False)]),
        ('future-required', sections + [Section(60000, 99, True, b'future')]),
        ('future-required-member', sections + [Section(4, 99, True, b'future')]),
        ('future-optional-references', [replace(sections[0], version=99, required=False), sections[1]]),
        ('future-optional-signature', [sections[0], replace(sections[1], version=99, required=False)]),
        ('bad-reference-payload', [replace(sections[0], payload=b'x'), sections[1]]),
        ('bad-signature-payload', [sections[0], replace(sections[1], payload=b'x')]),
        ('bad-member-payload', tuple_sections[:-1] + [replace(tuple_sections[-1], payload=b'x')]),
        ('wrong-member-owner-shape', tuple_sections[:-1] + [Section(4, 1, True, encode_members([MemberRef('function_invoke')]))]),
        ('tuple-ordinal', tuple_sections[:-1] + [Section(4, 1, True, encode_members([MemberRef('tuple_element', element=2)]))]),
        ('unbound-nominal', [Section(2, 1, True, encode_bindings(Bindings(()))),
                             Section(3, 1, True, encode_signature(Node('nominal', index=1), references=True))]),
    ]:
        check(name, image(parts), True)
    _, context, bindings = read_profile(sections)
    for name, changed in [('missing-type-owner', replace(bindings, type_owner=0)),
                          ('missing-method-owner', replace(bindings, method_owner=0)),
                          ('missing-self-owner', replace(bindings, self_owner=0))]:
        check(name, image([replace(sections[0], payload=encode_bindings(changed)), sections[1]]), True)
    check('nominal-method-token', image([sections[0], replace(sections[1], payload=encode_signature(
        Node('nominal', index=3), context, references=True))]), True)
    for size in (0, 15, 16, 31, len(tuple_image)-1):
        check('truncation-' + str(size), tuple_image[:size], True)
    check('trailing-byte', tuple_image + b'x', True)

    out = OUTPUT / 'dotnet-library'
    print(command('dotnet', 'build', ROOT / 'tools/metadata/MetadataConformance/MetadataConformance.csproj',
                  '-o', out, '--nologo', '-v', 'minimal'))
    path = out / 'profile-vectors.json'
    path.write_text(json.dumps(vectors))
    print(command('dotnet', out / 'MetadataConformance.dll', 'profile-vectors', path).strip())
    report = dict(date='2026-09-30', scope='reference-profile document composition; no PE loading or compiler integration',
                  dotnet_sdk=command('dotnet', '--version').strip(), target_framework='net10.0',
                  shared_vectors=len(vectors), rejection_vectors=sum(v['reject'] for v in vectors),
                  independent_emission_creation_and_ownership_checks='passed',
                  cases=[dict(name=v['name'], rejected=v['reject']) for v in vectors])
    (HERE / 'dotnet-profiles-validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k: v for k, v in report.items() if k != 'cases'}, indent=2))


if __name__ == '__main__':
    main()

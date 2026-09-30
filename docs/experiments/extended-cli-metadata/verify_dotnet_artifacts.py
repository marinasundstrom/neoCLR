"""Bounded PE recognition, extraction and typed-profile validation on .NET."""
import json
import struct

from codec import FormatError, Section, decode, encode
from members import read_members
from references import read_profile
from pe_container import embed, extract, layout, streams, recognize, marked_prefix
from verify_recognition import replace_metadata
from verify_dotnet_references import b64
from verify_pe import HERE, OUTPUT, ROOT, PROBE, command, require


def main():
    out = OUTPUT / 'dotnet-artifacts'
    out.mkdir(parents=True, exist_ok=True)
    print(command('dotnet', 'build', HERE / 'pe-probe/Probe.csproj', '-o', OUTPUT / 'probe', '--nologo', '-v', 'minimal'))
    base_path = out / 'baseline.dll'
    command('dotnet', PROBE, 'generate', base_path)
    base = base_path.read_bytes()
    neo = (HERE / 'fixtures/tuple-members.neox').read_bytes()
    image = embed(base, neo, recognized=True)
    prefix, values = streams(layout(image)['metadata'])
    base_prefix, _ = streams(layout(base)['metadata'])
    vectors = []

    def case(name, data, expected=True, result='reject'):
        try:
            classification = recognize(data, expected_extended=expected)
            if classification == 'extended-neox-0.1':
                profile_bytes = extract(data)
                sections = decode(profile_bytes, {2: 1, 3: 1, 4: 1})
                profile = read_profile(sections)
                if profile is None:
                    raise FormatError('reference profile required')
                read_members(sections, profile)
                actual = 'extended'
            else:
                actual = 'ordinary'
        except FormatError:
            actual = 'reject'
        require(actual == result, name + ': unexpected Python result ' + actual)
        path = out / (name + '.dll')
        path.write_bytes(data)
        vector = dict(name=name, path=str(path), expected_extended=expected, result=result)
        if result == 'extended':
            vector['profile'] = b64(profile_bytes)
        vectors.append(vector)

    case('ordinary-opt-in', base, False, 'ordinary')
    case('ordinary-default', base)
    case('marked', image, result='extended')
    case('marked-auto', image, False, 'extended')
    case('unmarked', embed(base, neo))
    case('unmarked-auto', embed(base, neo), False)
    stripped = {k: v for k, v in values.items() if k != '#Neo'}
    case('stripped-stream', replace_metadata(image, prefix, stripped))
    erased = replace_metadata(image, base_prefix, stripped)
    case('all-marks-stripped', erased)
    case('erased-opt-in', erased, False, 'ordinary')
    case('reordered-streams', replace_metadata(image, prefix, dict(reversed(list(values.items())))), result='extended')
    for name in ('#Strings', '#Neo'):
        changed = dict(values)
        changed[name] = changed[name][:-1] + bytes([changed[name][-1] ^ 1])
        case('changed-' + name[1:], replace_metadata(image, prefix, changed))
    case('unsupported-marker', replace_metadata(image, prefix.replace(b'NEOX.0.1', b'NEOX.9.1'), values))
    digest_index = prefix.index(b'sha256=') + 7
    for name, replacement in [('bad-digest-character', b'g'), ('wrong-digest', b'0' if prefix[digest_index] != 48 else b'1')]:
        changed = prefix[:digest_index] + replacement + prefix[digest_index+1:]
        case(name, replace_metadata(image, changed, values))
    changed = bytearray(prefix)
    changed[-3] = 1  # nonzero after the terminating null
    case('bad-marker-padding', replace_metadata(image, bytes(changed), values))
    for name, payload in [('unknown-required', encode([Section(60000, 1, True, b'x')], {60000: 1})),
                          ('empty-profile', encode([], {})), ('bad-envelope', b'x' * 16)]:
        case(name, embed(base, payload, recognized=True))
    # Larger transport needs embedding, which also recomputes the digest.
    case('excess-neo-padding', embed(base, neo + b'\0' * 4, recognized=True))
    case('nonzero-neo-padding', embed(base, neo + b'\1', recognized=True))
    # A recomputed digest admits changed conventional metadata; it is not trust or CLI validation.
    changed = dict(values)
    changed['#Strings'] = b'\1' + changed['#Strings'][1:]
    case('rebound-conventional-data', replace_metadata(image, marked_prefix(prefix, changed), changed), result='extended')
    marked_path = out / 'marked.dll'
    rewritten_path = out / 'rewritten.dll'
    command('dotnet', PROBE, 'rewrite', marked_path, rewritten_path)
    case('cecil-lossy-rewrite', rewritten_path.read_bytes())

    info = layout(image)
    for name, offset, code, value in [
        ('pe-signature', info['pe'], 'I', 0), ('pe32-plus', info['optional'], 'H', 0x20b),
        ('section-count', info['pe'] + 6, 'H', 17), ('alignment', info['optional'] + 36, 'I', 3),
        ('certificate', info['optional'] + 96 + 32, 'I', 1), ('checksum', info['optional'] + 64, 'I', 1),
        ('cli-flags', info['cli'] + 16, 'I', 9), ('metadata-rva', info['cli'] + 8, 'I', 0xffffffff),
        ('metadata-length', info['cli'] + 12, 'I', 0xffffffff), ('pe-pointer-overflow', 0x3c, 'I', 0xffffffff),
        ('version-length', info['metadata_offset'] + 12, 'I', 0xffffffff),
        ('stream-count', info['metadata_offset'] + 16 + struct.unpack_from('<I', info['metadata'], 12)[0] + 2, 'H', 17),
        ('section-overlap', info['table'] + 40 + 20, 'I', info['sections'][0][2]),
    ]:
        changed = bytearray(image)
        struct.pack_into('<' + code, changed, offset, value)
        case(name, bytes(changed))
    # Mutation in the stream directory is checked before marker binding.
    directory = info['metadata_offset'] + 20 + struct.unpack_from('<I', info['metadata'], 12)[0]
    for name, offset, value in [('stream-offset', directory, 0), ('stream-size', directory + 4, 0xffffffff)]:
        changed = bytearray(image)
        struct.pack_into('<I', changed, offset, value)
        case(name, bytes(changed))
    headers = []
    cursor = directory
    for _ in values:
        end = image.index(b'\0', cursor + 8)
        headers.append((cursor, cursor + 8, end))
        cursor = cursor + 8 + ((end - cursor - 8 + 1 + 3) & ~3)
    for name, offset, value in [('empty-stream-name', headers[0][1], 0),
                                ('non-ascii-stream-name', headers[0][1], 255),
                                ('bad-name-padding', headers[0][2] + 1, 1)]:
        changed = bytearray(image)
        changed[offset] = value
        case(name, bytes(changed))
    changed = bytearray(image)
    struct.pack_into('<I', changed, headers[1][0], struct.unpack_from('<I', image, headers[0][0])[0])
    case('overlapping-streams', bytes(changed))
    changed = bytearray(image)
    changed[headers[0][1]:headers[0][1]+32] = b'A' * 32
    case('unterminated-stream-name', bytes(changed))
    for size in (0, 1, 63, info['pe'] + 3, info['table'] + 39, len(image)-1):
        case('truncated-' + str(size), image[:size])
    case('overlay', image + b'extra')

    print(command('dotnet', 'build', ROOT / 'tools/metadata/MetadataConformance/MetadataConformance.csproj',
                  '-o', OUTPUT / 'dotnet-library', '--nologo', '-v', 'minimal'))
    manifest = out / 'vectors.json'
    manifest.write_text(json.dumps(vectors))
    print(command('dotnet', OUTPUT / 'dotnet-library/MetadataConformance.dll', 'artifact-vectors', manifest).strip())
    report = dict(date='2026-09-30', scope='bounded unsigned IL-only PE32, consistency and reference-profile validation only',
                  dotnet_sdk=command('dotnet', '--version').strip(), target_framework='net10.0',
                  shared_cases=len(vectors), rejections=sum(v['result'] == 'reject' for v in vectors),
                  size_and_ownership_checks='passed', cases=[{k: v[k] for k in ('name', 'expected_extended', 'result')} for v in vectors])
    (HERE / 'dotnet-artifacts-validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k: v for k, v in report.items() if k != 'cases'}, indent=2))


if __name__ == '__main__':
    main()

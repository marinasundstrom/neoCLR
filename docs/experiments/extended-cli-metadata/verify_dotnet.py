"""Cross-language framing conformance for the experimental .NET host library."""
import hashlib
import json
import platform
import struct

from codec import FormatError, Section, decode, encode
from verify_pe import HERE, OUTPUT, ROOT, command, require


def main():
    destination = OUTPUT / 'dotnet-library'
    project = ROOT / 'tools' / 'metadata' / 'MetadataConformance' / 'MetadataConformance.csproj'
    print(command('dotnet', 'build', project, '-o', destination, '--nologo', '-v', 'minimal'))
    consumer = destination / 'MetadataConformance.dll'
    print(command('dotnet', consumer, 'selftest').strip())
    schemas = {1: 1, 2: 1, 3: 1, 4: 1, 7: 2}
    hashes = {}
    for fixture in sorted((HERE / 'fixtures').glob('*.neox')):
        output = destination / fixture.name
        command('dotnet', consumer, 'roundtrip', fixture, output)
        require(output.read_bytes() == fixture.read_bytes(), 'cross-reader framing roundtrip changed fixture')
        require(encode(decode(output.read_bytes(), schemas), schemas) == fixture.read_bytes(),
                'Python could not roundtrip .NET output')
        hashes[fixture.name] = hashlib.sha256(output.read_bytes()).hexdigest()
    emitted = destination / 'dotnet-emitted.neox'
    command('dotnet', consumer, 'emit', emitted)
    expected = [Section(7, 2, True, b'abc'), Section(60000, 9, False, b'\0\xff\0')]
    require(decode(emitted.read_bytes(), schemas) == expected, 'Python read of independent .NET emission differs')
    require(encode(expected, schemas) == emitted.read_bytes(), 'independent writers disagree')
    hashes[emitted.name] = hashlib.sha256(emitted.read_bytes()).hexdigest()

    # Shared malformed byte vectors exercise independent readers, not code mirrored from .NET.
    golden = bytes.fromhex('4e454f58000001000100000023000000'
                           '07000100000000002000000003000000616263')
    cases = {f'truncated-{i}': golden[:i] for i in range(len(golden))}
    for name, offset, code, value in [
        ('major-version', 4, 'H', 1), ('minor-version', 6, 'H', 2),
        ('section-count', 8, 'I', 65), ('total-length', 12, 'I', 0),
        ('zero-kind', 16, 'H', 0), ('zero-schema', 18, 'H', 0),
        ('unknown-flags', 20, 'I', 2), ('offset-header', 24, 'I', 0),
        ('offset-overflow', 24, 'I', 0xffffffff), ('length-overflow', 28, 'I', 0xffffffff),
    ]:
        changed = bytearray(golden)
        struct.pack_into('<' + code, changed, offset, value)
        cases[name] = bytes(changed)
    cases['trailing-data'] = golden + b'x'
    cases['wrong-magic'] = b'NOPE' + golden[4:]
    cases['unknown-required'] = encode([Section(60000, 1, True, b'x')], {60000: 1})
    duplicate = bytearray(encode([Section(7, 1, False, b'x'), Section(8, 1, False, b'y')]))
    struct.pack_into('<H', duplicate, 32, 7)
    cases['duplicate-kind'] = bytes(duplicate)
    rejected = []
    path = destination / 'malformed.neox'
    for name, data in cases.items():
        try:
            decode(data, schemas)
        except FormatError:
            pass
        else:
            raise RuntimeError('Python accepted ' + name)
        path.write_bytes(data)
        require(command('dotnet', consumer, 'reject', path).strip() == 'rejected', '.NET accepted ' + name)
        rejected.append(name)

    report = {
        'date': '2026-09-30', 'scope': 'NEOX framing only; payload schemas remain opaque in .NET',
        'python': platform.python_version(), 'dotnet_sdk': command('dotnet', '--version').strip(),
        'target_framework': 'net10.0',
        'python_fixture_dotnet_roundtrips': len(hashes) - 1,
        'independent_dotnet_emission_read_by_python': True,
        'writer_and_owned_buffer_checks': 'passed',
        'malformed_vectors_rejected_by_both': rejected,
        'artifact_sha256': hashes,
    }
    (HERE / 'dotnet-validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()

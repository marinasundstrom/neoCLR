"""Build and qualify only the experimental PE container/reader compatibility probe."""
import hashlib
import json
from pathlib import Path
import platform
import struct
import subprocess
import sys

from codec import FormatError, Section, encode
from pe_container import embed, extract, layout, streams

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
OUTPUT = ROOT / 'target' / 'extended-cli-metadata'
PROBE = OUTPUT / 'probe' / 'Probe.dll'


def command(*args):
    return subprocess.run([str(a) for a in args], cwd=ROOT, check=True,
                          capture_output=True, text=True).stdout


def reader(path):
    return json.loads(command('dotnet', PROBE, 'read', path))


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def reject(action):
    try:
        action()
    except FormatError:
        return
    raise RuntimeError('malformed or unsupported image was accepted')


def main():
    OUTPUT.mkdir(parents=True, exist_ok=True)
    print(command('dotnet', 'build', HERE / 'pe-probe' / 'Probe.csproj',
                  '-o', OUTPUT / 'probe', '--nologo', '-v', 'minimal'))
    baseline_path = OUTPUT / 'baseline.dll'
    command('dotnet', PROBE, 'generate', baseline_path)
    baseline = baseline_path.read_bytes()
    extension = (HERE / 'fixtures' / 'tuple-members.neox').read_bytes()
    extended = embed(baseline, extension)
    extended_path = OUTPUT / 'extended.dll'
    extended_path.write_bytes(extended)
    before, after = reader(baseline_path), reader(extended_path)
    require(before == after, 'ordinary reader metadata or method bodies changed')
    _, original_streams = streams(layout(baseline)['metadata'])
    _, extended_streams = streams(layout(extended)['metadata'])
    require(set(extended_streams) == set(original_streams) | {'#Neo'}, 'stream set changed')
    require(all(extended_streams[name] == value for name, value in original_streams.items()),
            'conventional stream bytes changed')
    require(extract(extended) == extension, 'extension extraction changed payload')
    inspection = json.loads(command(sys.executable, HERE / 'codec.py', '--pe', extended_path))
    require(inspection['sections'][-1]['owner_shape_validated'], 'member inspection failed')

    # Required extensions are invisible to unaware readers, not an execution gate.
    unknown = encode([Section(60000, 1, True, b'unknown')], {60000: 1})
    unknown_path = OUTPUT / 'unknown-required.dll'
    unknown_path.write_bytes(embed(baseline, unknown))
    require(reader(unknown_path) == before, 'ordinary reader could not inspect unknown required data')
    result = subprocess.run([sys.executable, str(HERE / 'codec.py'), '--pe', str(unknown_path)],
                            capture_output=True, text=True)
    require(result.returncode == 1 and 'unsupported required section' in result.stderr and not result.stdout,
            'aware inspector did not reject unknown required semantics')

    rewritten_path = OUTPUT / 'rewritten.dll'
    command('dotnet', PROBE, 'rewrite', extended_path, rewritten_path)
    rewritten = rewritten_path.read_bytes()
    _, rewritten_streams = streams(layout(rewritten)['metadata'])
    require('#Neo' not in rewritten_streams, 'Cecil rewrite behavior changed; reassess preservation')
    reject(lambda: extract(rewritten))
    reader(rewritten_path)  # still ordinarily readable; not a semantic-equivalence claim

    info = layout(baseline)
    cases = {}
    for label, offset, code, value in [
        ('bad-pe-signature', info['pe'], 'I', 0),
        ('pe32-plus-unsupported', info['optional'], 'H', 0x20b),
        ('bad-section-count', info['pe'] + 6, 'H', 100),
        ('bad-alignment', info['optional'] + 36, 'I', 3),
        ('certificate-directory', info['optional'] + 96 + 4 * 8, 'I', 1),
        ('strong-name-flag', info['cli'] + 16, 'I', 9),
        ('metadata-rva-outside-image', info['cli'] + 8, 'I', 0xffffffff),
        ('metadata-size-outside-image', info['cli'] + 12, 'I', 0xffffffff),
    ]:
        data = bytearray(baseline)
        struct.pack_into('<' + code, data, offset, value)
        reject(lambda: embed(data, extension))
        cases[label] = 'rejected'
    reject(lambda: embed(baseline[:-1], extension))
    reject(lambda: embed(extended, extension))
    reject(lambda: embed(baseline + b'overlay', extension))
    busy = bytearray(baseline)
    busy[info['table'] + info['count'] * 40] = 1
    reject(lambda: embed(busy, extension))
    cases.update(truncated_image='rejected', duplicate_extension='rejected',
                 overlay='rejected', occupied_section_slot='rejected')

    metadata = layout(extended)['metadata']
    first_entry = 16 + struct.unpack_from('<I', metadata, 12)[0] + 4
    broken = bytearray(metadata)
    struct.pack_into('<I', broken, first_entry, 0)
    reject(lambda: streams(broken))
    second_entry = first_entry + 12  # #~ header has four-byte name field
    broken = bytearray(metadata)
    struct.pack_into('<I', broken, second_entry, struct.unpack_from('<I', metadata, first_entry)[0])
    reject(lambda: streams(broken))
    cases.update(stream_in_directory='rejected', overlapping_streams='rejected')
    bad_padding = bytearray(embed(baseline, unknown))
    bad_info = layout(bad_padding)
    # Last stream is #Neo, at the metadata root's end including stream padding.
    bad_padding[bad_info['metadata_offset'] + len(bad_info['metadata']) - 1] = 1
    reject(lambda: extract(bad_padding))
    cases['nonzero_extension_padding'] = 'rejected'

    report = {
        'date': '2026-09-30', 'scope': 'PE32 fixture metadata inspection, no execution',
        'python': platform.python_version(), 'platform': platform.platform(),
        'dotnet_sdk': command('dotnet', '--version').strip(),
        'dotnet_runtime': before['Runtime'], 'srm_version': before['SrmVersion'],
        'cecil_version': before['CecilVersion'],
        'ordinary_reader_snapshots_equal': True,
        'conventional_streams_byte_identical': list(original_streams),
        'extension_roundtrip': True, 'aware_member_inspection': True,
        'unknown_required_ordinary_readers': 'accepted for inspection',
        'unknown_required_aware_inspector': 'rejected',
        'cecil_rewrite': '#Neo stripped; rewritten image must not be accepted as preserving neoCLR semantics',
        'negative_cases': cases,
        'artifacts': {name: hashlib.sha256((OUTPUT / name).read_bytes()).hexdigest()
                      for name in ('baseline.dll', 'extended.dll', 'unknown-required.dll', 'rewritten.dll')},
    }
    (HERE / 'pe-validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()

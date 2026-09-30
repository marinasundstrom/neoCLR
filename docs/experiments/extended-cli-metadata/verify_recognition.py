"""Qualify marked metadata recognition separately from raw PE transport."""
import hashlib
import json
import struct
import subprocess
import sys

from codec import FormatError, Section, encode
from pe_container import (build_metadata, embed, layout, marked_prefix, metadata_digest,
                          recognize, streams)
from verify_pe import HERE, OUTPUT, PROBE, command, reader, require


def replace_metadata(image, prefix, values):
    info = layout(image)
    metadata = build_metadata(prefix, values)
    require(len(metadata) <= len(info['metadata']), 'test mutation exceeds existing metadata span')
    output = bytearray(image)
    start = info['metadata_offset']
    output[start:start + len(metadata)] = metadata
    struct.pack_into('<I', output, info['cli'] + 12, len(metadata))
    return bytes(output)


def main():
    out = OUTPUT / 'recognition'
    out.mkdir(parents=True, exist_ok=True)
    print(command('dotnet', 'build', HERE / 'pe-probe' / 'Probe.csproj',
                  '-o', OUTPUT / 'probe', '--nologo', '-v', 'minimal'))
    base_path = out / 'baseline.dll'
    command('dotnet', PROBE, 'generate', base_path)
    base = base_path.read_bytes()
    neo = (HERE / 'fixtures' / 'tuple-members.neox').read_bytes()
    raw = embed(base, neo)
    image = embed(base, neo, recognized=True)
    marked_path = out / 'marked.dll'
    marked_path.write_bytes(image)
    baseline_prefix, _ = streams(layout(base)['metadata'])
    prefix, values = streams(layout(image)['metadata'])
    require(recognize(base) == 'ordinary-cli', 'ordinary CLI misclassified')
    require(recognize(image, expected_extended=True) == 'extended-neox-0.1', 'marked artifact rejected')
    require(reader(base_path) == reader(marked_path), 'marker changed ordinary reader snapshot')
    inspected = json.loads(command(sys.executable, HERE / 'codec.py', '--recognized-pe', marked_path))
    require(inspected['sections'][-1]['owner_shape_validated'], 'recognized member inspection failed')
    require(metadata_digest(values) == metadata_digest(dict(reversed(list(values.items())))),
            'stream directory order leaked into binding')

    cases = {}

    def rejection(label, data, expected):
        path = out / (label + '.dll')
        path.write_bytes(data)
        try:
            recognize(data, expected_extended=True)
        except FormatError as error:
            require(expected in str(error), f'{label}: wrong error {error}')
        else:
            raise RuntimeError(label + ': unexpected acceptance')
        result = subprocess.run([sys.executable, str(HERE / 'codec.py'), '--recognized-pe', str(path)],
                                capture_output=True, text=True)
        require(result.returncode == 1 and expected in result.stderr and not result.stdout,
                label + ': inspector failed to reject cleanly')
        cases[label] = expected

    rejection('ordinary-when-extended-expected', base, 'marker and #Neo missing')
    rejection('unmarked-transport', raw, 'unmarked #Neo')
    stripped_values = {k: v for k, v in values.items() if k != '#Neo'}
    rejection('stripped-stream', replace_metadata(image, prefix, stripped_values), 'required #Neo stream missing')
    for name in ('#Strings', '#Neo'):
        altered = dict(values)
        data = bytearray(altered[name])
        data[-1] ^= 1
        altered[name] = bytes(data)
        rejection('changed-' + name[1:], replace_metadata(image, prefix, altered), 'metadata binding mismatch')
    rejection('unsupported-profile', replace_metadata(image, prefix.replace(b'NEOX.0.1', b'NEOX.9.1'), values),
              'unsupported extended artifact recognition version')
    forged = bytearray(prefix)
    first_digest = bytes(forged).index(b'sha256=') + len(b'sha256=')
    forged[first_digest] = ord('g')
    rejection('invalid-digest', replace_metadata(image, bytes(forged), values),
              'invalid metadata binding digest')
    forged = bytearray(prefix)
    forged[first_digest] = ord('0') if forged[first_digest] != ord('0') else ord('1')
    rejection('wrong-digest', replace_metadata(image, bytes(forged), values), 'metadata binding mismatch')
    no_marks = replace_metadata(image, baseline_prefix, stripped_values)
    rejection('both-markers-stripped', no_marks, 'marker and #Neo missing')
    require(recognize(no_marks) == 'ordinary-cli', 'classification must not claim erased provenance')

    rewritten_path = out / 'cecil-rewritten.dll'
    command('dotnet', PROBE, 'rewrite', marked_path, rewritten_path)
    rewritten = rewritten_path.read_bytes()
    rewritten_prefix, rewritten_values = streams(layout(rewritten)['metadata'])
    require(rewritten_prefix == prefix and '#Neo' not in rewritten_values,
            'Cecil recognition behavior changed; reassess contract')
    reader(rewritten_path)
    rejection('cecil-stripped-extension', rewritten, 'required #Neo stream missing')

    # Marker validation must not bypass envelope capability checks.
    unknown = encode([Section(60000, 1, True, b'unknown')], {60000: 1})
    unknown_path = out / 'unknown-required.dll'
    unknown_image = embed(base, unknown, recognized=True)
    unknown_path.write_bytes(unknown_image)
    require(recognize(unknown_image) == 'extended-neox-0.1', 'valid binding rejected')
    result = subprocess.run([sys.executable, str(HERE / 'codec.py'), '--recognized-pe', str(unknown_path)],
                            capture_output=True, text=True)
    require(result.returncode == 1 and 'unsupported required section' in result.stderr and not result.stdout,
            'binding bypassed semantic capability rejection')
    cases['unknown-required-feature'] = 'unsupported required section'

    # An aware writer can bind newly emitted metadata; digest alone is not trust.
    altered = dict(values)
    altered['#Strings'] = bytes([values['#Strings'][0] ^ 1]) + values['#Strings'][1:]
    rebound = replace_metadata(image, marked_prefix(prefix, altered), altered)
    require(recognize(rebound) == 'extended-neox-0.1', 'consistent rebound metadata rejected')

    report = {
        'date': '2026-09-30', 'scope': 'experimental metadata consistency, no authentication or execution',
        'dotnet_sdk': command('dotnet', '--version').strip(),
        'reader_versions': {k: v for k, v in reader(marked_path).items() if k not in ('Srm', 'Cecil')},
        'ordinary_reader_snapshots_equal': True,
        'cecil_preserves_marker_but_strips_neo': True,
        'explicit_expected_profile_rejects_all_markers_removed': True,
        'automatic_classification_cannot_recover_erased_provenance': True,
        'recomputed_digest_is_not_authentication_or_metadata_validation': True,
        'rejections': cases,
        'artifacts': {name: hashlib.sha256((out / name).read_bytes()).hexdigest()
                      for name in ('baseline.dll', 'marked.dll', 'cecil-rewritten.dll', 'unknown-required.dll')},
    }
    (HERE / 'recognition-validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()

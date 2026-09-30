"""Real CLI declarations beneath a minimal Cecil-inspired read-only model."""
import json

from pe_container import embed, layout, streams, marked_prefix, recognize
from verify_recognition import replace_metadata
from verify_pe import HERE, ROOT, OUTPUT, PROBE, command, reader, require


def main():
    out = OUTPUT / 'dotnet-model'
    out.mkdir(parents=True, exist_ok=True)
    print(command('dotnet', 'build', HERE / 'pe-probe/Probe.csproj', '-o', OUTPUT / 'probe', '--nologo', '-v', 'minimal'))
    base_path = out / 'ordinary.dll'
    command('dotnet', PROBE, 'generate-model', base_path)
    base = base_path.read_bytes()
    neo = (HERE / 'fixtures/tuple-members.neox').read_bytes()
    image = embed(base, neo, recognized=True)
    marked_path = out / 'marked.dll'
    marked_path.write_bytes(image)
    require(reader(base_path) == reader(marked_path), 'SRM/Cecil conventional metadata changed')
    # Consistent marker around corrupt CLI tables: artifact inspection accepts,
    # but the new declaration reader must reject (digest is not verification).
    prefix, values = streams(layout(image)['metadata'])
    changed = dict(values)
    key = '#~' if '#~' in values else '#-'
    changed[key] = b'\xff' * len(changed[key])
    invalid = replace_metadata(image, marked_prefix(prefix, changed), changed)
    require(recognize(invalid) == 'extended-neox-0.1', 'bad-table artifact should pass consistency only')
    invalid_path = out / 'invalid-tables.dll'
    invalid_path.write_bytes(invalid)
    excessive_path = out / 'excessive-names.dll'
    command('dotnet', PROBE, 'generate-model-large-names', excessive_path)
    require(len(excessive_path.read_bytes()) < 4 * 1024 * 1024, 'name-budget fixture exceeds container cap')
    print(command('dotnet', 'build', ROOT / 'tools/metadata/MetadataConformance/MetadataConformance.csproj',
                  '-o', OUTPUT / 'dotnet-library', '--nologo', '-v', 'minimal'))
    print(command('dotnet', OUTPUT / 'dotnet-library/MetadataConformance.dll', 'assembly-model',
                  base_path, marked_path, invalid_path, excessive_path).strip())
    report = dict(date='2026-09-30', scope='read-only manifest module and TypeDef model; no dependency binding, member signatures or assembly emission',
                  dotnet_sdk=command('dotnet', '--version').strip(), target_framework='net10.0',
                  srm_cecil_declarations_preserved=True, checks=['assembly/module fields', 'Unicode names', 'generic arity',
                  'nested declaring type', 'module pseudo-type', 'snapshot-scoped references', 'missing/wrong-kind lookup',
                  'owned data', 'decoded name amplification limit', 'ordinary input opt-in', 'invalid conventional tables rejected despite valid digest'])
    (HERE / 'dotnet-model-validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()

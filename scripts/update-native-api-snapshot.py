#!/usr/bin/env python3
"""Pin a reviewed native documentation audit for clean-checkout website builds."""
import argparse
import hashlib
import json
from pathlib import Path
import zipfile
from native_api_snapshot import validate_source_links

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('audit', type=Path)
    parser.add_argument('--renderer-revision', required=True)
    args = parser.parse_args()
    source = args.audit / 'site/docs/api'
    evidence_path = args.audit / 'inputs/build-evidence.json'
    if evidence_path.is_file():
        build = json.loads(evidence_path.read_text())
    else:
        # A verified HTTP-only rebuild preserves the other bundled libraries.
        # It records exact source/compiler hashes rather than inventing git IDs.
        evidence_path = args.audit / 'inputs/http-foundation-build.json'
        evidence = json.loads(evidence_path.read_text())
        if evidence['exitCode'] != 0:
            raise ValueError('HTTP library build did not pass')
        source_hash = hashlib.sha256(json.dumps(evidence['sources'], sort_keys=True).encode()).hexdigest()
        build = dict(sourceRevision='http-source-sha256:' + source_hash,
                     compilerRevision=evidence.get('compilerRevision') or 'binary-sha256:' + evidence['compilerSha256'])
    bundle = json.loads((args.audit / 'inputs/bundle.json').read_text())
    for relative in ('System/Object/index.html', 'System/Math/field_Pi.html',
                     'System/Linq/Operators/index.html', 'api-navigation.html'):
        if not (source / relative).is_file():
            raise ValueError('Incomplete native reference: ' + relative)
    repository = json.loads((ROOT / 'website/site.json').read_text())['sourceRepository']
    validate_source_links(source, repository)
    archive = ROOT / 'api-docs/native-reference.zip'
    with zipfile.ZipFile(archive, 'w', compression=zipfile.ZIP_DEFLATED) as output:
        for path in sorted(source.rglob('*')):
            if path.is_file():
                info = zipfile.ZipInfo('site/docs/api/' + path.relative_to(source).as_posix())
                info.compress_type = zipfile.ZIP_DEFLATED
                info.external_attr = 0o100644 << 16
                output.writestr(info, path.read_bytes())
    manifest = dict(format=1, archive=archive.name,
                    sourceRepository={key: repository[key] for key in ('url', 'revision')},
                    sha256=hashlib.sha256(archive.read_bytes()).hexdigest(),
                    librarySourceRevision=build['sourceRevision'],
                    compilerRevision=build['compilerRevision'],
                    libraryBuildEvidence=dict(file=evidence_path.name, sha256=hashlib.sha256(evidence_path.read_bytes()).hexdigest()),
                    rendererRevision=args.renderer_revision,
                    libraryArtifacts={name: digest for name, digest in bundle['files'].items()
                                      if name.endswith(('.dll', '.neox'))},
                    renderingEvidence={name: hashlib.sha256((args.audit / name).read_bytes()).hexdigest()
                                       for name in ('normalized-sidecars.json', 'restored-xml-comments.json')},
                    scope='Reviewed native API rendering; legacy coverage gaps remain explicitly marked by the website overlay.')
    (ROOT / 'api-docs/native-reference.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(archive)


if __name__ == '__main__':
    main()

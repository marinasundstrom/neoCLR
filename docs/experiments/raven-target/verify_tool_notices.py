"""Check toolchain dependency coverage and preserved notice bytes before distribution."""
import argparse
import hashlib
import json
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[3])
parser.add_argument('--tools', type=Path, required=True, help='Extracted SDK or runtime bundle')
args = parser.parse_args()
root = args.root.resolve()
manifest = json.loads((root / 'third-party/raven-tools/manifest.json').read_text())
known = {p['package'] for p in manifest['packages'] if p['kind'] == 'nuget'}
seen = set()
for path in args.tools.rglob('*.deps.json'):
    for name, info in json.loads(path.read_text()).get('libraries', {}).items():
        if info.get('type') == 'package':
            seen.add(name)
if not seen:
    raise SystemExit('No package dependency manifests found')
if seen - known:
    raise SystemExit('Unreviewed dependencies: ' + ', '.join(sorted(seen - known)))
for package in manifest['packages']:
    for notice in package['notices']:
        path = (root / notice['path']).resolve()
        if root not in path.parents or hashlib.sha256(path.read_bytes()).hexdigest() != notice['sha256']:
            raise SystemExit('Invalid notice: ' + notice['path'])
upstream_root = root / 'third-party/raven-tools/upstream/raven'
upstream = json.loads((upstream_root / 'source.json').read_text())
for name, expected in upstream['files'].items():
    path = (upstream_root / name).resolve()
    if upstream_root not in path.parents or hashlib.sha256(path.read_bytes()).hexdigest() != expected:
        raise SystemExit('Invalid Raven upstream notice: ' + name)
bundle_manifest = args.tools / 'manifest.json'
if bundle_manifest.is_file():
    bundle = json.loads(bundle_manifest.read_text())
    if bundle.get('format') == 'neoclr-native-poc-v1':
        for name in ('source.json', *upstream['files']):
            relative = 'third-party/raven-tools/upstream/raven/' + name
            expected = hashlib.sha256((upstream_root / name).read_bytes()).hexdigest()
            path = args.tools / relative
            if bundle['files'].get(relative) != expected or not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest() != expected:
                raise SystemExit('Missing or mismatched bundled Raven upstream notice: ' + relative)
print(f'{len(seen)} NuGet dependencies covered; all {len(manifest["packages"])} package notice sets and Raven upstream notices verified')

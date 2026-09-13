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
print(f'{len(seen)} NuGet dependencies covered; all {len(manifest["packages"])} package notice sets verified')

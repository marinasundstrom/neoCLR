"""Build the bounded Raven-authored System.Math implementation and its bootstrap snapshots."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]
SOURCE = ROOT / 'runtime/raven/src/Math.rvn'
PROJECT = ROOT / 'runtime/raven/System.rvnproj'
GENERATED = ROOT / 'runtime/raven/generated'

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def fragments(text):
    lines = text.splitlines(keepends=True)
    methods, helpers = [], {}
    while lines:
        if not lines[0].strip():
            lines.pop(0)
            continue
        match = re.match(r'\.function ([^(]+)\(', lines[0])
        assert match, lines[0]
        end = lines.index('.end\n')
        body = ''.join(lines[:end + 1])
        if match[1].startswith('System.Math.'):
            # Retain the bootstrap owner used by direct IL and the archived Neo frontend.
            # Raven consumers see namespace functions through marked CLI metadata.
            methods.append(body.replace('.function System.Math.', '.method static ', 1))
        else:
            assert match[1] not in helpers
            helpers[match[1]] = body
        lines = lines[end + 1:]
    # Retain only transitively called adapters; no application entry-point shim.
    used = set()
    pending = re.findall(r'(?m)^(?:call|ldftn) ([^(]+)\(', ''.join(methods))
    while pending:
        name = pending.pop()
        if name in used or name not in helpers:
            continue
        used.add(name)
        pending.extend(re.findall(r'(?m)^(?:call|ldftn) ([^(]+)\(', helpers[name]))
    banner = '; Generated from runtime/raven/src/Math.rvn. Regenerate with build_runtime_library.py.\n'
    return {'Math.methods.neoil': banner + ''.join(methods),
            'Math.helpers.neoil': banner + ''.join(body for name, body in helpers.items() if name in used)}

def check_snapshot():
    data = json.loads((GENERATED / 'Math.json').read_text())
    for path, expected in data['inputs'].items():
        if digest(ROOT / path) != expected:
            raise SystemExit('Stale Raven library input: ' + path)
    for path, expected in data['outputs'].items():
        if digest(GENERATED / path) != expected:
            raise SystemExit('Modified generated library: ' + path)
    print('Raven library source and bootstrap snapshot hashes match')

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path)
    parser.add_argument('--bridge', type=Path)
    parser.add_argument('--check', action='store_true', help='Regenerate and compare without modifying the snapshot')
    parser.add_argument('--check-snapshot', action='store_true', help='Check source/artifact hashes without an SDK')
    args = parser.parse_args()
    if args.check_snapshot:
        check_snapshot()
        return
    if not args.compiler or not args.bridge:
        parser.error('--compiler and --bridge are required for regeneration')
    with tempfile.TemporaryDirectory(prefix='neoclr-runtime-library-') as temporary:
        root = Path(temporary)
        (root / 'demo').mkdir()
        core = root / 'demo/NeoCLR.CoreProbe.dll'
        bridge = ['dotnet', str(args.bridge.resolve())]
        subprocess.run([*bridge, '--reference-core', str(core)], check=True)
        subprocess.run(['dotnet', str(args.compiler.resolve()), str(PROJECT), '--no-project-restore',
                        '-o', str(root / 'compiled')], env={**os.environ, 'NeoCLRBootstrapRoot': str(root)}, check=True)
        subprocess.run([*bridge, '--library-implementation', str(root / 'compiled/NeoCLR.System.dll'),
                        str(core), 'System.Math', str(root / 'imported')], check=True)
        outputs = fragments((root / 'imported/Implementation.neoil').read_text())
        if args.check:
            check_snapshot()
            for name, text in outputs.items():
                if (GENERATED / name).read_text() != text:
                    raise SystemExit('Regenerated library differs: ' + name)
            print('Clean bootstrap regeneration matches checked-in implementation')
            return
        GENERATED.mkdir(exist_ok=True)
        data = {'format': 'raven-library-bootstrap-v1', 'owner': 'System.Math',
                'inputs': {str(p.relative_to(ROOT)): digest(p) for p in [SOURCE, PROJECT]},
                'outputs': {name: hashlib.sha256(text.encode()).hexdigest() for name, text in outputs.items()},
                'compilerSha256': digest(args.compiler), 'coreSha256': digest(core),
                'exports': re.findall(r'(?m)^\.method static (.+)', outputs['Math.methods.neoil'])}
        for name, text in outputs.items():
            (GENERATED / name).write_text(text)
        (GENERATED / 'Math.json').write_text(json.dumps(data, indent=2) + '\n')
        print('Generated System.Math bootstrap fragments')

if __name__ == '__main__':
    main()

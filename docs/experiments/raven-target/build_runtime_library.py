"""Build bounded Raven-authored runtime implementations and its bootstrap snapshots."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]
SLICES = {"Math": "System.Math", "Linq": "System.Linq.Operators", "Int32": "System.Int32", "Char": "System.Char", "ArrayList": "System.Collections.ArrayList", "HashMap": "System.Collections.HashMap", "Time": "System.Time"}
PROJECT = ROOT / 'runtime/raven/System.rvnproj'
GENERATED = ROOT / 'runtime/raven/generated'

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def fragments(text, name="Math", owner="System.Math"):
    lines = text.splitlines(keepends=True)
    methods, helpers, types = [], {}, []
    while lines:
        if not lines[0].strip():
            lines.pop(0)
            continue
        if lines[0].startswith('.type '):
            depth = 0
            for index, line in enumerate(lines):
                token = line.strip().split(' ', 1)[0]
                if token in ('.type', '.method', '.property'):
                    depth += 1
                elif token == '.end':
                    depth -= 1
                    if depth == 0:
                        break
            else:
                raise ValueError('Unclosed private implementation type')
            body = ''.join(lines[:index + 1])
            if lines[0].startswith('.type class ' + owner + '<') or lines[0].strip() == '.type ' + owner:
                methods.append(body)
            else:
                types.append(body)
            lines = lines[index + 1:]
            continue
        match = re.match(r'\.function ([^(]+)\(', lines[0])
        assert match, lines[0]
        end = lines.index('.end\n')
        body = ''.join(lines[:end + 1])
        if match[1].startswith(owner + '.'):
            # Retain the bootstrap owner used by direct IL and the archived Neo frontend.
            # Namespace functions use marked containers; static APIs retain their owner.
            methods.append(body.replace('.function ' + owner + '.', '.method static ', 1))
        else:
            assert match[1] not in helpers
            helpers[match[1]] = body
        lines = lines[end + 1:]
    # Retain only transitively called adapters; no application entry-point shim.
    used = set()
    pending = re.findall(r'(?m)^(?:call|ldftn) ([^(]+)\(', ''.join(methods + types))
    while pending:
        helper = pending.pop()
        if helper in used or helper not in helpers:
            continue
        used.add(helper)
        pending.extend(re.findall(r'(?m)^(?:call|ldftn) ([^(]+)\(', helpers[helper]))
    banner = f'; Generated from runtime/raven/src/{name}.rvn. Regenerate with build_runtime_library.py.\n'
    return {name + '.methods.neoil': banner + ''.join(methods),
            name + '.helpers.neoil': banner + ''.join(body for name, body in helpers.items() if name in used) + ''.join(types)}

def check_snapshot():
    for name in SLICES:
        data = json.loads((GENERATED / (name + '.json')).read_text())
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
        subprocess.run([*bridge, '--reference-library-core', str(core)], check=True)
        subprocess.run(['dotnet', str(args.compiler.resolve()), str(PROJECT), '--no-project-restore',
                        '-o', str(root / 'compiled')], env={**os.environ, 'NeoCLRBootstrapRoot': str(root), 'NeoCLRLibrarySlice': ''}, check=True)
        if args.check:
            check_snapshot()
        generated = {}
        for name, owner in SLICES.items():
            compiled = root / 'compiled'
            if name in ('ArrayList', 'HashMap', 'Time'):
                compiled = root / ('compiled-' + name)
                subprocess.run(['dotnet', str(args.compiler.resolve()), str(PROJECT), '--no-project-restore',
                                '-o', str(compiled)], env={**os.environ, 'NeoCLRBootstrapRoot': str(root),
                                'NeoCLRLibrarySlice': name}, check=True)
            imported = root / ('imported-' + name)
            subprocess.run([*bridge, '--library-implementation', str(compiled / 'NeoCLR.System.dll'),
                            str(core), owner, str(imported)], check=True)
            outputs = fragments((imported / 'Implementation.neoil').read_text(), name, owner)
            if args.check:
                for output, text in outputs.items():
                    if (GENERATED / output).read_text() != text:
                        raise SystemExit('Regenerated library differs: ' + output)
                continue
            inputs = [ROOT / ('runtime/raven/src/' + source + '.rvn') for source in SLICES]
            inputs += [PROJECT, ROOT / 'build/NeoCLR.Raven.props']
            data = {'format': 'raven-library-bootstrap-v1', 'owner': owner,
                    'inputs': {str(p.relative_to(ROOT)): digest(p) for p in inputs},
                    'outputs': {output: hashlib.sha256(text.encode()).hexdigest() for output, text in outputs.items()},
                    'compilerSha256': digest(args.compiler), 'coreSha256': digest(core),
                    'exports': re.findall(r'(?m)^\.method (?:static |instance )(.+)', outputs[name + '.methods.neoil'])}
            generated.update(outputs)
            generated[name + '.json'] = json.dumps(data, indent=2) + '\n'
        # Do not publish a partial snapshot when a later implementation fails admission.
        GENERATED.mkdir(exist_ok=True)
        for output, text in generated.items():
            (GENERATED / output).write_text(text)
        print('Clean bootstrap regeneration matches checked-in implementation' if args.check else 'Generated Raven library bootstrap fragments')

if __name__ == '__main__':
    main()

"""Assign RuntimeTypeHandle to the combined source library, then execute native consumers."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'ownership', 'seed-source', 'core', 'runtime', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    manifest = json.loads(args.ownership.read_text())
    if len(manifest['libraries']) != 1:
        raise ValueError('Expected the combined single-library ownership manifest')
    library = manifest['libraries'][0]
    source = 'runtime/raven/src/System/RuntimeTypeHandle.rvn'
    library['sources'] = sorted(set(library['sources']) | {source})
    library['types'] = sorted(set(library['types']) | {'System.RuntimeTypeHandle'})
    manifest.setdefault('nativePrimitives', {})['System.RuntimeTypeHandle'] = library['assemblyName']
    ownership = output / 'ownership.json'
    ownership.write_text(json.dumps(manifest, indent=2) + '\n')
    seed_text = args.seed_source.read_text()
    declaration = '.type System.RuntimeTypeHandle\n.sealed\n.end\n'
    if seed_text.count(declaration) != 1:
        raise ValueError('Expected exactly one retained empty handle declaration')
    seed_source = output / 'System.neoil'
    seed_source.write_text(seed_text.replace(declaration, ''))
    seed = output / 'System.neox'
    artifact = output / (library['assemblyName'] + '.dll')
    commands = []

    def run(command):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True, timeout=600)
        commands.append(dict(command=list(map(str, command)), exitCode=result.returncode,
                             stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode:
            raise RuntimeError(result.stdout + result.stderr)

    run([args.runtime.resolve(), 'assemble', seed_source, seed, '--format', 'neox'])
    run(['dotnet', args.compiler.resolve(), 'neoclr', '--core-reference', args.core.resolve(),
         '--runtime-seed', seed, '--bootstrap-intrinsics', '--bootstrap-ownership', ownership,
         '--library', '-o', artifact] + library['sources'])
    run([sys.executable, Path(__file__).with_name('verify_cumulative_json.py'),
         '--compiler', args.compiler.resolve(), '--library', artifact, '--ownership', ownership,
         '--seed', seed, '--core', args.core.resolve(), '--runtime', args.runtime.resolve(),
         '--tasks', '--output', output / 'consumers'])
    inputs = [ROOT / s for s in library['sources']] + [ownership, seed_source, seed, artifact,
        args.compiler.resolve(), args.core.resolve(), args.runtime.resolve(), Path(__file__)]
    revision = lambda path: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=path, text=True).strip()
    evidence = dict(runtimeBase=revision(ROOT), compilerBase=revision(args.compiler.resolve().parent),
        commands=commands, ownership=manifest,
        hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs})
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()

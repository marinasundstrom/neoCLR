"""Compile the Object service adapters and a test facade; execute a separate native consumer."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'library', 'ownership', 'seed', 'core', 'runtime', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    compiler, library, ownership, seed, core, runtime, output = (
        getattr(args, name).resolve() for name in
        ('compiler', 'library', 'ownership', 'seed', 'core', 'runtime', 'output'))
    output.mkdir(parents=True, exist_ok=False)
    sources = [ROOT / path for path in (
        'runtime/raven/native/RuntimeObjectCalls.rvn',
        'runtime/raven/native/RuntimeObjectServices.rvn',
        'docs/experiments/extended-cli-metadata/bootstrap/object-services-library.rvn')]
    consumer = ROOT / 'docs/experiments/extended-cli-metadata/bootstrap/object-services-consumer.rvn'
    support, app = output / 'ObjectServices.dll', output / 'App.dll'
    common = ['dotnet', compiler, 'neoclr', '--core-reference', core, '--runtime-seed', seed,
              '--bootstrap-intrinsics', '--bootstrap-ownership', ownership, '--reference', library]
    commands = []

    def run(command, expected=None):
        command = list(map(str, command))
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=180)
        commands.append(dict(command=command, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode or expected is not None and result.stdout != expected:
            raise RuntimeError(result.stdout + result.stderr)

    run(common + ['--library', '-o', support] + sources)
    run(common + ['--reference', support, '-o', app, consumer])
    dependencies = ['--system', seed, '--module', library, '--module', support]
    run([runtime, 'verify', app] + dependencies)
    run([runtime, 'run', app, '--instructions', '100000000'] + dependencies, 'Object service checks passed\n')
    paths = sources + [consumer, support, app, compiler, library, ownership, seed, core, runtime, Path(__file__)]
    paths += [compiler.parent / name for name in
              ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    revision = lambda path: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=path, text=True).strip()
    evidence = dict(runtimeBase=revision(ROOT), compilerRevision=revision(compiler.parent), commands=commands,
                    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths})
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()

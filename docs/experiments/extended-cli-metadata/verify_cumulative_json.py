"""Execute artifact-only JSON (and optional Tasks) consumers against a cumulative library."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
BOOTSTRAP = Path(__file__).resolve().parent / 'bootstrap'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'library', 'ownership', 'seed', 'core', 'runtime', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--tasks', action='store_true', help='Also execute the existing Tasks/Concurrency consumer.')
    args = parser.parse_args()
    compiler, library, ownership, seed, core, runtime, output = (
        getattr(args, name).resolve() for name in
        ('compiler', 'library', 'ownership', 'seed', 'core', 'runtime', 'output'))
    output.mkdir(parents=True, exist_ok=False)
    common = ['dotnet', compiler, 'neoclr', '--core-reference', core, '--runtime-seed', seed,
              '--bootstrap-intrinsics', '--bootstrap-ownership', ownership, '--reference', library]
    cases = [
        ('NativeMapping', [BOOTSTRAP / 'json-object-consumer.rvn'], 42,
         'Model constructed\nName assigned\nModel constructed\nInvalid input begins\nInvalid input ends\nNative JSON object mapping passed\n'),
        ('ExistingMapping', [ROOT / 'docs/experiments/json-object-mapping' / name
                             for name in ('Mapping.rvn', 'Main.rvn')], 0,
         'JSON object mapping checks passed\n'),
    ]
    if args.tasks:
        cases.append(('Tasks', [BOOTSTRAP / 'tasks-consumer.rvn'], 42, ''))
    commands = []

    def run(command, expected=0, stdout=None):
        command = [str(part) for part in command]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=180)
        commands.append(dict(command=command, exitCode=result.returncode,
                             stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != expected or stdout is not None and result.stdout != stdout:
            raise RuntimeError(json.dumps(commands[-1], indent=2))

    for name, sources, status, expected in cases:
        app = output / (name + '.dll')
        run(common + ['-o', app] + sources)
        run([runtime, 'verify', app, '--system', seed, '--module', library])
        run([runtime, 'run', app, '--instructions', '100000000', '--system', seed,
             '--module', library], status, expected)
    inputs = [library, ownership, seed, core, compiler, runtime, Path(__file__)]
    inputs += [compiler.parent / name for name in
               ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    inputs += [path for _, sources, _, _ in cases for path in sources] + list(output.glob('*.dll'))
    revision = lambda path: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=path, text=True).strip()
    evidence = dict(runtimeRevision=revision(ROOT), compilerRevision=revision(compiler.parent),
                    instructionBudget=100000000, commands=commands,
                    hashes={str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in inputs})
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()

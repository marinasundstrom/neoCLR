"""Build unchanged heap sources and execute a separate native metadata consumer."""
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
    parser.add_argument('--compiler-revision', help='Declared source revision for an extracted compiler bundle')
    args = parser.parse_args()
    compiler, library, ownership, seed, core, runtime, output = (
        getattr(args, name).resolve() for name in
        ('compiler', 'library', 'ownership', 'seed', 'core', 'runtime', 'output'))
    output.mkdir(parents=True, exist_ok=False)
    common = ['dotnet', compiler, 'neoclr', '--core-reference', core, '--runtime-seed', seed,
              '--bootstrap-intrinsics', '--bootstrap-ownership', ownership, '--reference', library]
    sources = [ROOT / path for path in (
        'runtime/raven/src/System/Runtime/GC.rvn',
        'runtime/raven/native/RuntimeHeapCalls.rvn',
        'runtime/raven/native/RuntimeHeapServices.rvn')]
    sample = Path(__file__).resolve().parent / 'bootstrap/heap-retention-consumer.rvn'
    heap, app = output / 'Heap.dll', output / 'HeapConsumer.dll'
    commands = []

    def run(command, expected_output=None, expected_exit=0):
        command = [str(part) for part in command]
        result = subprocess.run(command, cwd=output, capture_output=True, text=True, timeout=180)
        commands.append(dict(command=command, cwd=str(output), exitCode=result.returncode,
                             stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != expected_exit or expected_output is not None and result.stdout != expected_output:
            raise RuntimeError(json.dumps(commands[-1], indent=2))

    run(common + ['--library', '-o', heap] + sources)
    run(common + ['--reference', heap, '-o', app, sample])
    dependencies = ['--system', seed, '--module', library, '--module', heap]
    run([runtime, 'verify', app] + dependencies)
    run([runtime, 'run', app, '--instructions', '100000000'] + dependencies,
        'Native source heap passed\n')
    # Import the emitted nullable signature with library sources absent, then execute.
    nullable_sample = sample.with_name('heap-consumer.rvn')
    nullable_app = output / 'HeapNullableConsumer.dll'
    run(common + ['--reference', heap, '-o', nullable_app, nullable_sample])
    run([runtime, 'verify', nullable_app] + dependencies)
    run([runtime, 'run', nullable_app, '--instructions', '100000000'] + dependencies,
        'Native source heap passed\n')
    inputs = [library, ownership, seed, core, compiler, runtime, Path(__file__), sample, nullable_sample, heap, app, nullable_app] + sources
    inputs += [compiler.parent / name for name in
               ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    revision = lambda path: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=path, text=True).strip()
    evidence = dict(runtimeRevision=revision(ROOT), compilerRevision=args.compiler_revision or revision(compiler.parent),
                    instructionBudget=100000000, commands=commands,
                    limitation="Explicit callable annotations only; nullable context and fields remain pending",
                    hashes={str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in inputs})
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()

"""Build unchanged calendar sources and execute a separate native metadata consumer."""
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
    parser.add_argument('--compiler-revision', required=True)
    args = parser.parse_args()
    compiler, library, ownership, seed, core, runtime, output = (
        getattr(args, name).resolve() for name in
        ('compiler', 'library', 'ownership', 'seed', 'core', 'runtime', 'output'))
    output.mkdir(parents=True, exist_ok=False)
    common = ['dotnet', compiler, 'neoclr', '--core-reference', core, '--runtime-seed', seed,
              '--bootstrap-intrinsics', '--bootstrap-ownership', ownership, '--reference', library]
    sources = [ROOT / path for path in (
        'runtime/raven/src/System/TimeZone.rvn',
        'runtime/raven/src/System/TimeZoneError.rvn',
        'runtime/raven/src/System/ZonedDateTime.rvn',
        'runtime/raven/src/System/LocalTimeMapping.rvn',
        'runtime/raven/src/System/DateTime.rvn',
        'runtime/raven/native/RuntimeTimeZoneCalls.rvn',
        'runtime/raven/native/RuntimeTimeZoneServices.rvn')]
    sample = Path(__file__).resolve().parent / 'bootstrap/calendar-consumer.rvn'
    calendar, app = output / 'Calendar.dll', output / 'CalendarConsumer.dll'
    commands = []

    def run(command, expected_output=None):
        command = [str(part) for part in command]
        result = subprocess.run(command, cwd=output, capture_output=True, text=True, timeout=180)
        commands.append(dict(command=command, cwd=str(output), exitCode=result.returncode,
                             stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != 0 or expected_output is not None and result.stdout != expected_output:
            raise RuntimeError(json.dumps(commands[-1], indent=2))

    run(common + ['--library', '-o', calendar] + sources)
    run(common + ['--reference', calendar, '-o', app, sample])
    dependencies = ['--system', seed, '--module', library, '--module', calendar]
    run([runtime, 'verify', app] + dependencies)
    run([runtime, 'run', app, '--instructions', '100000000'] + dependencies,
        'Native source calendar passed\n')
    inputs = [library, ownership, seed, core, compiler, runtime, Path(__file__), sample, calendar, app] + sources
    inputs += [ROOT / name for name in ('src/native.rs', 'src/vm.rs')]
    inputs += [compiler.parent / name for name in
               ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    revision = lambda path: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=path, text=True).strip()
    evidence = dict(runtimeRevision=revision(ROOT), compilerRevision=args.compiler_revision,
                    instructionBudget=100000000, commands=commands,
                    hashes={str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in inputs})
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()

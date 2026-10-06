"""Compile source Console with explicit reduced bootstrap, then execute a native consumer."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'probe', 'library', 'integers', 'ownership', 'runtime', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--compiler-revision', required=True)
    args = parser.parse_args()
    compiler, probe, library, integers, ownership, runtime, output = (
        getattr(args, name).resolve() for name in
        ('compiler', 'probe', 'library', 'integers', 'ownership', 'runtime', 'output'))
    output.mkdir(parents=True, exist_ok=False)
    sources = [ROOT / path for path in (
        'runtime/raven/src/System/Console/Functions.rvn',
        'runtime/raven/src/System/Console/Streams.rvn',
        'runtime/raven/src/System/ConsoleReadError.rvn',
        'runtime/raven/native/RuntimeConsoleCalls.rvn',
        'runtime/raven/native/RuntimeConsoleServices.rvn',
        'runtime/raven/native/RuntimeNativeIntegerCalls.rvn',
        'runtime/raven/native/RuntimeNativeIntegerServices.rvn')]
    manifest = json.loads(ownership.read_text())
    manifest['libraries'].append(dict(assemblyName='Console',
        sources=[str(path.relative_to(ROOT)) for path in sources],
        types=['System.Console', 'System.ConsoleReadError', 'System.IO.ConsoleInputStream',
               'System.IO.ConsoleOutputStream']))
    selected = output / 'ownership.json'
    selected.write_text(json.dumps(manifest, indent=2) + '\n')
    seed_source = ROOT / 'runtime/raven/native/poc-seed.neoil'
    seed_text = seed_source.read_text()
    # Remove this exact legacy owner and service together. All other seed services remain.
    legacy = '''.type System.Console
.method static WriteLine(Int32 value) -> Void
ldarg 0
call neoCLR.Runtime.Int32ToString(Int32)
call neoCLR.Runtime.WriteLine(String)
ret
.end
.method static WriteLine(String value) -> Void
ldarg 0
call neoCLR.Runtime.WriteLine(String)
ret
.end
.end
.function neoCLR.Runtime.WriteLine(String value) -> Void
.methodimpl InternalCall
.end
'''
    if seed_text.count(legacy) != 1:
        raise RuntimeError('Console seed ownership changed; review the explicit reduction')
    reduced = output / 'System.neoil'
    reduced.write_text(seed_text.replace(legacy, ''))
    core, seed, console, consumer = [output / name for name in
                                    ('Core.dll', 'System.neox', 'Console.dll', 'Consumer.dll')]
    commands = []

    def run(command, stdin=None, stdout=None, stderr=None, exit_code=0):
        command = [str(part) for part in command]
        result = subprocess.run(command, cwd=output, input=stdin, capture_output=True,
                                text=True, timeout=180)
        commands.append(dict(command=command, cwd=str(output), stdin=stdin,
                             exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if (result.returncode != exit_code or stdout is not None and result.stdout != stdout
                or stderr is not None and result.stderr != stderr):
            raise RuntimeError(json.dumps(commands[-1], indent=2))

    run(['dotnet', probe, '--reference-source-console-core', core])
    run([runtime, 'assemble', reduced, seed, '--format', 'neox'])
    common = ['dotnet', compiler, 'neoclr', '--core-reference', core, '--runtime-seed', seed,
              '--bootstrap-ownership', selected, '--reference', library, '--reference', integers]
    run(common + ['--library', '-o', console] + sources)
    sample = Path(__file__).resolve().parent / 'bootstrap/console-consumer.rvn'
    run(common + ['--reference', console, '-o', consumer, sample])
    dependencies = ['--system', seed, '--module', library, '--module', integers, '--module', console]
    run([runtime, 'verify', consumer] + dependencies)
    run([runtime, 'run', consumer] + dependencies, stdin='héllo\n',
        stdout='Native source console\n42\nTrue\n42\n0\n0\nräven\nstill open\n',
        stderr='native stderr\n', exit_code=42)
    inputs = sources + [Path(__file__), sample, library, integers, ownership, selected, seed_source,
                        reduced, core, seed, console, consumer, compiler, probe, runtime]
    inputs += [compiler.parent / name for name in
               ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    inputs += [ROOT / path for path in ('src/native.rs', 'src/vm_host_call.rs', 'tests/console.rs',
               'docs/experiments/raven-target/CoreDeclarations.cs', 'docs/experiments/raven-target/Program.cs')]
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    evidence = dict(runtimeRevision=revision, compilerRevision=args.compiler_revision,
                    commands=commands,
                    hashes={str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in inputs})
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()

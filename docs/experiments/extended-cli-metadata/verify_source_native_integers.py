"""Build unchanged native integer sources and execute a separate native metadata consumer."""
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
    sources = [ROOT / path for path in (
        'runtime/raven/src/System/IntPtr.rvn',
        'runtime/raven/src/System/UIntPtr.rvn',
        'runtime/raven/native/RuntimeNativeIntegerCalls.rvn',
        'runtime/raven/native/RuntimeNativeIntegerServices.rvn')]
    manifest = json.loads(ownership.read_text())
    manifest['libraries'].append(dict(assemblyName='NativeIntegers',
        sources=[str(path.relative_to(ROOT)) for path in sources], types=['System.IntPtr', 'System.UIntPtr']))
    manifest['nativePrimitives'].update({'System.IntPtr': 'NativeIntegers', 'System.UIntPtr': 'NativeIntegers'})
    selected_ownership = output / 'ownership.json'
    selected_ownership.write_text(json.dumps(manifest, indent=2) + '\n')
    common = ['dotnet', compiler, 'neoclr', '--core-reference', core, '--runtime-seed', seed,
              '--bootstrap-ownership', selected_ownership, '--reference', library]
    sample = Path(__file__).resolve().parent / 'bootstrap/native-integers-consumer.rvn'
    integers, inputs, app = output / 'NativeIntegers.dll', output / 'NativeWidthInputs.dll', output / 'Consumer.dll'
    commands = []

    def run(command, expected_output=None, expected_exit=0):
        command = [str(part) for part in command]
        result = subprocess.run(command, cwd=output, capture_output=True, text=True, timeout=180)
        commands.append(dict(command=command, cwd=str(output), exitCode=result.returncode,
                             stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != expected_exit or expected_output is not None and result.stdout != expected_output:
            raise RuntimeError(json.dumps(commands[-1], indent=2))

    # API-authored nonzero native values avoid inventing Raven cast semantics.
    run(['dotnet', 'run', '--project', ROOT / 'tools/metadata/NeoCLR.Metadata.Experimental.Tests',
         '--no-build', '--', '--native-integer-inputs', core, inputs])
    run(common + ['--library', '-o', integers] + sources)
    run(common + ['--reference', integers, '--reference', inputs, '-o', app, sample])
    dependencies = ['--system', seed, '--module', library, '--module', integers, '--module', inputs]
    run([runtime, 'verify', app] + dependencies)
    run([runtime, 'run', app] + dependencies, '', 42)
    inputs = [library, ownership, selected_ownership, seed, core, compiler, runtime,
              Path(__file__), sample, integers, inputs, app] + sources
    inputs += [ROOT / 'tools/metadata/NeoCLR.Metadata.Experimental.Tests/NativeIntegerChecks.cs']
    inputs += [ROOT / name for name in ('src/native.rs', 'src/services.rs')]
    inputs += [compiler.parent / name for name in
               ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    revision = lambda path: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=path, text=True).strip()
    evidence = dict(runtimeRevision=revision(ROOT), compilerRevision=args.compiler_revision,
                    commands=commands,
                    hashes={str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in inputs})
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()

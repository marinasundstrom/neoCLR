"""Execute source NativeMemory through a separately compiled native consumer."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'core', 'seed', 'numbers', 'integers', 'inputs', 'ownership', 'runtime', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--compiler-revision', required=True)
    args = parser.parse_args()
    for name in ('compiler', 'core', 'seed', 'numbers', 'integers', 'inputs', 'ownership', 'runtime', 'output'):
        setattr(args, name, getattr(args, name).resolve())
    output = args.output
    output.mkdir(parents=True, exist_ok=False)
    sources = [ROOT / p for p in ('runtime/raven/src/System/Runtime/InteropServices/NativeMemory/Functions.rvn',
        'runtime/raven/native/NativeAllocation.rvn', 'runtime/raven/native/RuntimeNativeAllocationCalls.rvn')]
    manifest = json.loads(args.ownership.read_text())
    manifest['libraries'].append(dict(assemblyName='NativeMemory', sources=[str(p.relative_to(ROOT)) for p in sources],
                                     types=['System.Runtime.CompilerServices.NativeAllocation']))
    ownership = output / 'ownership.json'
    ownership.write_text(json.dumps(manifest, indent=2) + '\n')
    commands = []

    def run(command, expected=0):
        result = subprocess.run([str(p) for p in command], cwd=ROOT, capture_output=True, text=True, timeout=180)
        commands.append(dict(command=result.args, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != expected:
            raise RuntimeError(json.dumps(commands[-1], indent=2))
        return result

    common = ['dotnet', args.compiler, 'neoclr', '--core-reference', args.core, '--runtime-seed', args.seed,
              '--bootstrap-ownership', ownership, '--reference', args.numbers, '--reference', args.integers]
    library = output / 'NativeMemory.dll'
    run(common + ['--library', '-o', library] + sources)
    sample = Path(__file__).resolve().parent / 'bootstrap/native-memory-consumer.rvn'
    dependencies = ['--system', args.seed, '--module', args.numbers, '--module', args.integers,
                    '--module', args.inputs, '--module', library]
    consumers = []
    cases = [('Consumer', sample.read_text(), 42, None),
             ('DoubleFree', sample.read_text().replace('Free(Identity(pointer))', 'Free(pointer)\n    Free(pointer)'), 1, 'double free'),
             ('Overflow', sample.read_text().replace('NativeWidthInputs.Six()', 'NativeWidthInputs.Maximum()'), 1, 'code=ArithmeticOverflow')]
    for name, source, expected, fault in cases:
        path = output / (name + '.rvn')
        path.write_text(source)
        artifact = output / (name + '.dll')
        run(common + ['--reference', library, '--reference', args.inputs, '-o', artifact, path])
        run([args.runtime, 'verify', artifact] + dependencies)
        result = run([args.runtime, 'run', artifact] + dependencies, expected)
        if result.stdout or (fault and fault not in result.stderr) or (not fault and result.stderr):
            raise RuntimeError('Unexpected execution output: ' + repr(result))
        consumers += [path, artifact]
    rejected = output / 'Unsupported.dll'
    bad = output / 'unsupported.rvn'
    bad.write_text('public unsafe func Echo(pointer: *string) -> *string { return pointer }\n')
    result = run(common + ['--reference', library, '--library', '-o', rejected, bad], 1)
    if rejected.exists() or 'NEOMETA001' not in result.stderr + result.stdout:
        raise RuntimeError('Unsupported pointer must reject before publication')
    inputs = sources + consumers + [Path(__file__), sample, bad, ownership, library, args.compiler,
        args.core, args.seed, args.numbers, args.integers, args.inputs, args.ownership, args.runtime]
    inputs += [args.compiler.parent / name for name in ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    evidence = dict(sourceRevision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                    compilerRevision=args.compiler_revision, commands=commands,
                    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs})
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()

"""Compile unchanged union library sources, then execute a reference-only consumer.

Build the compiler and generate a primitive storage core with the documented
reference-storage-core command first. No source/seed union projections are used.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--core', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    compiler, runtime, core = (str(p.resolve()) for p in (args.compiler, args.runtime, args.core))
    commands = []

    def run(command, expected=0, stdout=None):
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=120)
        commands.append(dict(command=command, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        if result.returncode != expected or stdout is not None and result.stdout != stdout:
            raise RuntimeError(json.dumps(commands[-1], indent=2))
        return result

    manifest = HERE / 'union-ownership.json'
    sources = json.loads(manifest.read_text())['libraries'][0]['sources']
    seed = output / 'System.neox'
    library = output / 'NeoCLR.Collections.dll'
    consumer = output / 'Consumer.dll'
    run([runtime, 'assemble', str(HERE / 'union-seed.neoil'), str(seed), '--format', 'neox'])
    common = ['dotnet', compiler, 'neoclr', '--core-reference', core, '--runtime-seed', str(seed),
              '--bootstrap-ownership', str(manifest), '--bootstrap-intrinsics']
    run(common + ['--library', '-o', str(library)] + sources)
    run(common + ['--reference', str(library), '-o', str(consumer), str(HERE / 'union-consumer.rvn')])
    run([runtime, 'verify', str(consumer), '--module', str(library), '--system', str(seed)])
    run([runtime, 'run', str(consumer), '--module', str(library), '--system', str(seed)], 42,
        'Option.Some(40)\nResult.Error(7)\n')
    rejected = output / 'MissingLibrary.dll'
    run(common + ['-o', str(rejected), str(HERE / 'union-consumer.rvn')], 1)
    if rejected.exists():
        raise RuntimeError('Missing source-library dependency published output')
    bad_source = output / 'DuplicateSeed.neoil'
    bad_source.write_text((HERE / 'union-seed.neoil').read_text() + '\n.type System.Option<T>\n.end\n')
    bad_seed = output / 'DuplicateSeed.neox'
    run([runtime, 'assemble', str(bad_source), str(bad_seed), '--format', 'neox'])
    bad_args = common.copy()
    bad_args[bad_args.index('--runtime-seed') + 1] = str(bad_seed)
    failure = run(bad_args + ['--library', '-o', str(rejected)] + sources, 1)
    if rejected.exists() or 'duplicates a source-owned declaration' not in failure.stderr:
        raise RuntimeError('Duplicate seed ownership was not rejected before output')
    paths = [Path(compiler), Path(runtime), Path(core), seed, library, consumer, manifest,
             HERE / 'union-seed.neoil', HERE / 'union-consumer.rvn'] + [ROOT / p for p in sources]
    evidence = dict(scope='Native unchanged Option/Result plus iteration contracts; separate native import and execution. Not the full dual-target class-library gate.',
                    commands=commands, artifacts=[dict(path=str(p), sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in paths])
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print('PASS unchanged source Option/Result native library and separate consumer')


if __name__ == '__main__':
    main()

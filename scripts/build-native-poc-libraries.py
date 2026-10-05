#!/usr/bin/env python3
"""Build the bounded POC libraries directly from Raven source into native metadata.

The primitive core and retained runtime seed are explicit bootstrap inputs. This
command never invokes the CLI translation bridge or regenerates legacy fragments.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'core', 'seed', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--compiler-revision', help='Declared source revision for an extracted SDK')
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    ownership_source = ROOT / 'runtime/raven/native/poc-ownership.json'
    ownership = output / 'ownership.json'
    ownership.write_bytes(ownership_source.read_bytes())
    manifest = json.loads(ownership.read_text())
    numbers = [ROOT / p for p in manifest['libraries'][0]['sources']]
    http = sorted((ROOT / 'runtime/raven/src/System/Networking').rglob('*.rvn'))
    http += sorted((ROOT / 'runtime/raven/src/System/Web').rglob('*.rvn'))
    http += [ROOT / 'runtime/raven/src/System' / n for n in ('Uri.rvn', 'UriError.rvn')]
    http += [ROOT / 'runtime/raven/native' / n for n in ('RuntimeNetworkCalls.rvn', 'RuntimeNetworkServices.rvn')]
    inputs = [args.compiler.resolve(), args.core.resolve(), args.seed.resolve(), ownership_source, Path(__file__)]
    inputs += [args.compiler.resolve().parent / n for n in ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
    report = dict(passed=False, scope='source-built native POC subset; explicit primitive core and retained seed, not full System bootstrap',
                  declaredCompilerRevision=args.compiler_revision,
                  sourceRevision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                  inputs={str(p): sha(p) for p in inputs}, sources={str(p.relative_to(ROOT)): sha(p) for p in numbers + http}, builds=[])
    common = ['dotnet', str(args.compiler.resolve()), 'neoclr', '--core-reference', str(args.core.resolve()),
              '--runtime-seed', str(args.seed.resolve()), '--bootstrap-intrinsics', '--bootstrap-ownership', str(ownership), '--library']
    try:
        for name, sources, references in [('Numbers', numbers, []), ('Http', http, [output / 'Numbers.dll'])]:
            artifact = output / (name + '.dll')
            command = common + ['-o', str(artifact)]
            for reference in references:
                command += ['--reference', str(reference)]
            command += [str(p) for p in sources]
            result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=420)
            build = dict(name=name, command=command, sourceCount=len(sources), exitCode=result.returncode,
                         stdout=result.stdout, stderr=result.stderr, published=artifact.exists())
            report['builds'].append(build)
            if result.returncode != 0 or not artifact.exists():
                raise RuntimeError(f'{name} failed; inspect {output / "build.json"}')
            build['sha256'] = sha(artifact)
        report['passed'] = True
    finally:
        (output / 'build.json').write_text(json.dumps(report, indent=2) + '\n')
    print(output / 'build.json')


if __name__ == '__main__':
    main()

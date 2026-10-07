#!/usr/bin/env python3
"""Qualify relocated bundle configuration through workspace symbols and ordinary project execution."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('bundle', 'probe', 'compiler', 'runtime', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--compiler-revision', required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    report = dict(compilerRevision=args.compiler_revision, sourceRevision=subprocess.check_output(
        ['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), commands=[])

    def run(command):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True, timeout=180)
        report['commands'].append(dict(command=result.args, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        (output / 'evidence.json').write_text(json.dumps(report, indent=2) + '\n')
        if result.returncode:
            raise RuntimeError(result.stdout + result.stderr)
        return result

    consumer = output / 'consumer'
    source = ROOT / 'docs/experiments/http-headers/Main.rvn'
    run(['dotnet', args.probe.resolve(), '--native-bundle-project', args.bundle.resolve(), consumer, source])
    relocated = consumer / 'SDK with spaces'
    manifest = json.loads((relocated / 'bundle.json').read_text())
    for name, expected in manifest['files'].items():
        if Path(name).is_absolute() or '..' in Path(name).parts or hashlib.sha256((relocated / name).read_bytes()).hexdigest() != expected:
            raise RuntimeError('Relocated manifest mismatch: ' + name)
    project = consumer / 'Headers.rvnproj'
    result = run(['dotnet', args.compiler.resolve(), 'neoclr', '--project', project, '--run', args.runtime.resolve()])
    artifact = consumer / 'bin/neoclr/Headers.dll'
    if result.stdout != f'Native build output: {artifact}\nHTTP header lookup checks passed\n':
        raise RuntimeError('Unexpected relocated consumer output: ' + result.stdout)
    inputs = [Path(__file__), args.compiler, args.probe, args.runtime, source, project, artifact, relocated / 'bundle.json']
    inputs += [relocated / name for name in manifest['files']]
    for tool in (args.compiler, args.probe):
        inputs += [tool.parent / name for name in ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    report['hashes'] = {str(path.resolve()): hashlib.sha256(path.read_bytes()).hexdigest() for path in inputs}
    (output / 'evidence.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Relocated native bundle workspace and project execution passed')


if __name__ == '__main__':
    main()

"""Compile native HTTP sources and run artifact-only HTTP contracts and loopback consumers."""
import argparse
import hashlib
import json
import platform
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
    common = ['dotnet', compiler, 'neoclr', '--core-reference', core, '--runtime-seed', seed,
              '--bootstrap-intrinsics', '--bootstrap-ownership', ownership, '--reference', library]
    sources = sorted((ROOT / 'runtime/raven/src/System/Networking').rglob('*.rvn'))
    sources += [ROOT / 'runtime/raven/native' / name for name in
                ('RuntimeNetworkCalls.rvn', 'RuntimeNetworkServices.rvn')]
    sources += sorted((ROOT / 'runtime/raven/src/System/Web').rglob('*.rvn'))
    sources += [ROOT / 'runtime/raven/src/System' / name for name in ('Uri.rvn', 'UriError.rvn')]
    samples = {
        'headers': (['http-headers/Main.rvn'], 'HTTP header lookup checks passed\n'),
        'base': (['http-base/Main.rvn'], 'HTTP base address and overload checks passed\n'),
        'association': (['http-request-association/Main.rvn'], 'HTTP request/header checks passed\n'),
        'json-client': (['http-json-client/Main.rvn'], 'HTTP JSON client checks passed\n'),
        'routes': (['http-routing/Routes.rvn', 'http-routing/Direct.rvn', 'http-routing/Main.rvn'], 'Route parsing checks passed\n'),
        'self-capture': (['extended-cli-metadata/bootstrap/self-capture-consumer.rvn'], 'Captured self identity passed\n'),
        'property-pattern': (['extended-cli-metadata/bootstrap/property-pattern-consumer.rvn'], 'Property pattern checks passed\n'),
    }
    network = output / 'Http.dll'
    commands = []

    def run(command, expected_output=None):
        command = [str(part) for part in command]
        result = subprocess.run(command, cwd=output, capture_output=True, text=True, timeout=420)
        commands.append(dict(command=command, exitCode=result.returncode,
                             stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != 0 or expected_output is not None and result.stdout != expected_output:
            raise RuntimeError(json.dumps(commands[-1], indent=2))

    run(common + ['--library', '-o', network] + sources)
    inputs = [library, ownership, seed, core, compiler, runtime, Path(__file__), network] + sources
    dependencies = ['--system', seed, '--module', library, '--module', network]
    for name, (relative_sources, expected) in samples.items():
        sample_sources = [ROOT / 'docs/experiments' / path for path in relative_sources]
        app = output / (name + '.dll')
        run(common + ['--reference', network, '-o', app] + sample_sources)
        run([runtime, 'verify', app] + dependencies)
        run([runtime, 'run', app, '--instructions', '100000000'] + dependencies, expected)
        inputs += sample_sources + [app]
    for name in ('http-cancellation', 'http-status', 'http-stream-upload'):
        script = ROOT / 'docs/experiments' / name / 'verify.py'
        run(['python3', script, '--compiler', compiler, '--core', core, '--seed', seed,
             '--ownership', ownership, '--native-library', library, '--native-library', network,
             '--runner', runtime])
        inputs += [script] + sorted(script.parent.glob('*.rvn')) + sorted(script.parent.glob('*.cs'))
    inputs += [compiler.parent / name for name in
               ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    revision = lambda path: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=path, text=True).strip()
    evidence = dict(runtimeRevision=revision(ROOT), compilerRevision=revision(compiler.parent),
                    platform=platform.platform(), runtimePath=str(runtime),
                    dotnetSdk=subprocess.check_output(['dotnet', '--version'], text=True).strip(),
                    rustc=subprocess.check_output(['rustc', '--version'], text=True).strip(),
                    instructionBudget=100000000, commands=commands,
                    hashes={str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in inputs})
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()

"""Executable .NET service checks followed by the unchanged source-library gate.

Failures are recorded in validation.json and return a failing process status. An
emitted library alone does not count as successful CLR consumption or execution.
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
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    compiler = args.compiler.resolve()
    commands = []
    artifacts = [Path(__file__).resolve(), compiler]
    stage = 'services'
    passed = False
    failure = None

    def run(command, expected=0, stdout=None, stderr=None):
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=180)
        commands.append(dict(command=command, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        if result.returncode != expected or stdout is not None and result.stdout != stdout or stderr is not None and result.stderr != stderr:
            raise RuntimeError('Unexpected result during ' + stage)

    try:
        services_project = HERE / 'dotnet-services/NeoCLR.DotNetServices.csproj'
        artifacts += [services_project, services_project.with_name('Services.cs')]
        run(['dotnet', 'build', str(services_project), '-o', str(output), '-p:WarningLevel=0'])
        services = output / 'NeoCLR.DotNetServices.dll'
        artifacts.append(services)
        common = ['dotnet', str(compiler), '--framework', 'net10.0', '--emit-core-types-only', '--refs', str(services)]

        def runtime_config(name):
            path = output / (name + '.runtimeconfig.json')
            path.write_text(json.dumps({'runtimeOptions': {'tfm': 'net10.0', 'framework': {'name': 'Microsoft.NETCore.App', 'version': '10.0.0'}}}) + '\n')
            artifacts.append(path)

        for name, expected, error in [('storage', 42, ''), ('failure', 1, 'adapter failure\n')]:
            source = HERE / ('dotnet-services/' + name + '-consumer.rvn')
            app = output / (name + '.dll')
            artifacts += [source, app]
            run(common + ['-o', str(app), str(source)])
            runtime_config(name)
            run(['dotnet', str(app)], expected, '', error)

        stage = 'source library compilation'
        manifest = HERE / 'array-ownership.json'
        sources = [ROOT / p for p in json.loads(manifest.read_text())['libraries'][0]['sources']]
        # The library's real namespace function delegates to the target service adapter.
        sources.append(ROOT / 'runtime/raven/src/System/Functions.rvn')
        library = output / 'NeoCLR.Collections.dll'
        artifacts += [manifest, library] + sources
        configured = common + ['--bootstrap-ownership', str(manifest)]
        run(configured + ['--output-type', 'classlib', '-o', str(library)] + [str(p) for p in sources])
        stage = 'separate consumer compilation'
        source = ROOT / 'docs/experiments/raven-target/samples/application-order-collections.rvn'
        expected = source.with_suffix('.expected.txt')
        app = output / 'Application.dll'
        artifacts += [source, expected, app]
        run(configured + ['--refs', str(library), '-o', str(app), str(source)])
        stage = 'application execution'
        runtime_config('Application')
        run(['dotnet', str(app)], 0, expected.read_text(), '')
        passed = True
    except (RuntimeError, subprocess.TimeoutExpired) as error:
        failure = str(error)
    finally:
        for name in ['Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll']:
            artifacts.append(compiler.parent / name)
        report = dict(passed=passed, stage=stage, failure=failure,
                      ownership='Same source ownership as native gate; additional unchanged System/Functions.rvn delegates to explicit .NET services. Embedded shims avoid Raven.Core union copies.',
                      revisions={name: subprocess.check_output(['git', '-C', str(path), 'rev-parse', 'HEAD'], text=True).strip()
                                 for name, path in [('neoclr', ROOT), ('compiler', compiler.parent)]},
                      commands=commands,
                      artifacts=[dict(path=str(p), sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in artifacts if p.is_file()])
        report["outputPublication"] = {name: (output / name).exists() for name in ["NeoCLR.Collections.dll", "Application.dll"]}
        (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS paired .NET source-library application' if passed else 'FAIL ' + stage + ': ' + str(failure))
    return 0 if passed else 1


if __name__ == '__main__':
    raise SystemExit(main())

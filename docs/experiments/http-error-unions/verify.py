#!/usr/bin/env python3
"""Verify standard-syntax unions, initialization boundaries and conditional outputs."""
import argparse
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bundle', required=True, type=Path)
parser.add_argument('--runner', required=True, type=Path,
                    help='measure_async runner for forced collections and final live-object checks')
args = parser.parse_args()
bundle = args.bundle.resolve()
source = Path(__file__).resolve().parent
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))


def run(command, success=True):
    result = subprocess.run([str(part) for part in command], env=env,
                            capture_output=True, text=True, timeout=180)
    if success and result.returncode:
        raise AssertionError(result.stdout + result.stderr)
    return result


def rejected(result, message):
    assert result.returncode != 0, 'Unexpected success'
    assert message in result.stdout + result.stderr, result.stdout + result.stderr


with tempfile.TemporaryDirectory(prefix='neoclr-union-') as directory:
    work = Path(directory)
    for name in ('Probe.rvnproj', 'Errors.rvn', 'Main.rvn'):
        shutil.copyfile(source / name, work / name)
    compiled = work / 'compiled'
    compiled.mkdir()
    compiler = ['dotnet', bundle / 'raven-sdk/tools/rvnc/rvnc.dll', work / 'Probe.rvnproj',
                '--no-project-restore', '--configuration', 'Debug', '-o', compiled]
    run(compiler)
    assembly = compiled / 'Probe.dll'
    run(['dotnet', 'build', source / 'Shape.csproj', '-v:q'])
    inspector = ['dotnet', source / 'bin/Debug/net10.0/Shape.dll']
    report = run([*inspector, assembly]).stdout
    assert 'field <Tag>: System.Byte' in report, report
    print(report)

    def import_image(image, name, dependencies=()):
        destination = work / name
        result = run(['dotnet', bundle / 'tools/bridge/Probe.dll', '--import', image,
                      bundle / 'demo/NeoCLR.CoreProbe.dll', destination, *dependencies], success=False)
        return result, destination / 'App.neoil'

    imported, artifact = import_image(assembly, 'imported')
    assert imported.returncode == 0, imported.stdout + imported.stderr
    system = bundle / 'lib/System.neoil'
    run([bundle / 'bin/neoclr', 'verify', artifact, '--system', system])
    executed = run([args.runner.resolve(), artifact, system, '64', '10000000'])
    assert executed.stdout.splitlines() == [
        'Header limit', 'localhost', 'Invalid request', 'Union defaults, cases, copies and boxing passed'
    ], executed.stdout
    assert re.search(r'\blive=0\b', executed.stderr), executed.stderr
    assert re.search(r'\bcollections=[1-9][0-9]*\b', executed.stderr), executed.stderr
    print(executed.stdout + executed.stderr)

    for mutation in ('nonconstructor-init', 'ordinary-out', 'unassigned-success',
                     'explicit-unmarked', 'explicit-payload', 'explicit-tag-overlap'):
        image = compiled / (mutation + '.dll')
        run([*inspector, assembly, mutation, image])
        imported, artifact = import_image(image, mutation)
        if mutation.startswith('explicit-'):
            rejected(imported, 'Unsupported application type: HttpErrorProbe.HttpLimit')
        elif mutation == 'nonconstructor-init':
            rejected(imported, 'Only local or value-constructor receiver initialization is admitted.')
        else:
            assert imported.returncode == 0, imported.stdout + imported.stderr
            rejected(run([bundle / 'bin/neoclr', 'run', artifact, '--system', system], success=False),
                     'out parameter has not been assigned')
        print('Rejected invalid contract: ' + mutation)

    # Exercise the metadata contract across a separately compiled library boundary.
    (work / 'Errors.rvnproj').write_text('''<Project>
  <Import Project="$(NeoCLRRoot)/build/NeoCLR.Raven.props" />
  <PropertyGroup><OutputType>Library</OutputType></PropertyGroup>
  <ItemGroup><Compile Include="Errors.rvn" /></ItemGroup>
</Project>''')
    library_output = work / 'library'
    library_output.mkdir()
    run(['dotnet', bundle / 'raven-sdk/tools/rvnc/rvnc.dll', work / 'Errors.rvnproj',
         '--no-project-restore', '-o', library_output])
    library = library_output / 'Errors.dll'
    (work / 'Probe.rvnproj').write_text((source / 'Probe.rvnproj').read_text()
        .replace('    <Compile Include="Errors.rvn" />',
                 '    <Reference Include="Errors"><HintPath>library/Errors.dll</HintPath></Reference>'))
    run(compiler)
    imported, artifact = import_image(assembly, 'library-import', (library,))
    assert imported.returncode == 0, imported.stdout + imported.stderr
    run([bundle / 'bin/neoclr', 'verify', artifact, '--system', system])
    library_run = run([args.runner.resolve(), artifact, system, '64', '10000000'])
    assert library_run.stdout == executed.stdout, library_run.stdout
    assert re.search(r'\blive=0\b', library_run.stderr), library_run.stderr
    assert re.search(r'\bcollections=[1-9][0-9]*\b', library_run.stderr), library_run.stderr
    print('Separate union library: ' + library_run.stderr)
    shutil.copyfile(source / 'Probe.rvnproj', work / 'Probe.rvnproj')

    # Mixing the existing erased SocketError with zero-initialized source unions is
    # deliberately still unsupported. Do not invent a default System.Value payload.
    shutil.copyfile(source / 'LegacyErrors.rvn', work / 'Errors.rvn')
    (work / 'Main.rvn').write_text('''import System.*
import System.Networking.Sockets.*
import HttpErrorProbe.*
func Main() {
    let error: HttpError = HttpError.Transport(SocketError.ConnectionRefused())
    Console.WriteLine(error.ToString())
}
''')
    run(compiler)
    imported, artifact = import_image(assembly, 'legacy')
    assert imported.returncode == 0, imported.stdout + imported.stderr
    rejected(run([bundle / 'bin/neoclr', 'verify', artifact, '--system', system], success=False),
             'managed default initialization is not defined for Value')
    print('Mixed legacy SocketError still rejected: no erased-payload default.')

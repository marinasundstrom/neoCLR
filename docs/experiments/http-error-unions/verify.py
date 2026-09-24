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

    def import_image(image, name):
        destination = work / name
        result = run(['dotnet', bundle / 'tools/bridge/Probe.dll', '--import', image,
                      bundle / 'demo/NeoCLR.CoreProbe.dll', destination], success=False)
        return result, destination / 'App.neoil'

    imported, artifact = import_image(assembly, 'imported')
    assert imported.returncode == 0, imported.stdout + imported.stderr
    system = bundle / 'lib/System.neoil'
    run([bundle / 'bin/neoclr', 'verify', artifact, '--system', system])
    executed = run([args.runner.resolve(), artifact, system, '64', '10000000'])
    assert executed.stdout.splitlines() == [
        'localhost', 'Invalid request', 'Union defaults, cases, copies and boxing passed'
    ], executed.stdout
    assert re.search(r'\blive=0\b', executed.stderr), executed.stderr
    assert re.search(r'\bcollections=[1-9][0-9]*\b', executed.stderr), executed.stderr
    print(executed.stdout + executed.stderr)

    for mutation in ('nonconstructor-init', 'ordinary-out', 'unassigned-success'):
        image = compiled / (mutation + '.dll')
        run([*inspector, assembly, mutation, image])
        imported, artifact = import_image(image, mutation)
        if mutation == 'nonconstructor-init':
            rejected(imported, 'Only local or value-constructor receiver initialization is admitted.')
        else:
            assert imported.returncode == 0, imported.stdout + imported.stderr
            rejected(run([bundle / 'bin/neoclr', 'run', artifact, '--system', system], success=False),
                     'out parameter has not been assigned')
        print('Rejected invalid contract: ' + mutation)

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

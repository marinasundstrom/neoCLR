#!/usr/bin/env python3
"""Build the checked-in Runtime project, finalize its seed, and execute unchanged orders."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'core', 'bootstrap-directory', 'translator', 'runtime', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--compiler-revision', required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    project = ROOT / 'runtime/raven/projects/System.Runtime/System.Runtime.rvnproj'
    ownership = project.parent / 'ownership.json'
    bootstrap = args.bootstrap_directory.resolve()
    library = project.parent / 'bin/neoclr/System.Runtime.dll'
    report = dict(compilerRevision=args.compiler_revision, sourceRevision=subprocess.check_output(
        ['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), commands=[])
    environment = dict(os.environ, RavenNeoClrCoreReference=str(args.core.resolve()),
                       RavenNeoClrRuntimeSeed=str(bootstrap / 'System.neox'))
    report['environment'] = {name: environment[name] for name in ('RavenNeoClrCoreReference', 'RavenNeoClrRuntimeSeed')}

    def run(command):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, env=environment, capture_output=True, text=True, timeout=300)
        report['commands'].append(dict(command=result.args, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        (output / 'evidence.json').write_text(json.dumps(report, indent=2) + '\n')
        if result.returncode:
            raise RuntimeError(result.stdout + result.stderr)
        return result

    run(['dotnet', args.compiler.resolve(), 'neoclr', '--project', project])
    seed = output / 'System.runtime.neox'
    run(['dotnet', args.translator.resolve(), bootstrap / 'System.retained.json', seed, '--reference', library])
    sample = ROOT / 'docs/experiments/raven-target/samples/application-order-collections.rvn'
    shutil.copyfile(sample, output / 'Main.rvn')
    consumer = ET.Element('Project', Sdk='Microsoft.NET.Sdk')
    properties = ET.SubElement(consumer, 'PropertyGroup')
    for name, value in dict(TargetFramework='net10.0', OutputType='Exe', AssemblyName='Orders',
                           RavenTargetPlatform='NeoCLR', RavenMetadataFormat='NeoCLR',
                           RavenNeoClrCoreReference=str(args.core.resolve()), RavenNeoClrRuntimeSeed=str(seed),
                           RavenNeoClrBootstrapOwnership=str(ownership), RavenNeoClrObjectLibrary='System.Runtime',
                           RavenNeoClrAsyncLibrary='System.Runtime').items():
        ET.SubElement(properties, name).text = value
    reference = ET.SubElement(ET.SubElement(consumer, 'ItemGroup'), 'Reference', Include='System.Runtime')
    ET.SubElement(reference, 'HintPath').text = str(library)
    consumer_path = output / 'Orders.rvnproj'
    ET.ElementTree(consumer).write(consumer_path, encoding='unicode')
    result = run(['dotnet', args.compiler.resolve(), 'neoclr', '--project', consumer_path, '--run', args.runtime.resolve()])
    artifact = output / 'bin/neoclr/Orders.dll'
    expected_file = sample.with_suffix('.expected.txt')
    if result.stdout != f'Native build output: {artifact}\n' + expected_file.read_text():
        raise RuntimeError('Orders output differs from checked-in expected output: ' + result.stdout)
    sources = [ROOT / path for path in json.loads(ownership.read_text())['libraries'][0]['sources']]
    report['runtimeSourceCount'] = len(sources)
    paths = [Path(__file__), project, ownership, sample, expected_file, consumer_path, artifact, library, seed,
             args.compiler, args.core, args.runtime, args.translator, bootstrap / 'System.neox', bootstrap / 'System.retained.json', *sources]
    paths += [args.compiler.parent / name for name in ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    report['hashes'] = {str(path.resolve()): hashlib.sha256(path.read_bytes()).hexdigest() for path in paths}
    (output / 'evidence.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Runtime project build, seed finalization and source-free orders execution passed')


if __name__ == '__main__':
    main()

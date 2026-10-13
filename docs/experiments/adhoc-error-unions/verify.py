#!/usr/bin/env python3
"""Compile an imported config API and exercise both ad-hoc union error alternatives."""
import argparse
import hashlib
import json
import shutil
import sys
from pathlib import Path
import subprocess
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[3]
FIXTURE = Path(__file__).resolve().parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('bundle', 'compiler', 'runtime', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--codegen', type=Path, help='Also exercise a native-GC AOT and JIT carrier consumer')
    parser.add_argument('--jit-host', type=Path, help='Matching native-GC JIT host library')
    args = parser.parse_args()
    if bool(args.codegen) != bool(args.jit_host):
        parser.error('--codegen and --jit-host must be provided together')
    bundle, compiler, runtime, output = (getattr(args, n).resolve() for n in ('bundle', 'compiler', 'runtime', 'output'))
    output.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, commands=[], inputs={}, scope='native metadata, separate library import, interpreter')
    lib = bundle / 'lib'
    catalog = json.loads((lib / 'bundle.json').read_text())
    modules = [lib / (name + '.dll') for name in catalog['assemblyNames']]
    inputs = [Path(__file__), FIXTURE / 'Config.rvn', FIXTURE / 'Main.rvn', compiler, runtime,
              compiler.parent / 'Raven.CodeAnalysis.dll', compiler.parent / 'Raven.CodeAnalysis.NeoClr.dll',
              lib / 'bundle.json', lib / catalog['runtimeSeed'], *modules]
    report['inputs'] = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}

    def run(command, expected=0):
        result = subprocess.run(list(map(str, command)), cwd=output, capture_output=True, text=True, timeout=180)
        report['commands'].append(dict(command=result.args, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        if result.returncode != expected:
            raise RuntimeError(result.stdout + result.stderr)
        return result

    def project(name, source, library=False, reference=None):
        directory = output / name
        directory.mkdir()
        (directory / 'Main.rvn').write_text(source)
        root = ET.Element('Project', Sdk='Microsoft.NET.Sdk')
        ET.SubElement(root, 'Import', Project=str(lib / 'NeoCLR.ClassLibrary.props'))
        properties = ET.SubElement(root, 'PropertyGroup')
        for key, value in dict(TargetFramework='net10.0', OutputType='Library' if library else 'Exe',
                               AssemblyName=name, EnableDefaultCompileItems='false').items():
            ET.SubElement(properties, key).text = value
        items = ET.SubElement(root, 'ItemGroup')
        ET.SubElement(items, 'Compile', Include='Main.rvn')
        if reference:
            item = ET.SubElement(items, 'Reference', Include=reference.stem)
            ET.SubElement(item, 'HintPath').text = str(reference)
        path = directory / (name + '.rvnproj')
        ET.ElementTree(root).write(path, encoding='unicode')
        return path, directory / 'bin/neoclr' / (name + '.dll')

    try:
        config_project, config = project('ConfigLibrary', (FIXTURE / 'Config.rvn').read_text(), library=True)
        run(['dotnet', compiler, 'neoclr', '--project', config_project])
        app_project, app = project('ConfigConsumer', (FIXTURE / 'Main.rvn').read_text(), reference=config)
        run(['dotnet', compiler, 'neoclr', '--project', app_project])
        context = ['--system', lib / catalog['runtimeSeed'], '--object-root', lib / 'System.Runtime.dll']
        for module in [*modules, config]:
            context += ['--module', module]
        run([runtime, 'verify', app, *context])
        (output / 'valid.cfg').write_text('4242')
        (output / 'invalid.cfg').write_text('not-an-integer')
        result = run([runtime, 'run', app, '--execution', 'interpreter', *context])
        assert result.stdout == 'PASS config success, file error and parse error\n', result.stdout
        negative_project, negative = project('UnrelatedError', '''import System.*
import System.Storage.*
func Invalid(value: Result<int, BooleanParseError>) -> Result<int, FileReadError | NumberParseError> {
    let parsed = value?
    return Result<int, FileReadError | NumberParseError>.Ok(parsed)
}
func Main() -> int => 0
''')
        rejection = run(['dotnet', compiler, 'neoclr', '--project', negative_project], expected=1)
        assert 'RAV1503' in rejection.stdout + rejection.stderr
        assert 'BooleanParseError' in rejection.stdout + rejection.stderr
        assert not negative.exists()
        if args.codegen:
            codegen, host = args.codegen.resolve(), args.jit_host.resolve()
            for path in [codegen, host, FIXTURE / 'CarrierChecks.rvn']:
                report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
            compiled_project, _ = project('CarrierChecks', (FIXTURE / 'CarrierChecks.rvn').read_text())
            # The native build validates real bundle paths and hashes; stage an isolated
            # copy so this gate does not mutate an installed SDK or follow library links.
            native_bundle = output / 'native-bundle'
            shutil.copytree(lib, native_bundle / 'lib')
            shutil.copytree(compiler.parent, native_bundle / 'sdk/tools/rvnc')
            native = output / 'native'
            run([sys.executable, ROOT / 'scripts/build-native-project.py', '--profile', 'console',
                 '--project', compiled_project, '--bundle', native_bundle, '--aot', codegen, '--output', native])
            expected = 'PASS native union propagation and arities 2 through 5\n'
            assert run([native / 'app']).stdout == expected
            assert run([runtime, 'run', native / 'app.dll', '--execution', 'jit',
                        '--jit-host', host, '--jit-compiler', codegen, *context]).stdout == expected
            report['scope'] += '; native-GC AOT/JIT propagation and carrier arities (no file I/O)'
        report['passed'] = True
    finally:
        (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS imported config API: success, both typed errors, unrelated error rejection')


if __name__ == '__main__':
    main()

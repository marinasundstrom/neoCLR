#!/usr/bin/env python3
"""Exercise project native builds, standalone execution and fail-closed publication."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import xml.etree.ElementTree as X

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('bundle', 'aot', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    bundle, aot, output = (getattr(args, name).resolve() for name in ('bundle', 'aot', 'output'))
    output.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, cases=[])
    catalog = json.loads((bundle / 'lib/bundle.json').read_text())
    context = ['--system', bundle / 'lib' / catalog['runtimeSeed'],
               *[arg for name in catalog['assemblyNames'] for arg in ('--module', bundle / 'lib' / (name + '.dll'))],
               '--object-root', bundle / 'lib/System.Runtime.dll']
    cases = [
        ('utf8', 'func Main() -> int {\n    System.Console.WriteLine("Hello, Café 🌍")\n    return 0\n}\n', b'Hello, Caf\xc3\xa9 \xf0\x9f\x8c\x8d\n', 0),
        ('interpolation', (ROOT / 'docs/experiments/aot-console/interpolation.rvn').read_text(),
         (''.join(f'Value: {v}\n{v}\nValue: {v}\n' for v in (42, -2147483648, 2147483647)) + 'Null: \n').encode(), 0),
        ('fault', 'func Main() -> int {\n    return Divide(0)\n}\nfunc Divide(value: int) -> int {\n    return 42 / value\n}\n', b'', 1),
        ('unsupported', 'func Main() -> int {\n    return Loop(2)\n}\nfunc Loop(value: int) -> int {\n    if value == 0 {\n        return 0\n    }\n    return Loop(value - 1)\n}\n', None, None),
    ]
    try:
        for name, source, expected, code in cases:
            project_dir = output / (name + ' project with spaces')
            project_dir.mkdir()
            project = X.Element('Project', Sdk='Microsoft.NET.Sdk')
            X.SubElement(project, 'Import', Project=str(bundle / 'lib' / catalog['projectConfiguration']))
            group = X.SubElement(project, 'PropertyGroup')
            for key, value in dict(TargetFramework='net10.0', OutputType='Exe', AssemblyName='App').items():
                X.SubElement(group, key).text = value
            X.indent(project)
            path = project_dir / 'App.rvnproj'
            X.ElementTree(project).write(path, encoding='unicode')
            (project_dir / 'Main.rvn').write_text(source)
            destination = output / (name + ' native output')
            command = ['python3', ROOT / 'scripts/build-native-project.py', '--project', path,
                       '--bundle', bundle, '--aot', aot, '--output', destination]
            built = subprocess.run(list(map(str, command)), capture_output=True, timeout=240)
            evidence = json.loads((destination / 'build.json').read_text())
            row = dict(name=name, source=source, buildExit=built.returncode, build=evidence)
            report['cases'].append(row)
            if expected is None:
                assert built.returncode != 0 and not evidence['passed'], built
                assert not (destination / 'app').exists() and not (destination / 'app.pending').exists()
                assert any(c['exitCode'] != 0 and '--closed-world' in c['command'] for c in evidence['commands']), evidence
                continue
            assert built.returncode == 0 and evidence['passed'], built.stderr.decode()
            vm = subprocess.run(list(map(str, [bundle / 'bin/neoclr', 'run', destination / 'app.dll', *context])),
                                capture_output=True, timeout=60)
            with tempfile.TemporaryDirectory(prefix='neoclr standalone ') as directory:
                binary = Path(directory) / 'app'
                shutil.copy2(destination / 'app', binary)
                native = subprocess.run([str(binary)], cwd=directory, env={}, capture_output=True, timeout=30)
                assert list(Path(directory).iterdir()) == [binary]
            assert native.returncode == vm.returncode == code, (native, vm)
            assert native.stdout == vm.stdout == expected, (native, vm)
            assert native.stderr == vm.stderr, (native, vm)
            if code == 0:
                assert not native.stderr
            else:
                assert native.stderr
            row['execution'] = dict(exit=code, stdout=native.stdout.decode(), stderr=native.stderr.decode(),
                                    matchesInterpreter=True, emptyEnvironment=True, executableOnlyDirectory=True)
            digest = hashlib.sha256((destination / 'app').read_bytes()).hexdigest()
            repeated = subprocess.run(list(map(str, command)), capture_output=True, timeout=30)
            assert repeated.returncode != 0
            assert hashlib.sha256((destination / 'app').read_bytes()).hexdigest() == digest
            row['existingOutputPreserved'] = True
        # A failed rebuild must never consume a previous successful project DLL.
        project_dir = output / 'utf8 project with spaces'
        (project_dir / 'Main.rvn').write_text('func Main() -> int {\n    return missingValue\n}\n')
        destination = output / 'invalid rebuild'
        failed = subprocess.run(list(map(str, ['python3', ROOT / 'scripts/build-native-project.py',
            '--project', project_dir / 'App.rvnproj', '--bundle', bundle, '--aot', aot,
            '--output', destination])), capture_output=True, timeout=240)
        evidence = json.loads((destination / 'build.json').read_text())
        assert failed.returncode != 0 and not evidence['passed']
        assert not (destination / 'app.dll').exists() and not (destination / 'app').exists()
        assert any(c['exitCode'] != 0 and '--project' in c['command'] for c in evidence['commands'])
        report['cases'].append(dict(name='failed-rebuild', build=evidence, staleOutputRejected=True))
        report['passed'] = True
    finally:
        (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Passed: native project output, UTF-8/interpolation/fault parity, isolated execution, unsupported recursion rejection and output preservation')


if __name__ == '__main__':
    main()

#!/usr/bin/env python3
"""Build ordinary Raven projects and execute isolated Windows native console EXEs."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import xml.etree.ElementTree as X

ROOT = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True, help='Extracted matching native toolchain candidate')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, scope='Development Windows x64 synchronous native console project builds',
                  platform=platform.platform(), commands=[], cases=[])

    def run(command, name, expected=0, **kwargs):
        command = list(map(str, command))
        result = subprocess.run(command, capture_output=True, timeout=600, **kwargs)
        (out / (name + '.stdout.log')).write_bytes(result.stdout)
        (out / (name + '.stderr.log')).write_bytes(result.stderr)
        report['commands'].append(dict(command=command, exitCode=result.returncode))
        if result.returncode != expected:
            raise RuntimeError(f'{name}: expected exit {expected}, got {result.returncode}; see retained logs')
        return result

    try:
        if platform.system() != 'Windows' or platform.machine().lower() not in ('amd64', 'x86_64'):
            raise ValueError('Requires Windows x64 with MSVC and the .NET SDK used by the Raven bundle')
        report['revision'] = run(['git', '-C', ROOT, 'rev-parse', 'HEAD'], 'revision').stdout.decode().strip()
        bundle = args.bundle.resolve()
        manifest = json.loads((bundle / 'manifest.json').read_text())
        spec = importlib.util.spec_from_file_location('project_builder', ROOT / 'scripts/build-native-project.py')
        builder = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(builder)
        builder.verify_files(bundle, manifest['files'])
        compiler = manifest['jit']['compiler']
        if compiler not in manifest['files']:
            raise ValueError('Native compiler is not catalogued by the package')
        aot = bundle / compiler
        report['bundle'] = dict(path=str(bundle), manifestSha256=sha(bundle / 'manifest.json'),
                               compilerSha256=sha(aot))
        report['dotnet'] = run(['dotnet', '--info'], 'dotnet').stdout.decode(errors='replace')
        lib = bundle / 'lib'
        catalog = json.loads((lib / 'bundle.json').read_text())
        context = ['--system', lib / catalog['runtimeSeed'],
                   *[arg for name in catalog['assemblyNames'] for arg in ('--module', lib / (name + '.dll'))],
                   '--object-root', lib / 'System.Runtime.dll']
        cases = [
            ('managed', (ROOT / 'samples/native-windows/Main.rvn').read_text(encoding='utf-8'),
             'Hello, Café 🌍\nAnswer: 42\nhé😀\0z\nreplacement\n'.encode(), b'', 0, b''),
            ('interpolation', (ROOT / 'docs/experiments/aot-console/interpolation.rvn').read_text(),
             (''.join(f'Value: {v}\n{v}\nValue: {v}\n' for v in (42, -2147483648, 2147483647)) + 'Null: \n').encode(), b'', 0, b''),
            ('input', (ROOT / 'docs/experiments/aot-console/input-stream.rvn').read_text(),
             b'1\n26\nClosed input\n', b'', 0, b'\x1a'),
            ('output', (ROOT / 'docs/experiments/aot-console/output-stream.rvn').read_text(),
             b'A\0BClosed output\n', b'B', 0, b''),
            ('fault', 'func Main() -> int {\n    return Divide(0)\n}\nfunc Divide(value: int) -> int {\n    return 42 / value\n}\n',
             b'', None, 1, b''),
        ]
        for name, source, expected_stdout, expected_stderr, code, stdin in cases:
            project_dir = out / (name + ' project with spaces')
            project_dir.mkdir()
            tree = X.Element('Project', Sdk='Microsoft.NET.Sdk')
            X.SubElement(tree, 'Import', Project='$(NeoClrBundleRoot)/lib/NeoCLR.ClassLibrary.props')
            group = X.SubElement(tree, 'PropertyGroup')
            for key, value in dict(TargetFramework='net10.0', OutputType='Exe', AssemblyName='WindowsNative').items():
                X.SubElement(group, key).text = value
            project = project_dir / 'App.rvnproj'
            X.indent(tree)
            X.ElementTree(tree).write(project, encoding='unicode')
            (project_dir / 'Main.rvn').write_text(source, encoding='utf-8')
            destination = out / (name + ' native output')
            command = [sys.executable, ROOT / 'scripts/build-native-project.py', '--profile', 'windows-console',
                       '--project', project, '--bundle', bundle, '--aot', aot, '--output', destination]
            run(command, name + '-build', cwd=ROOT)
            build = json.loads((destination / 'build.json').read_text())
            if build.get('passed') is not True:
                raise ValueError('Build did not report completion')
            vm = run([bundle / 'bin/neoclr.exe', 'run', destination / 'app.dll', '--execution', 'interpreter', *context],
                     name + '-interpreter', expected=code, input=stdin)
            isolated = out / (name + ' executable only')
            isolated.mkdir()
            executable = isolated / 'app.exe'
            shutil.copy2(destination / 'app.exe', executable)
            # OS variables only: no SDK, bundle, managed runtime or helper DLL on PATH.
            environment = {key: value for key, value in os.environ.items()
                           if key.upper() in ('SYSTEMROOT', 'WINDIR', 'TEMP', 'TMP')}
            native = run([executable], name + '-native', expected=code, input=stdin, cwd=isolated, env=environment)
            if sorted(p.name for p in isolated.iterdir()) != ['app.exe']:
                raise ValueError('Standalone directory acquired unexpected files')
            if native.stdout != vm.stdout or native.stdout != expected_stdout or native.stderr != vm.stderr:
                raise ValueError(name + ': native/interpreter byte parity failed')
            if expected_stderr is not None and native.stderr != expected_stderr:
                raise ValueError(name + ': unexpected stderr')
            if code and b'DivideByZero:' not in native.stderr:
                raise ValueError('Missing guest fault diagnostic')
            report['cases'].append(dict(name=name, exitCode=code, stdout=native.stdout.decode('utf-8'),
                                       stderr=native.stderr.decode('utf-8'), dependencies=build['dependencies'],
                                       standalone=True, interpreterParity=True))
            if name == 'managed':
                # Existing artifacts are never overwritten. Then break the project
                # while its prior Raven output still exists: no stale EXE publication.
                previous = sha(destination / 'app.exe')
                run(command, 'existing-output-rejected', expected=1, cwd=ROOT)
                if sha(destination / 'app.exe') != previous:
                    raise ValueError('Existing executable was overwritten')
                (project_dir / 'Main.rvn').write_text('func Main( { invalid', encoding='utf-8')
                failed = out / 'failed rebuild'
                run([*command[:-1], failed], 'stale-output-rejected', expected=1, cwd=ROOT)
                if (failed / 'app.exe').exists() or (failed / 'app.pending.exe').exists():
                    raise ValueError('Failed rebuild published an executable')
                (project_dir / 'Main.rvn').write_text(source, encoding='utf-8')
        report['existingOutputPreserved'] = True
        report['staleOutputRejected'] = True
        report['passed'] = True
    except Exception as error:
        report['error'] = str(error)
    finally:
        report['files'] = {p.relative_to(out).as_posix(): sha(p) for p in sorted(out.rglob('*')) if p.is_file()}
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print('Windows native project: ' + ('PASS' if report['passed'] else 'FAIL: ' + report['error']))
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())

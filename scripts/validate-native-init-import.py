#!/usr/bin/env python3
"""Check native init contracts through a separate library and source consumer."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    out, bundle, runtime = args.output.resolve(), args.bundle.resolve(), args.runtime.resolve()
    out.mkdir(parents=True, exist_ok=False)
    compiler = bundle / 'sdk/tools/rvnc/rvnc.dll'
    sources = ROOT / 'docs/experiments/init-accessors'
    inputs = [Path(__file__).resolve(), runtime, bundle / 'manifest.json',
              *[p for folder in ('library', 'raven') for p in (sources / folder).iterdir() if p.is_file()],
              *[p for p in compiler.parent.iterdir() if p.is_file()]]
    report = dict(passed=False, inputs={str(p): sha(p) for p in inputs}, commands=[], rejections=[])
    env = dict(os.environ, NeoClrBundleRoot=str(bundle))

    def execute(command, name, environment=None):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, env=environment,
                                capture_output=True, text=True, timeout=180)
        (out / (name + '.stdout.txt')).write_text(result.stdout)
        (out / (name + '.stderr.txt')).write_text(result.stderr)
        report['commands'].append(dict(command=result.args, exitCode=result.returncode))
        return result

    def compile_project(project, name):
        result = execute(['dotnet', compiler, 'neoclr', '--project', project], name, env)
        if result.returncode:
            raise ValueError(name + ' failed: ' + result.stderr)
        return Path(next(line.removeprefix('Native build output: ') for line in result.stdout.splitlines()
                         if line.startswith('Native build output: ')))

    try:
        for folder in ('library', 'raven'):
            dest = out / folder
            dest.mkdir()
            for file in (sources / folder).iterdir():
                if file.is_file():
                    shutil.copy2(file, dest / file.name)
        library = compile_project(out / 'library/Native.rvnproj', 'library')
        env['InitContractsPath'] = str(library)
        consumer = compile_project(out / 'raven/Native.rvnproj', 'consumer')
        lib = bundle / 'lib'
        catalog = json.loads((lib / 'bundle.json').read_text())
        result = execute([runtime, 'run', consumer, '--system', lib / catalog['runtimeSeed'],
                          *[arg for name in catalog['assemblyNames'] for arg in ('--module', lib / (name + '.dll'))],
                          '--module', library, '--object-root', lib / 'System.Runtime.dll'], 'interpreter')
        if (result.returncode, result.stdout, result.stderr) != (0, 'Init accessors passed\n', ''):
            raise ValueError('imported init execution mismatch')
        source = (sources / 'raven/Main.rvn').read_text()
        for name, statement, expected in [('class-update', 'settings.Port = 9', 'RAV0200'),
                                           ('pair-update', 'pair.Value = 9', 'RAV0200'),
                                           ('direct-accessor', 'settings.set_Port(9)', 'RAV0117')]:
            dest = out / name
            dest.mkdir()
            shutil.copy2(sources / 'raven/Native.rvnproj', dest / 'Native.rvnproj')
            (dest / 'Main.rvn').write_text(source.replace('        if settings.Port',
                                                        f'        {statement}\n        if settings.Port'))
            result = execute(['dotnet', compiler, 'neoclr', '--project', dest / 'Native.rvnproj'], name, env)
            diagnostic = result.stdout + result.stderr
            if result.returncode == 0 or 'error ' + expected not in diagnostic or list(dest.rglob('*.dll')):
                raise ValueError(name + ' did not reject cleanly: ' + diagnostic)
            report['rejections'].append(dict(name=name, diagnostic=diagnostic))
        if any(sha(Path(p)) != digest for p, digest in report['inputs'].items()):
            raise ValueError('inputs changed during validation')
        report['passed'] = True
    except Exception as error:
        report['error'] = str(error)
    finally:
        report['files'] = {p.relative_to(out).as_posix(): sha(p) for p in sorted(out.rglob('*')) if p.is_file()}
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Native init import: ' + ('PASS' if report['passed'] else report['error']))
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())

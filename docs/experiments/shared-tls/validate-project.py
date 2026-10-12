#!/usr/bin/env python3
"""Build a Raven TLS ownership project and verify interpreter/JIT/AOT parity."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('bundle', 'aot', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    bundle, aot, output = (getattr(args, name).resolve() for name in ('bundle', 'aot', 'output'))
    output.mkdir(parents=True, exist_ok=False)
    windows = platform.system() == 'Windows'
    runtime = bundle / ('bin/neoclr.exe' if windows else 'bin/neoclr')
    report = dict(passed=False, platform=platform.platform(), commands=[])

    def run(command, **kwargs):
        result = subprocess.run(list(map(str, command)), capture_output=True, text=True, timeout=240, **kwargs)
        report['commands'].append(dict(command=list(map(str, command)), exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        if result.returncode:
            raise RuntimeError(result.stdout + result.stderr)
        return result

    try:
        project = output / 'project'
        project.mkdir()
        for path in (HERE / 'project').iterdir():
            if path.suffix in ('.rvn', '.rvnproj'):
                shutil.copy2(path, project / path.name)
        build = output / 'native'
        run([sys.executable, ROOT / 'scripts/build-native-project.py', '--profile', 'windows-console' if windows else 'console',
             '--project', project / 'App.rvnproj', '--bundle', bundle, '--aot', aot, '--output', build])
        evidence = json.loads((build / 'build.json').read_text())
        assert evidence['passed'] and evidence['managedTls']
        assert any(row.get('phase') == 'native-tls-build' for row in evidence['commands'])
        assembly = build / 'app.dll'
        run([runtime, 'run', assembly])  # Default installed JIT.
        catalog = json.loads((bundle / 'lib/bundle.json').read_text())
        context = ['--system', bundle / 'lib' / catalog['runtimeSeed'], '--object-root', bundle / 'lib/System.Runtime.dll']
        for name in catalog['assemblyNames']:
            context += ['--module', bundle / 'lib' / (name + '.dll')]
        run([runtime, 'run', assembly, '--execution', 'interpreter', *context])
        with tempfile.TemporaryDirectory(prefix='neoclr-tls-') as directory:
            executable = Path(directory) / ('app.exe' if windows else 'app')
            shutil.copy2(build / executable.name, executable)
            run([executable], cwd=directory)
            assert list(Path(directory).iterdir()) == [executable]
        # Unused private declarations must not add a TLS dependency.
        (project / 'Main.rvn').write_text('func Main() -> int {\n    return 0\n}\n')
        plain = output / 'plain'
        run([sys.executable, ROOT / 'scripts/build-native-project.py', '--profile', 'windows-console' if windows else 'console',
             '--project', project / 'App.rvnproj', '--bundle', bundle, '--aot', aot, '--output', plain])
        plain_report = json.loads((plain / 'build.json').read_text())
        assert plain_report['passed'] and not plain_report['managedTls']
        assert not any(row.get('phase') == 'native-tls-build' for row in plain_report['commands'])
        run([plain / ('app.exe' if windows else 'app')])
        report.update(passed=True, modes=['interpreter', 'default-jit', 'aot'], standaloneExecution=True, unusedProviderOmitted=True,
                      buildManifestSha256=hashlib.sha256((build / 'build.json').read_bytes()).hexdigest())
    finally:
        (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(dict(passed=True, report=str(output / 'validation.json'))))


if __name__ == '__main__':
    main()

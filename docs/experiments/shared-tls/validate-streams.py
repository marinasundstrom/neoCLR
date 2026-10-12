#!/usr/bin/env python3
"""Build and execute managed SslStream/HTTP JSON on interpreter, default JIT and AOT."""
import argparse
import importlib.util
import json
import hashlib
import time
from pathlib import Path
import platform
import shutil
import subprocess
import sys

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
    report = dict(passed=False, platform=platform.platform(), commands=[])

    def run(command):
        started = time.monotonic()
        result = subprocess.run(list(map(str, command)), capture_output=True, text=True, timeout=900)
        report['commands'].append(dict(command=list(map(str, command)), exitCode=result.returncode,
                                       stdout=result.stdout, stderr=result.stderr,
                                       elapsedSeconds=round(time.monotonic() - started, 6)))
        if result.returncode:
            raise RuntimeError(result.stdout + result.stderr)
        return result

    try:
        spec = importlib.util.spec_from_file_location('tls_bindings', HERE / 'validate-bindings.py')
        helpers = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(helpers)
        cert, key = helpers.create_credentials(output, run)
        project = output / 'project'
        shutil.copytree(HERE / 'stream-project', project)
        fixtures = 'import System.*\n\ninternal static class Fixtures {\n'
        for name, path in [('Certificate', cert), ('Key', key)]:
            fixtures += '    internal static func ' + name + '() -> byte[] {\n        return ['
            fixtures += ', '.join('(byte)' + str(b) for b in path.read_bytes()) + ']\n    }\n'
        fixtures += '}\n'
        (project / 'Fixtures.rvn').write_text(fixtures)
        build = output / 'native'
        run([sys.executable, ROOT / 'scripts/build-native-project.py', '--profile',
             'windows-http' if windows else 'http', '--project', project / 'App.rvnproj',
             '--bundle', bundle, '--aot', aot, '--output', build, '--http-heap-bytes', '8388608', '--command-timeout', '600'])
        evidence = json.loads((build / 'build.json').read_text())
        assert evidence['passed'] and evidence['managedTls']
        runtime = bundle / ('bin/neoclr.exe' if windows else 'bin/neoclr')
        assembly = build / 'app.dll'
        catalog = json.loads((bundle / 'lib/bundle.json').read_text())
        context = ['--system', bundle / 'lib' / catalog['runtimeSeed'], '--object-root', bundle / 'lib/System.Runtime.dll']
        for name in catalog['assemblyNames']:
            context += ['--module', bundle / 'lib' / (name + '.dll')]
        commands = [
            [runtime, 'run', assembly, '--execution', 'interpreter', '--instructions', '100000000', '--array-elements', '1048576', '--array-bytes', '67108864', *context],
            [runtime, 'run', assembly],
            [build / ('app.exe' if windows else 'app')],
        ]
        for command in commands:
            result = run(command)
            assert result.stdout == 'Managed HTTPS JSON exchange passed\n', result.stdout
        report.update(passed=True, modes=['interpreter', 'default-jit', 'aot'])
        report['scope'] = 'Managed lifetime and HTTPS nested-JSON exchange with GC; timings include compilation and are not benchmarks.'
        sources = list((ROOT / 'runtime/raven/src/System/Networking/Security').glob('*.rvn'))
        sources += list((HERE / 'stream-project').glob('*'))
        report['sourceHashes'] = {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest() for path in sources if path.is_file()}
    except Exception as error:
        report['error'] = str(error)
        raise
    finally:
        (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()

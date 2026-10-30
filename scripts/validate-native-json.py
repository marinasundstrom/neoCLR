#!/usr/bin/env python3
"""Qualify public Raven reflection and JSON against the matching interpreter."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tarfile
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('native_http', ROOT / 'scripts/validate-native-http-project.py')
common = importlib.util.module_from_spec(spec)
spec.loader.exec_module(common)
sha = common.sha


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--packaged-tools', action='store_true', help='Use verified runtime/backend roles from --bundle')
    parser.add_argument('--bundle', type=Path, help='Required on macOS; Windows rebuilds the pinned bundle libraries')
    parser.add_argument('--case', choices=['introspection', 'json'], help='Run one focused consumer; default runs both')
    args = parser.parse_args()
    if args.packaged_tools and not args.bundle:
        parser.error('--packaged-tools requires --bundle')
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, platform=platform.platform(), cases=[], commands=[])

    def run(command, name):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, timeout=600)
        (out / (name + '.stdout.log')).write_bytes(result.stdout)
        (out / (name + '.stderr.log')).write_bytes(result.stderr)
        report['commands'].append(dict(command=result.args, exitCode=result.returncode))
        if result.returncode:
            raise RuntimeError(name + ' failed; see retained logs')
        return result

    try:
        windows = platform.system() == 'Windows'
        if not windows and platform.system() != 'Darwin':
            raise ValueError('Requires macOS ARM64 or Windows x64')
        report['revision'] = run(['git', 'rev-parse', 'HEAD'], 'revision').stdout.decode().strip()
        samples = [('introspection', 'native-introspection', 'Report\n'),
                   ('json', 'native-json', '{"Name":"Café","Count":3,"Active":true}\n')]
        if args.case:
            samples = [sample for sample in samples if sample[0] == args.case]
        inputs = [Path(__file__).resolve(), ROOT / 'scripts/build-native-project.py',
                  ROOT / 'scripts/validate-native-http-project.py', ROOT / 'scripts/validate-windows-project.py',
                  ROOT / 'scripts/prepare-native-development-bundle.py']
        for _, sample, _ in samples:
            inputs += [ROOT / 'docs/experiments' / sample / n for n in ('Main.rvn', 'Native.rvnproj')]
        report['inputs'] = {p.relative_to(ROOT).as_posix(): sha(p) for p in inputs}
        if args.bundle:
            bundle = args.bundle.resolve()
        elif windows:
            prereq = out.parent / 'windows-project-prerequisites'
            prereq.mkdir(exist_ok=True)
            archive = prereq / common.windows_project.ARCHIVE
            if not archive.exists():
                urllib.request.urlretrieve(common.windows_project.URL, archive)
            if sha(archive) != common.windows_project.BUNDLE_SHA:
                raise ValueError('Bundle archive hash mismatch')
            bundle = prereq / 'neoclr-native-poc'
            if not bundle.exists():
                with tarfile.open(archive) as source:
                    source.extractall(prereq, filter='data')
            bundle = common.rebuild_libraries(run, bundle, out / 'development toolchain')
        else:
            raise ValueError('--bundle is required on macOS')
        report['bundleManifestSha256'] = sha(bundle / 'manifest.json')
        if args.packaged_tools:
            manifest = json.loads((bundle / 'manifest.json').read_text())
            builder = common.load('json_builder', ROOT / 'scripts/build-native-project.py')
            builder.verify_files(bundle, manifest['files'])
            roles = [manifest['runtime'], manifest['jit']['compiler']]
            if any(role not in manifest['files'] for role in roles):
                raise ValueError('Uncatalogued runtime/backend role')
            runtime, aot = [bundle / role for role in roles]
        else:
            run(['cargo', 'build', '--locked', '--manifest-path', ROOT / 'runtime/codegen/Cargo.toml'], 'aot-build')
            aot = ROOT / ('target/debug/neoclr-codegen' + ('.exe' if windows else ''))
            run(['cargo', 'build', '--locked', '--release', '--bin', 'neoclr'], 'interpreter-build')
            runtime = ROOT / ('target/release/neoclr' + ('.exe' if windows else ''))
        report['aotSha256'] = sha(aot)
        report['interpreterSha256'] = sha(runtime)
        for name, sample, expected in samples:
            project = out / (name + ' project with spaces')
            project.mkdir()
            for file in ('Main.rvn', 'Native.rvnproj'):
                shutil.copy2(ROOT / 'docs/experiments' / sample / file, project / file)
            dest = out / (name + ' native output')
            roots = ['Report'] if name == 'introspection' else ['Report', 'Guarded', 'Bulletin', 'NodeEnvelope', 'GuardedNode', 'CollectionEnvelope']
            # The expanded mapper corpus retains Stream implementations, including
            # socket-backed cleanup. Until selection prunes those unused branches,
            # it needs the full HTTP host bindings even though it does no network I/O.
            profile = ('windows-http' if windows else 'http') if name == 'json' else ('windows-console' if windows else 'console')
            run([sys.executable, ROOT / 'scripts/build-native-project.py', '--profile', profile,
                 '--project', project / 'Native.rvnproj', *[arg for root in roots for arg in ('--reflection-type', root)],
                 '--bundle', bundle, '--aot', aot, '--output', dest], name + '-build')
            build = json.loads((dest / 'build.json').read_text())
            if not build['passed']:
                raise ValueError('Native build failed')
            isolated = out / (name + ' executable only')
            isolated.mkdir()
            exe = isolated / ('app.exe' if windows else 'app')
            shutil.copy2(dest / exe.name, exe)
            lib = bundle / 'lib'
            catalog = json.loads((lib / 'bundle.json').read_text())
            interpreter = [runtime, 'run', dest / 'app.dll', '--execution', 'interpreter',
                           '--system', lib / catalog['runtimeSeed'],
                           *[arg for assembly in catalog['assemblyNames'] for arg in ('--module', lib / (assembly + '.dll'))],
                           '--object-root', lib / 'System.Runtime.dll', '--instructions', '100000000']
            env = {k: v for k, v in os.environ.items() if k.upper() in ('SYSTEMROOT', 'WINDIR', 'TEMP', 'TMP')}
            results = []
            for mode, command, cwd, environment in [('native', [exe], isolated, env), ('interpreter', interpreter, ROOT, None)]:
                result = subprocess.run(list(map(str, command)), cwd=cwd, env=environment, capture_output=True, timeout=600 if mode == 'interpreter' else 90)
                stdout = result.stdout.decode('utf-8').replace('\r\n', '\n')
                stderr = result.stderr.decode('utf-8').replace('\r\n', '\n')
                record = dict(mode=mode, command=list(map(str, command)), exitCode=result.returncode, stdout=stdout, stderr=stderr)
                results.append(record)
                (out / (name + '-' + mode + '.json')).write_text(json.dumps(record, indent=2) + '\n')
                if (result.returncode, stdout, stderr) != (0, expected, ''):
                    raise ValueError(name + ' ' + mode + ' result mismatch')
            report['cases'].append(dict(name=name, passed=True, results=results))
        if sha(runtime) != report['interpreterSha256'] or sha(aot) != report['aotSha256'] or any(sha(ROOT / n) != h for n, h in report['inputs'].items()):
            raise ValueError('Inputs changed during validation')
        report['passed'] = True
    except Exception as error:
        report['error'] = str(error)
    finally:
        report['files'] = {p.relative_to(out).as_posix(): sha(p) for p in sorted(out.rglob('*')) if p.is_file()}
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Native reflection/JSON: ' + ('PASS' if report['passed'] else 'FAIL: ' + report['error']))
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())

#!/usr/bin/env python3
"""Run the bounded Windows x64 AOT gate, retaining compiler, linker and execution evidence."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def require_execution(path):
    data = json.loads(path.read_text())
    if data.get('passed') is not True or len(data.get('outcomes', [])) != 32:
        raise ValueError('Windows C-consumer gate did not complete all 32 native/interpreter comparisons')
    return data


def require_raven_execution(path):
    data = json.loads(path.read_text(encoding='utf-8'))
    outcomes = data.get('outcomes', [])
    if (data.get('passed') is not True or len(outcomes) != 2
            or {case.get('container') for case in outcomes} != {'PE/#Neo', 'NEOX'}
            or any(case.get('exitCode') != 0 or case.get('stdout') != 'Hello, world!\n'
                   or case.get('stderr') != '' for case in outcomes)):
        raise ValueError('Raven Hello World gate did not complete both standalone container executions')
    return data


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--raven-source', type=Path, required=True,
                        help='Checkout of the native-enabled Raven compiler to build')
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    evidence = out / 'cases'
    evidence.mkdir()
    raven = args.raven_source.resolve()
    fresh = out / 'fresh-source'
    fresh.mkdir()
    assembly = fresh / 'RavenHello.dll'
    report = dict(passed=False, scope='Windows x64 scalar/literal-console AOT; no managed profiles or project kit qualification',
                  platform=platform.platform(), machine=platform.machine(), commands=[])

    def run(command, name, cwd=ROOT):
        command = list(map(str, command))
        log = out / (name + '.log')
        with log.open('w', encoding='utf-8') as stream:
            result = subprocess.run(command, cwd=cwd, stdout=stream, stderr=subprocess.STDOUT,
                env={**os.environ, 'NEOCLR_WINDOWS_AOT_EVIDENCE': str(evidence),
                     'NEOCLR_WINDOWS_FRESH_RAVEN': str(assembly)}, timeout=1500)
        report['commands'].append(dict(command=command, exitCode=result.returncode, log=log.name))
        if result.returncode:
            raise RuntimeError(f'{name} failed with exit {result.returncode}; see {log}')
        return log.read_text(encoding='utf-8', errors='replace').strip()

    try:
        if platform.system() != 'Windows' or platform.machine().lower() not in ('amd64', 'x86_64'):
            raise ValueError('Execution qualification requires a Windows x64 host and MSVC developer environment')
        report['revision'] = run(['git', 'rev-parse', 'HEAD'], 'revision')
        report['rustc'] = run(['rustc', '-Vv'], 'rustc')
        report['cargo'] = run(['cargo', '-V'], 'cargo')
        report['cl'] = run(['where.exe', 'cl'], 'cl-location')
        report['compilerRevision'] = run(['git', '-C', raven, 'rev-parse', 'HEAD'], 'raven-revision')
        report['dotnet'] = run(['dotnet', '--info'], 'dotnet')
        # The primitive/console bootstrap needs no Raven.Core or packaged SDK.
        run(['dotnet', 'build', raven / 'src/Raven.Compiler/Raven.Compiler.csproj',
             '-c', 'Release', '-f', 'net10.0', '-p:UseRavenCoreReference=false',
             '-p:NeoClrMetadataProject=' + str(ROOT / 'tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj')],
            'raven-build', cwd=raven)
        compiler = raven / 'src/Raven.Compiler/bin/Release/net10.0/rvnc.dll'
        compiler_files = ('rvnc.dll', 'Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll',
                          'NeoCLR.Metadata.Experimental.dll')
        report['compilerFiles'] = {name: hashlib.sha256((compiler.parent / name).read_bytes()).hexdigest()
                                   for name in compiler_files}
        source = fresh / 'hello.rvn'
        shutil.copyfile(ROOT / 'docs/experiments/aot-hello/hello.rvn', source)
        run(['dotnet', compiler, 'neoclr', '-o', assembly, source], 'raven-compile')
        if not assembly.is_file():
            raise ValueError('Raven compiler did not produce fresh native metadata')
        command = ['cargo', 'test', '--locked', '--manifest-path', 'tools/aot-poc/Cargo.toml',
                   '--test', 'windows_scalar', '--', '--nocapture']
        run(command, 'tests')
        report['execution'] = require_execution(evidence / 'windows-execution.json')
        report['ravenExecution'] = require_raven_execution(evidence / 'raven-execution.json')
        report['freshRavenExecution'] = require_raven_execution(evidence / 'fresh-raven-execution.json')
        report['passed'] = True
    except Exception as error:
        report['error'] = str(error)
    finally:
        report['files'] = {p.relative_to(out).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                           for p in sorted(out.rglob('*')) if p.is_file()}
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
        summary = f"Windows scalar AOT: {'PASS' if report['passed'] else 'FAIL'}\n\nRevision: {report.get('revision', 'unavailable')}\n\n"
        summary += '32 scalar comparisons and 4 standalone Raven Hello World executions (retained and fresh source) passed.\n' if report['passed'] else report.get('error', 'Unknown failure') + '\n'
        if os.environ.get('GITHUB_STEP_SUMMARY'):
            with open(os.environ['GITHUB_STEP_SUMMARY'], 'a', encoding='utf-8') as stream:
                stream.write(summary)
        print(summary)
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    sys.exit(main())

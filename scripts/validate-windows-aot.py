#!/usr/bin/env python3
"""Run the bounded Windows x64 AOT gate, retaining compiler, linker and execution evidence."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def require_execution(path):
    data = json.loads(path.read_text())
    if data.get('passed') is not True or len(data.get('outcomes', [])) != 32:
        raise ValueError('Windows C-consumer gate did not complete all 32 native/interpreter comparisons')
    return data


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    evidence = out / 'cases'
    evidence.mkdir()
    report = dict(passed=False, scope='Windows x64 scalar/literal-console AOT; no managed profiles or project kit qualification',
                  platform=platform.platform(), machine=platform.machine(), commands=[])

    def run(command, name):
        log = out / (name + '.log')
        with log.open('w', encoding='utf-8') as stream:
            result = subprocess.run(command, cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT,
                env={**os.environ, 'NEOCLR_WINDOWS_AOT_EVIDENCE': str(evidence)}, timeout=1500)
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
        command = ['cargo', 'test', '--locked', '--manifest-path', 'tools/aot-poc/Cargo.toml',
                   '--test', 'windows_scalar', '--', '--nocapture']
        run(command, 'tests')
        report['execution'] = require_execution(evidence / 'windows-execution.json')
        report['passed'] = True
    except Exception as error:
        report['error'] = str(error)
    finally:
        report['files'] = {p.relative_to(out).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                           for p in sorted(out.rglob('*')) if p.is_file()}
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
        summary = f"Windows scalar AOT: {'PASS' if report['passed'] else 'FAIL'}\n\nRevision: {report.get('revision', 'unavailable')}\n\n"
        summary += '32 native/interpreter comparisons passed.\n' if report['passed'] else report.get('error', 'Unknown failure') + '\n'
        if os.environ.get('GITHUB_STEP_SUMMARY'):
            with open(os.environ['GITHUB_STEP_SUMMARY'], 'a', encoding='utf-8') as stream:
                stream.write(summary)
        print(summary)
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    sys.exit(main())

#!/usr/bin/env python3
"""Qualify private Windows host memory prerequisites, independently of AOT codegen."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, scope='Private Windows x64 guarded heap; no managed execution or stack qualification',
                  platform=platform.platform(), commands=[])

    def run(command, name):
        command = list(map(str, command))
        result = subprocess.run(command, cwd=out, capture_output=True, timeout=120)
        (out / (name + '.stdout.log')).write_bytes(result.stdout)
        (out / (name + '.stderr.log')).write_bytes(result.stderr)
        report['commands'].append(dict(command=command, exitCode=result.returncode))
        if result.returncode:
            raise RuntimeError(f'{name} failed: see retained logs')
        return result

    try:
        if platform.system() != 'Windows' or platform.machine().lower() not in ('amd64', 'x86_64'):
            raise ValueError('Requires Windows x64 and an MSVC developer shell')
        report['revision'] = run(['git', '-C', ROOT, 'rev-parse', 'HEAD'], 'revision').stdout.decode().strip()
        inputs = [ROOT / 'tools/native' / name for name in
                  ('windows-host-memory.c', 'windows-host-memory.h', 'windows-host-memory-test.c')]
        report['inputs'] = {p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}
        run(['cl', '/nologo', '/W4', '/WX', '/std:c11', '/O2', '/MT', '/Fe:host-memory.exe', inputs[0], inputs[2]], 'build')
        result = run([out / 'host-memory.exe'], 'execute')
        expected = b'Windows guarded heap: 12 allocation lifecycles passed\r\n'
        if result.stdout != expected or result.stderr:
            raise ValueError('Host memory acceptance did not report exact completion')
        report['execution'] = dict(exitCode=0, stdout=result.stdout.decode(), stderr='', allocationLifecycles=12)
        report['passed'] = True
    except Exception as error:
        report['error'] = str(error)
    finally:
        report['files'] = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(out.iterdir()) if p.is_file()}
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print('Windows host memory: ' + ('PASS' if report['passed'] else 'FAIL: ' + report['error']))
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())

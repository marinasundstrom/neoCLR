#!/usr/bin/env python3
"""Run identical private native socket contracts on macOS and Windows."""
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
    report = dict(passed=False, platform=platform.platform(), machine=platform.machine(), commands=[], cases=[])

    def run(command, name):
        command = list(map(str, command))
        result = subprocess.run(command, cwd=out, capture_output=True, timeout=60)
        (out / (name + '.stdout.log')).write_bytes(result.stdout)
        (out / (name + '.stderr.log')).write_bytes(result.stderr)
        report['commands'].append(dict(command=command, exitCode=result.returncode))
        if result.returncode:
            raise RuntimeError(name + ' failed; see retained logs')
        return result

    try:
        windows = platform.system() == 'Windows'
        if not windows and platform.system() != 'Darwin':
            raise ValueError('Qualification requires macOS or Windows')
        report['revision'] = run(['git', '-C', ROOT, 'rev-parse', 'HEAD'], 'revision').stdout.decode().strip()
        base = ROOT / 'docs/experiments/aot-console'
        inputs = [base / name for name in ('socket-os.h', 'socket-listener.h', 'socket-listener.c',
                  'socket-listener-test.c', 'socket-accept-test.c', 'socket-transfer-test.c',
                  'native-gc.c', 'native-gc.h', 'root-probe.c', 'root-probe.h', 'text-arena.c', 'text-arena.h')]
        inputs += [base.parent / 'aot-fault-details/fault-details.h', Path(__file__).resolve()]
        report['inputs'] = {p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}
        if windows:
            compiler = ['cl', '/nologo', '/W4', '/WX', '/std:c11', '/experimental:c11atomics', '/O2', '/MT', '/DNEOCLR_NATIVE_GC']
        else:
            clang = run(['xcrun', '--sdk', 'macosx', '--find', 'clang'], 'clang').stdout.decode().strip()
            sdk = run(['xcrun', '--sdk', 'macosx', '--show-sdk-path'], 'sdk').stdout.decode().strip()
            compiler = [clang, '-isysroot', sdk, '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-DNEOCLR_NATIVE_GC', '-fsanitize=undefined,bounds']
        for name in ('listener', 'accept', 'transfer'):
            binary = out / (name + ('.exe' if windows else ''))
            sources = [base / ('socket-' + name + '-test.c'), base / 'socket-listener.c',
                       base / 'native-gc.c', base / 'root-probe.c', base / 'text-arena.c']
            output = ['/Fe:' + str(binary), '/link', 'Ws2_32.lib'] if windows else ['-o', binary]
            run([*compiler, *sources, *output], name + '-build')
            result = run([binary], name + '-execute')
            if result.stdout or result.stderr:
                raise ValueError('Unexpected contract output: ' + name)
            report['cases'].append(dict(name=name, passed=True))
        report['passed'] = len(report['cases']) == 3
    except Exception as error:
        report['error'] = str(error)
    finally:
        report['files'] = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(out.iterdir()) if p.is_file()}
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print('Native socket contracts: ' + ('PASS' if report['passed'] else 'FAIL: ' + report['error']))
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())

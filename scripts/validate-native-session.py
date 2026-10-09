#!/usr/bin/env python3
"""Qualify private retained-state session ownership and compiled callbacks on both OSes."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[1]

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--aot', type=Path, required=True)
    args = parser.parse_args()
    out = args.output.resolve(); out.mkdir(parents=True, exist_ok=False)
    aot = args.aot.resolve()
    report = dict(passed=False, platform=platform.platform(), machine=platform.machine(), cases=[], commands=[])
    def run(command, name):
        result = subprocess.run(list(map(str, command)), cwd=out, capture_output=True, timeout=120)
        (out / (name + '.stdout')).write_bytes(result.stdout)
        (out / (name + '.stderr')).write_bytes(result.stderr)
        report['commands'].append(dict(command=result.args, exitCode=result.returncode))
        if result.returncode: raise RuntimeError(name + ' failed; see retained logs')
        return result
    try:
        windows = platform.system() == 'Windows'
        if not windows and platform.system() != 'Darwin': raise ValueError('Requires macOS or Windows')
        report['revision'] = run(['git', '-C', ROOT, 'rev-parse', 'HEAD'], 'revision').stdout.decode().strip()
        base = ROOT / 'docs/experiments/aot-console'
        sources = [base / name for name in ('native-session.c', 'native-gc.c', 'root-probe.c')]
        extra = [base / 'text-arena.c', base.parent / 'aot-fault-details/render.c']
        inputs = [*sources, *extra, *[base / n for n in ('native-session.h', 'native-session-test.c', 'native-session-compiled-test.c',
                  'native-gc.h', 'root-probe.h', 'text-arena.h', 'native-stack.h', 'host-callbacks.neoil')],
                  base.parent / 'aot-fault-details/fault-details.h', Path(__file__).resolve()]
        if windows:
            extra += [ROOT / 'tools/native/windows-native-stack.c']
            inputs += [extra[-1]]
            cc = ['cl', '/nologo', '/W4', '/WX', '/std:c11', '/experimental:c11atomics', '/O2', '/MT', '/DNEOCLR_NATIVE_GC', '/I' + str(base)]
        else:
            clang = run(['xcrun', '--find', 'clang'], 'clang').stdout.decode().strip()
            sdk = run(['xcrun', '--sdk', 'macosx', '--show-sdk-path'], 'sdk').stdout.decode().strip()
            cc = [clang, '-isysroot', sdk, '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-DNEOCLR_NATIVE_GC', '-I', base, '-fsanitize=undefined,bounds']
        report['inputs'] = {p.relative_to(ROOT).as_posix(): sha(p) for p in inputs}
        report['aotSha256'] = sha(aot)
        seed = out / 'System.neoil'; seed.write_text('.module System\n.references ()\n')
        helper = out / 'Helpers.neoil'; helper.write_text('.module Helpers\n.references ()\n')
        obj = out / ('guest.obj' if windows else 'guest.o')
        flags = ['--compile-system', '--reference-arena', '--native-gc', '--native-host-bootstrap']
        if windows: flags += ['--target', 'x86_64-pc-windows-msvc', '--windows-console-experiment', '--native-stack-budget']
        run([aot, '--closed-world', base / 'host-callbacks.neoil', 'Calculate', obj, '--system', seed, '--module', helper, *flags], 'guest-build')
        for name in ('kernel', 'compiled'):
            binary = out / (name + ('.exe' if windows else ''))
            test = base / ('native-session-test.c' if name == 'kernel' else 'native-session-compiled-test.c')
            link = ['/Fe:' + str(binary), '/link', '/STACK:1048576'] if windows else ['-o', binary]
            run([*cc, test, *sources, *([*extra, obj] if name == 'compiled' else []), *link], name + '-build')
            result = run([binary], name + '-execute')
            stderr = result.stderr.decode().replace('\r\n', '\n')
            expected = '' if name == 'kernel' else 'DivideByZero: Division by zero\n   at Fail [instruction 2]\n'
            if result.stdout or stderr != expected:
                raise ValueError('Unexpected contract output: ' + name + ': ' + stderr)
            report['cases'].append(dict(name=name, passed=True, stdout=result.stdout.decode(), stderr=stderr))
        if sha(aot) != report['aotSha256'] or any(sha(ROOT / p) != h for p, h in report['inputs'].items()):
            raise ValueError('Inputs changed during validation')
        report['passed'] = len(report['cases']) == 2
    except Exception as error: report['error'] = str(error)
    finally:
        report['files'] = {p.name: sha(p) for p in sorted(out.iterdir()) if p.is_file()}
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Native retained session: ' + ('PASS' if report['passed'] else 'FAIL: ' + report['error']))
    return 0 if report['passed'] else 1

if __name__ == '__main__': raise SystemExit(main())

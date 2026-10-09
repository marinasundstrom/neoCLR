#!/usr/bin/env python3
"""Execute the private native type-token foundation on macOS ARM64 and Windows x64."""
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
    parser.add_argument('--aot', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    aot, out = args.aot.resolve(), args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, platform=platform.platform(), machine=platform.machine(), commands=[])
    def run(command, name):
        r = subprocess.run(list(map(str, command)), cwd=out, capture_output=True, timeout=120)
        (out / (name + '.stdout')).write_bytes(r.stdout)
        (out / (name + '.stderr')).write_bytes(r.stderr)
        report['commands'].append(dict(command=r.args, exitCode=r.returncode))
        if r.returncode:
            raise RuntimeError(name + ' failed; see logs')
        return r
    try:
        windows = platform.system() == 'Windows'
        if not windows and platform.system() != 'Darwin':
            raise ValueError('Requires macOS ARM64 or Windows x64')
        report['revision'] = run(['git', '-C', ROOT, 'rev-parse', 'HEAD'], 'revision').stdout.decode().strip()
        base = ROOT / 'docs/experiments/aot-console'
        sources = [base / n for n in ('type-tokens-test.c', 'native-gc.c', 'root-probe.c', 'text-arena.c')]
        inputs = [*sources, base / 'type-tokens.neoil', *[base / n for n in ('native-gc.h', 'root-probe.h', 'text-arena.h', 'native-stack.h')],
                  base.parent / 'aot-fault-details/fault-details.h', Path(__file__).resolve()]
        if windows:
            sources += [ROOT / 'tools/native/windows-native-stack.c']
            inputs += [sources[-1]]
            cc = ['cl', '/nologo', '/W4', '/WX', '/std:c11', '/experimental:c11atomics', '/O2', '/MT', '/DNEOCLR_NATIVE_GC', '/I' + str(base)]
        else:
            clang = run(['xcrun', '--find', 'clang'], 'clang').stdout.decode().strip()
            sdk = run(['xcrun', '--sdk', 'macosx', '--show-sdk-path'], 'sdk').stdout.decode().strip()
            cc = [clang, '-isysroot', sdk, '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-DNEOCLR_NATIVE_GC', '-I', base, '-fsanitize=undefined,bounds']
        report['inputs'] = {p.relative_to(ROOT).as_posix(): sha(p) for p in inputs}
        report['aotSha256'] = sha(aot)
        seed, helper = out / 'System.neoil', out / 'Helpers.neoil'
        seed.write_text('.module System\n.references ()\n')
        helper.write_text('.module Helpers\n.references ()\n')
        obj = out / ('guest.obj' if windows else 'guest.o')
        flags = ['--compile-system', '--reference-arena', '--native-gc']
        if windows:
            flags += ['--target', 'x86_64-pc-windows-msvc', '--windows-console-experiment', '--native-stack-budget']
        run([aot, '--closed-world', base / 'type-tokens.neoil', 'Calculate', obj, '--system', seed, '--module', helper, *flags], 'guest-build')
        exe = out / ('host.exe' if windows else 'host')
        link = ['/Fe:' + str(exe), '/link', '/STACK:1048576'] if windows else ['-o', exe]
        run([*cc, *sources, obj, *link], 'host-build')
        r = run([exe], 'execute')
        if r.stdout.replace(b'\r\n', b'\n') != b'Type tokens: 42\n' or r.stderr:
            raise ValueError('Unexpected token identity result')
        if sha(aot) != report['aotSha256'] or any(sha(ROOT / n) != h for n, h in report['inputs'].items()):
            raise ValueError('Inputs changed during validation')
        report.update(passed=True, stdout='Type tokens: 42\n', stderr='')
    except Exception as error:
        report['error'] = str(error)
    finally:
        report['files'] = {p.name: sha(p) for p in sorted(out.iterdir()) if p.is_file()}
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Native type tokens: ' + ('PASS' if report['passed'] else 'FAIL: ' + report['error']))
    return 0 if report['passed'] else 1

if __name__ == '__main__':
    raise SystemExit(main())

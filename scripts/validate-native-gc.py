#!/usr/bin/env python3
"""Qualify shared native GC and root validation on macOS and Windows."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import signal
import subprocess

ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / 'docs/experiments/aot-console'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    windows = platform.system() == 'Windows'
    report = dict(passed=False, platform=platform.platform(), machine=platform.machine(), commands=[])

    def run(label, command, expected=0, timeout=90):
        command = list(map(str, command))
        result = subprocess.run(command, cwd=out, capture_output=True, timeout=timeout)
        (out / (label + '.stdout.log')).write_bytes(result.stdout)
        (out / (label + '.stderr.log')).write_bytes(result.stderr)
        report['commands'].append(dict(label=label, command=command, exitCode=result.returncode, expected=expected))
        if result.returncode != expected:
            raise RuntimeError(label + ' failed; see retained logs')
        return result.stdout.decode().strip()

    try:
        report['revision'] = run('revision', ['git', '-C', ROOT, 'rev-parse', 'HEAD'])
        if windows:
            cc = ['cl', '/nologo', '/W4', '/WX', '/std:c11', '/experimental:c11atomics', '/O2', '/MT']
        elif platform.system() == 'Darwin':
            clang = run('clang', ['xcrun', '--sdk', 'macosx', '--find', 'clang'])
            sdk = run('sdk', ['xcrun', '--sdk', 'macosx', '--show-sdk-path'])
            cc = [clang, '-isysroot', sdk, '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-fsanitize=undefined,bounds']
        else:
            raise ValueError('Requires Windows or macOS')
        names = ['native-gc-test', 'native-gc-intern-test', 'native-gc-base-test', 'native-gc-format-test', 'native-gc-primitive-test', 'native-gc-reference-test', 'native-gc-borrowed-test', 'native-gc-utf8-test', 'native-gc-unicode-test', 'native-gc-arguments-test', 'native-gc-producer-test', 'native-gc-preference-test', 'native-gc-admission-test', 'native-gc-allocation-test', 'native-gc-helper-test', 'native-gc-text-helper-test', 'native-gc-index-test', 'native-gc-dense-test', 'native-gc-pass-test', 'native-gc-profile-test']
        text_crate = ROOT / 'tools/aot-native-text'
        run('unicode-library', ['cargo', 'build', '--locked', '--release', '--features', 'time-zone-rules', '--manifest-path', text_crate / 'Cargo.toml'], timeout=300)
        text_library = text_crate / 'target/release' / ('neoclr_aot_native_text.lib' if windows else 'libneoclr_aot_native_text.a')
        inputs = [BASE / 'system-culture.c', BASE / 'time-zone.c', BASE / 'file-input.c', ROOT / 'tools/native/entry-arguments.c', BASE / 'string-unicode.c', text_crate / 'Cargo.toml', text_crate / 'Cargo.lock', *sorted((text_crate / 'src').glob('*.rs')), *sorted((ROOT / 'runtime/native').rglob('*.[ch]')), Path(__file__).resolve(), *BASE.glob('*.h'), BASE / 'native-gc.c', BASE / 'root-probe.c', BASE / 'text-arena.c',
                  BASE / 'root-validation-test.c', BASE.parent / 'aot-fault-details/fault-details.h',
                  *[BASE / (name + '.c') for name in names]]
        report['inputs'] = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}

        def build(name, source, gc=True, profile=False):
            binary = out / (name + ('.exe' if windows else ''))
            defines = (['NEOCLR_NATIVE_GC'] if gc else []) + (['NEOCLR_GC_PROFILE'] if profile else [])
            sources = [BASE / (source + '.c'), BASE / 'root-probe.c']
            if gc:
                sources.append(BASE / 'native-gc.c')
            if source in ['native-gc-text-helper-test', 'native-gc-format-test', 'native-gc-primitive-test', 'native-gc-reference-test', 'native-gc-borrowed-test', 'native-gc-utf8-test']:
                sources.append(BASE / 'text-arena.c')
            if source == 'native-gc-preference-test':
                sources.extend([BASE / 'text-arena.c', BASE / 'system-culture.c', BASE / 'time-zone.c', text_library])
                if not windows:
                    sources.extend(['-framework', 'CoreFoundation'])
            if source == 'native-gc-producer-test':
                sources.append(BASE / 'text-arena.c')
            if source == 'native-gc-arguments-test':
                sources.extend([BASE / 'text-arena.c', ROOT / 'tools/native/entry-arguments.c'])
            if source == 'native-gc-unicode-test':
                sources.extend([BASE / 'text-arena.c', BASE / 'string-unicode.c', text_library])
            run(name + '-build', [*cc, *(['/I' + str(BASE)] if windows else ['-I', BASE]), *[('/D' if windows else '-D') + value for value in defines],
                *sources, *(['/Fe:' + str(binary)] if windows else ['-o', binary])])
            return binary

        for name in names:
            run(name, [build(name, name)])
        run('profile-enabled', [build('profile-enabled', 'native-gc-profile-test', profile=True)])
        for gc in [False, True]:
            name = 'root-validation-' + ('gc' if gc else 'probe')
            binary = build(name, 'root-validation-test', gc=gc)
            for case in ['address', 'width', 'flags', 'fault', 'transient']:
                run(name + '-' + case, [binary, case], 3 if windows else -signal.SIGABRT)
        report['passed'] = True
    except Exception as error:
        report['error'] = str(error)
    finally:
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'passed': report['passed'], 'error': report.get('error')}))
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())

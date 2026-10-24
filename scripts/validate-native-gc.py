#!/usr/bin/env python3
"""Qualify shared native GC and root validation on macOS and Windows."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import signal
import subprocess
import sys

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
            # Surface compiler diagnostics in CI without downloading the SDK archive.
            # Full streams remain in the retained logs.
            for name, data in [('stdout', result.stdout), ('stderr', result.stderr)]:
                if data:
                    print(label + ' ' + name + ':\n' + data[-16384:].decode(errors='replace'), file=sys.stderr)
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
        names = ['native-gc-state-test', 'native-gc-trigger-test', 'native-gc-scan-test', 'native-gc-slot-map-test', 'native-gc-owned-chain-test', 'native-gc-free-index-test', 'native-gc-owned-test', 'native-gc-pacing-test', 'native-gc-test', 'native-gc-intern-test', 'native-gc-object-test', 'native-gc-base-test', 'native-gc-format-test', 'native-gc-primitive-test', 'native-gc-reference-test', 'native-gc-borrowed-test', 'native-gc-utf8-test', 'native-gc-unicode-test', 'native-gc-arguments-test', 'native-gc-producer-test', 'native-gc-preference-test', 'native-gc-admission-test', 'native-gc-allocation-test', 'native-gc-helper-test', 'native-gc-text-helper-test', 'native-gc-index-test', 'native-gc-dense-test', 'native-gc-pass-test', 'native-gc-seed-test', 'native-gc-profile-test', 'native-gc-lookup-profile-test']
        text_crate = ROOT / 'tools/aot-native-text'
        run('unicode-library', ['cargo', 'build', '--locked', '--release', '--features', 'time-zone-rules', '--manifest-path', text_crate / 'Cargo.toml'], timeout=300)
        text_library = text_crate / 'target/release' / ('neoclr_aot_native_text.lib' if windows else 'libneoclr_aot_native_text.a')
        inputs = [BASE / 'system-culture.c', BASE / 'time-zone.c', BASE / 'file-input.c', ROOT / 'tools/native/entry-arguments.c', BASE / 'string-unicode.c', text_crate / 'Cargo.toml', text_crate / 'Cargo.lock', *sorted((text_crate / 'src').glob('*.rs')), *sorted((ROOT / 'runtime/native').rglob('*.[ch]')), Path(__file__).resolve(), *BASE.glob('*.h'), BASE / 'native-gc.c', BASE / 'root-probe.c', BASE / 'text-arena.c',
                  BASE / 'root-validation-test.c', BASE.parent / 'aot-fault-details/fault-details.h',
                  *[BASE / (name + '.c') for name in names]]
        carrier = ROOT / 'docs/experiments/runtime-async'
        inputs += [BASE / 'native-stack.c', ROOT / 'runtime/native/memory/static-storage-test.c']
        inputs += [carrier / name for name in ['native-runtime.c', 'native-runtime.h', 'native-owned-body-test.c', 'native-idle-drain-test.c']]
        report['inputs'] = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}

        def build(name, source, gc=True, profile=False):
            binary = out / (name + ('.exe' if windows else ''))
            defines = (['NEOCLR_NATIVE_GC'] if gc else []) + (['NEOCLR_GC_PROFILE'] if profile else [])
            threaded_pacing = source in ['native-gc-state-test', 'native-gc-pacing-test', 'native-gc-lookup-profile-test', 'native-gc-trigger-test'] and not windows
            if source in ['native-gc-pacing-test', 'native-gc-trigger-test']: defines += ['NEOCLR_GC_PACED']
            if threaded_pacing: defines += ['NEOCLR_TEST_THREADS']
            sources = [BASE / (source + '.c'), BASE / 'root-probe.c']
            if gc:
                sources.append(BASE / 'native-gc.c')
            if source in ['native-gc-pacing-test', 'native-gc-object-test', 'native-gc-text-helper-test', 'native-gc-format-test', 'native-gc-primitive-test', 'native-gc-reference-test', 'native-gc-borrowed-test', 'native-gc-utf8-test']:
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
                *(['-pthread'] if threaded_pacing else []),
                *sources, *(['/Fe:' + str(binary)] if windows else ['-o', binary])])
            return binary

        for name in names:
            run(name, [build(name, name)])
        run('trigger-profile-enabled', [build('trigger-profile-enabled', 'native-gc-trigger-test', profile=True)])
        run('profile-enabled', [build('profile-enabled', 'native-gc-profile-test', profile=True)])
        run('lookup-profile-enabled', [build('lookup-profile-enabled', 'native-gc-lookup-profile-test', profile=True)])
        run('owned-profile-enabled', [build('owned-profile-enabled', 'native-gc-owned-test', profile=True)])
        for gc in [False, True]:
            name = 'root-validation-' + ('gc' if gc else 'probe')
            binary = build(name, 'root-validation-test', gc=gc)
            for case in ['address', 'width', 'flags', 'fault', 'transient', 'plan-missing', 'plan-hint-missing', 'plan-length', 'plan-short', 'batch-address', 'batch-width', 'batch-flags', 'batch-transient', 'context-mutated', 'context-null', 'context-ancestor-null']:
                run(name + '-' + case, [binary, case], 3 if windows else -signal.SIGABRT)
            run(name + '-plan-moved', [binary, 'plan-moved'])
            run(name + '-batch-valid', [binary, 'batch-valid'])
            run(name + '-context-valid', [binary, 'context-valid'])
            run(name + '-state-valid', [binary, 'state-valid'])
            if gc:
                for case in ['owned-entry-invalid', 'owned-mixed-invalid', 'owned-transient-invalid']:
                    run(name + '-' + case, [binary, case], 3 if windows else -signal.SIGABRT)
                run(name + '-owned-valid', [binary, 'owned-valid'])
            for position in ['vector', 'tail']:
                for width in [0, 1, 3, 4, 5, 7, 8, 9, 12, 16, 2147483648, 4294967292, 4294967295]:
                    case = f'width-{position}-{width}'
                    expected = 0 if width in [4, 8] else (3 if windows else -signal.SIGABRT)
                    run(name + '-' + case, [binary, case], expected)
        for name in ['native-owned-body-test', 'native-idle-drain-test']:
            binary = out / (name + ('.exe' if windows else ''))
            flags = ['/DNEOCLR_NATIVE_GC', '/I' + str(BASE), '/I' + str(carrier)] if windows else ['-DNEOCLR_NATIVE_GC', '-Wno-deprecated-declarations', '-I', BASE, '-I', carrier]
            stack = ROOT / 'tools/native/windows-native-stack.c' if windows else BASE / 'native-stack.c'
            run(name + '-build', [*cc, *flags, carrier / (name + '.c'), carrier / 'native-runtime.c',
                BASE / 'native-gc.c', BASE / 'root-probe.c', stack, *(['/Fe:' + str(binary)] if windows else ['-o', binary])])
            run(name, [binary])
            if windows and name == 'native-owned-body-test':
                run(name + '-host-fiber', [binary, '--host-fiber'])
        for fixture in ['static-storage', 'memory-views', 'storage-initialization']:
            for gc in [False, True]:
                name = fixture + '-' + ('gc' if gc else 'arena')
                binary = out / (name + ('.exe' if windows else ''))
                flags = (['/DNEOCLR_NATIVE_GC'] if windows else ['-DNEOCLR_NATIVE_GC']) if gc else []
                sources = [ROOT / 'runtime/native/memory' / (fixture + '-test.c'), BASE / 'text-arena.c']
                if gc:
                    sources += [BASE / 'native-gc.c', BASE / 'root-probe.c']
                run(name + '-build', [*cc, *flags, *sources, *(['/Fe:' + str(binary)] if windows else ['-o', binary])])
                run(name, [binary])
        report['passed'] = True
    except Exception as error:
        report['error'] = str(error)
    finally:
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'passed': report['passed'], 'error': report.get('error')}))
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())

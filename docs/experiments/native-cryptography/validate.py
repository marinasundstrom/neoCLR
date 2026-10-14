#!/usr/bin/env python3
"""Identical IL cases through interpreter, JIT and AOT; macOS ARM64/Windows x64."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent


def literal(name, data):
    text = f'.local arrayref<Byte> {name}\nldc.i4 {len(data)}\nnewarr Byte\nstloc {name}\n'
    for i, value in enumerate(data):
        text += f'ldloc {name}\nldc.i4 {i}\nldc.i4 {value}\nconv.u1\nstelem Byte\n'
    return text


def check(name, data):
    text = ''
    for i, value in enumerate(data):
        text += f'ldloc {name}\nldc.i4 {i}\nldelem Byte\nldc.i4 {value}\nconv.u1\nbeq ok_{name}_{i}\nfault "wrong digest"\nok_{name}_{i}:\n'
    return text


def cases():
    sha = 'call neoCLR.Runtime.CryptoSha256(arrayref<Byte>,arrayref<Byte>)\n'
    mac = 'call neoCLR.Runtime.CryptoHmacSha256(arrayref<Byte>,arrayref<Byte>,arrayref<Byte>)\n'
    verify = 'call neoCLR.Runtime.CryptoHmacSha256Verify(arrayref<Byte>,arrayref<Byte>,arrayref<Byte>)\n'
    for name, data in [('empty', b''), ('abc', b'abc'), ('multiblock', b'a' * 100)]:
        yield 'sha-' + name, literal('data', data) + literal('output', [0xa5] * 33) + 'ldloc data\nldloc output\n' + sha + 'pop\n' + check('output', hashlib.sha256(data).digest() + b'\xa5'), False
    yield 'sha-alias', literal('data', [0] * 32) + 'ldloc data\nldloc data\n' + sha + 'pop\n' + check('data', hashlib.sha256(bytes(32)).digest()), False
    setup = literal('key', [0x0b] * 20) + literal('data', b'Hi There') + literal('output', [0] * 32)
    expected = bytes.fromhex('b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7')
    signed = setup + 'ldloc key\nldloc data\nldloc output\n' + mac + 'pop\n' + check('output', expected)
    yield 'hmac-valid', signed + 'ldloc key\nldloc data\nldloc output\n' + verify + 'brtrue verified\nfault "MAC verification failed"\nverified:\n', False
    yield 'hmac-tampered', signed + 'ldloc output\nldc.i4 0\nldc.i4 0\nconv.u1\nstelem Byte\nldloc key\nldloc data\nldloc output\n' + verify + 'brfalse rejected\nfault "tampered MAC accepted"\nrejected:\n', False
    for size in [0, 32]:
        yield f'random-{size}', literal('output', [0] * size) + 'ldloc output\ncall neoCLR.Runtime.CryptoRandomFill(arrayref<Byte>)\nldc.i4 0\nbeq filled\nfault "random failed"\nfilled:\n', False
    yield 'short-output', literal('data', b'abc') + literal('output', [0xa5] * 31) + 'ldloc data\nldloc output\n' + sha + 'ldc.i4 -1\nbeq short\nfault "wrong short result"\nshort:\n' + check('output', [0xa5] * 31), False
    reserved = '.local arrayref<Byte> output\nldc.i4 32\narray.reserve Byte\nstloc output\n'
    yield 'reserved-output', literal('data', b'abc') + reserved + 'ldloc data\nldloc output\n' + sha + 'pop\n' + check('output', hashlib.sha256(b'abc').digest()), False
    for count in [1, 65537]:
        data = f'.local arrayref<Byte> data\nldc.i4 {count}\narray.reserve Byte\nstloc data\n'
        yield f'uninitialized-{count}', data + literal('output', [0] * 32) + 'ldloc data\nldloc output\n' + sha + 'pop\n', True


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--runtime', type=Path, required=True)
    p.add_argument('--compiler', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    windows = platform.system() == 'Windows'
    if not ((windows and platform.machine().lower() in ('amd64', 'x86_64')) or
            (platform.system() == 'Darwin' and platform.machine() == 'arm64')):
        p.error('requires macOS ARM64 or Windows x64')
    out = a.output.resolve(); out.mkdir(parents=True, exist_ok=True)
    runtime, compiler = a.runtime.resolve(), a.compiler.resolve()
    report = dict(passed=False, platform=platform.platform(), commands=[], cases=[])
    def run(command, expected=0):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True, timeout=180)
        report['commands'].append(dict(command=list(map(str, command)), exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        if result.returncode != expected:
            raise RuntimeError(f'{command[0]} exited {result.returncode}, expected {expected}: {result.stderr}')
        return result.stdout
    try:
        manifest = ROOT / 'tools/native-crypto/Cargo.toml'
        metadata = json.loads(run(['cargo', 'metadata', '--locked', '--format-version', '1', '--manifest-path', manifest]))
        packages = sorted(package['name'] for package in metadata['packages'])
        if any(name.startswith('rustls') or name == 'neoclr-native-tls' for name in packages):
            raise RuntimeError('standalone cryptography unexpectedly depends on TLS')
        report['providerPackages'] = packages
        run(['cargo', 'build', '--locked', '--release', '--manifest-path', manifest])
        archive = ROOT / 'tools/native-crypto/target/release' / ('neoclr_native_crypto.lib' if windows else 'libneoclr_native_crypto.a')
        report['providerArchive'] = dict(path=str(archive.relative_to(ROOT)), sha256=hashlib.sha256(archive.read_bytes()).hexdigest())
        base = ROOT / 'docs/experiments/aot-console'
        sources = [HERE / 'host.c', base / 'cryptography.c', base / 'text-arena.c', base / 'native-gc.c', base / 'root-probe.c']
        sources.append(ROOT / 'tools/native/windows-native-stack.c' if windows else base / 'native-stack.c')
        host = out / ('host.dll' if windows else 'host.dylib')
        host.unlink(missing_ok=True)
        if windows:
            cc = ['cl', '/nologo', '/W4', '/WX', '/std:c11', '/experimental:c11atomics', '/O2', '/MT', '/DNEOCLR_NATIVE_GC', '/DNEOCLR_NATIVE_BYTE_ARRAY_LIMIT=131072', '/I'+str(base), '/Fo'+str(out)+'/', *sources]
            libs = [archive, '/link', 'Bcrypt.lib', 'Ntdll.lib', 'Userenv.lib', 'Advapi32.lib', 'Ws2_32.lib']
            run([*cc, '/LD', '/Fe:'+str(host), *libs, '/EXPORT:neoclr_gc_callback_read_v1', '/EXPORT:neoclr_gc_enter_v1', '/EXPORT:neoclr_gc_entry_check_v1', '/EXPORT:neoclr_gc_leave_v1', '/EXPORT:neoclr_gc_stack_roots_v1', '/EXPORT:neoclr_gc_transient_v1', '/EXPORT:neoclr_native_stack_check_v1', '/EXPORT:neoclr_jit_run_v1', '/EXPORT:neoclr_cryptography_v1', '/EXPORT:neoclr_allocate_bytes_v1', '/EXPORT:neoclr_reserve_bytes_v1', '/EXPORT:neoclr_check_bytes_initialized_v1'])
        else:
            cc = [os.environ.get('CC', 'clang'), '-isysroot', os.environ.get('SDKROOT') or run(['xcrun','--show-sdk-path']).strip(), '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-DNEOCLR_NATIVE_GC', '-DNEOCLR_NATIVE_BYTE_ARRAY_LIMIT=131072', '-I', base, *sources]
            libs = [archive]
            run([*cc, '-dynamiclib', *libs, '-o', host])
            run(['codesign', '--force', '--sign', '-', host])
        seed = out / 'System.neoil'; seed.write_text('.module System\n.references ()\n' + (ROOT / 'runtime/neoCLR/Runtime/Cryptography.neoil').read_text())
        seed_binary = out / 'System.neox'
        seed_binary.unlink(missing_ok=True)
        run([runtime, 'assemble', seed, seed_binary, '--format', 'neox'])
        dependency = out / 'Helpers.neoil'; dependency.write_text('.module Helpers\n.references ()\n')
        flags = ['--system', seed_binary, '--module', dependency, '--compile-system', '--reference-arena', '--bind-cryptography', '--native-gc', '--native-stack-budget']
        if windows: flags += ['--target', 'x86_64-pc-windows-msvc', '--windows-console-experiment']
        for name, body, fault in cases():
            locals_ = [line for line in body.splitlines() if line.startswith('.local ')]
            instructions = [line for line in body.splitlines() if not line.startswith('.local ')]
            app = out / (name + '.neoil')
            app.write_text('.module App\n.entry Main\n.function Main() -> Int32\n' + '\n'.join(locals_ + instructions) + '\nldc.i4 0\nret\n.end\n')
            run([runtime, 'run', app, '--execution', 'interpreter', '--system', seed_binary, '--array-elements', '131072', '--array-bytes', '67108864'], expected=1 if fault else 0)
            binary = out / (name + '.neox')
            binary.unlink(missing_ok=True)
            run([runtime, 'assemble', app, binary, '--format', 'neox', '--system', seed_binary])
            if fault and 'RuntimeError' not in report['commands'][-2]['stderr']:
                raise RuntimeError('interpreter did not report the expected terminal RuntimeError')
            obj = out / (name + ('.obj' if windows else '.o'))
            obj.unlink(missing_ok=True)
            if name == 'sha-empty':
                run([compiler, '--closed-world', binary, 'Main', obj,
                     *[flag for flag in flags if flag != '--bind-cryptography']], expected=1)
                if obj.exists(): raise RuntimeError('unbound service published an object')
            selection = json.loads(run([compiler, '--closed-world', binary, 'Main', obj, *flags]))
            assert any(row['implementation'] == 'cryptography-v1' for row in selection['nativeBindings'])
            executable = out / (name + ('.exe' if windows else '.native'))
            executable.unlink(missing_ok=True)
            run([*cc, '/DNEOCLR_CRYPTO_AOT', obj, '/Fe:'+str(executable), *libs] if windows else [*cc, '-DNEOCLR_CRYPTO_AOT', obj, *libs, '-o', executable])
            if not windows: run(['codesign', '--force', '--sign', '-', executable])
            run([executable], expected=103 if fault else 0)
            run([compiler, '--jit-run', '--closed-world', binary, 'Main', host, *flags], expected=103 if fault else 0)
            jit = dict(guestExitCode=103 if fault else 0)
            report['cases'].append(dict(name=name, interpreter=True, aot=True, jit=jit))
        report['passed'] = True
    finally:
        (out / 'validation.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(dict(passed=True, cases=len(report['cases']), report=str(out / 'validation.json'))))


if __name__ == '__main__':
    main()

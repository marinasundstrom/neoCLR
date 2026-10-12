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


def cases(cert, key):
    create = 'call neoCLR.Runtime.TlsCreateClient(arrayref<Byte>,arrayref<Byte>,arrayref<Byte>)\n'
    server = 'call neoCLR.Runtime.TlsCreateServer(arrayref<Byte>,arrayref<Byte>,arrayref<Byte>,arrayref<Byte>)\n'
    session = 'call neoCLR.Runtime.TlsCreateSession(arrayref<Byte>,arrayref<Byte>,arrayref<Byte>)\n'
    release = 'call neoCLR.Runtime.TlsRelease(arrayref<Byte>)\n'
    abort = 'call neoCLR.Runtime.TlsAbort(arrayref<Byte>)\n'
    def expect(value, label):
        return f'ldc.i4 {value}\nbeq {label}\nfault "wrong TLS status"\n{label}:\n'
    empty = literal('owner', b'') + literal('empty', b'')
    yield 'invalid-trust', empty + 'ldloc owner\nldloc empty\nldloc empty\n' + create + expect(-14, 'rejected'), False
    yield 'release-empty', empty + 'ldloc owner\n' + release + expect(0, 'released') + 'ldloc owner\n' + release + expect(0, 'again'), False
    yield 'abort-empty', empty + 'ldloc owner\n' + abort + expect(-18, 'absent'), False
    yield 'invalid-owner', literal('owner', b'x') + 'ldloc owner\n' + release + 'pop\n', True
    reserved = '.local arrayref<Byte> reserved\nldc.i4 1\narray.reserve Byte\nstloc reserved\n'
    yield 'uninitialized', empty + reserved + 'ldloc owner\nldloc reserved\nldloc empty\n' + create + 'pop\n', True
    for role in ['client', 'server']:
        body = empty + literal('cert', cert) + literal('session', b'') + literal('name', b'localhost' if role == 'client' else b'')
        if role == 'server':
            body += literal('key', key) + 'ldloc owner\nldloc cert\nldloc key\nldloc empty\n' + server
        else:
            body += 'ldloc owner\nldloc cert\nldloc empty\n' + create
        body += expect(0, 'configured')
        body += 'ldloc session\nldloc owner\nldloc name\n' + session + expect(0, 'created')
        body += 'ldloc owner\n' + release + expect(0, 'config_released')
        body += 'ldloc session\n' + abort + expect(0, 'aborted')
        # Deliberately leave a live session for collector/host teardown.
        yield role + '-session', body, False


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
        manifest = ROOT / 'tools/native-tls/Cargo.toml'
        run(['cargo', 'build', '--locked', '--release', '--manifest-path', manifest])
        metadata = json.loads(run(['cargo', 'metadata', '--no-deps', '--format-version', '1', '--manifest-path', manifest]))
        archive = Path(metadata['target_directory']) / 'release' / ('neoclr_native_tls.lib' if windows else 'libneoclr_native_tls.a')
        cert_path, key_path = out / 'cert.pem', out / 'key.pem'
        openssl_config = out / 'openssl.cnf'
        openssl_config.write_text('[req]\nprompt=no\ndistinguished_name=dn\n[dn]\nCN=localhost\n')
        run(['openssl', 'req', '-x509', '-newkey', 'ec', '-pkeyopt', 'ec_paramgen_curve:prime256v1', '-pkeyopt', 'ec_param_enc:named_curve',
             '-nodes', '-sha256', '-days', '1', '-config', openssl_config,
             '-addext', 'basicConstraints=critical,CA:FALSE',
             '-addext', 'keyUsage=critical,digitalSignature',
             '-addext', 'extendedKeyUsage=serverAuth', '-addext', 'subjectAltName=DNS:localhost',
             '-keyout', key_path, '-out', cert_path])
        os.chmod(key_path, 0o600)
        base = ROOT / 'docs/experiments/aot-console'
        sources = [HERE / 'host.c', base / 'tls-managed.c', base / 'text-arena.c', base / 'native-gc.c', base / 'root-probe.c']
        sources.append(ROOT / 'tools/native/windows-native-stack.c' if windows else base / 'native-stack.c')
        host = out / ('host.dll' if windows else 'host.dylib')
        host.unlink(missing_ok=True)
        if windows:
            cc = ['cl', '/nologo', '/W4', '/WX', '/std:c11', '/experimental:c11atomics', '/O2', '/MT', '/DNEOCLR_NATIVE_GC', '/DNEOCLR_NATIVE_BYTE_ARRAY_LIMIT=131072', '/I'+str(base), '/Fo'+str(out)+'/', *sources]
            libs = [archive, '/link', 'Bcrypt.lib', 'Ntdll.lib', 'Userenv.lib', 'Advapi32.lib', 'Ws2_32.lib']
            run([*cc, '/LD', '/Fe:'+str(host), *libs, '/EXPORT:neoclr_gc_callback_read_v1', '/EXPORT:neoclr_gc_enter_v1', '/EXPORT:neoclr_gc_entry_check_v1', '/EXPORT:neoclr_gc_leave_v1', '/EXPORT:neoclr_gc_stack_roots_v1', '/EXPORT:neoclr_gc_transient_v1', '/EXPORT:neoclr_native_stack_check_v1', '/EXPORT:neoclr_jit_run_v1', '/EXPORT:neoclr_tls_managed_v1', '/EXPORT:neoclr_allocate_bytes_v1', '/EXPORT:neoclr_reserve_bytes_v1', '/EXPORT:neoclr_probe_enter_v1', '/EXPORT:neoclr_probe_leave_v1', '/EXPORT:neoclr_probe_stack_roots_v2', '/EXPORT:neoclr_check_bytes_initialized_v1'])
        else:
            cc = [os.environ.get('CC', 'clang'), '-isysroot', os.environ.get('SDKROOT') or run(['xcrun','--show-sdk-path']).strip(), '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-DNEOCLR_NATIVE_GC', '-DNEOCLR_NATIVE_BYTE_ARRAY_LIMIT=131072', '-I', base, *sources]
            libs = [archive]
            run([*cc, '-dynamiclib', *libs, '-o', host])
            run(['codesign', '--force', '--sign', '-', host])
        seed = out / 'System.neoil'; seed.write_text('.module System\n.references ()\n' + (ROOT / 'runtime/neoCLR/Runtime/Tls.neoil').read_text())
        seed_binary = out / 'System.neox'
        seed_binary.unlink(missing_ok=True)
        run([runtime, 'assemble', seed, seed_binary, '--format', 'neox'])
        dependency = out / 'Helpers.neoil'; dependency.write_text('.module Helpers\n.references ()\n')
        flags = ['--system', seed_binary, '--module', dependency, '--compile-system', '--reference-arena', '--bind-tls', '--native-gc', '--native-stack-budget']
        if windows: flags += ['--target', 'x86_64-pc-windows-msvc', '--windows-console-experiment']
        for name, body, fault in cases(cert_path.read_bytes(), key_path.read_bytes()):
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
            if name == 'invalid-trust':
                run([compiler, '--closed-world', binary, 'Main', obj,
                     *[flag for flag in flags if flag != '--bind-tls']], expected=1)
                if obj.exists(): raise RuntimeError('unbound service published an object')
                for omitted in [('--native-gc', '--native-stack-budget'), ('--reference-arena',)]:
                    output = run([compiler, '--closed-world', binary, 'Main', obj,
                                  *[flag for flag in flags if flag not in omitted]], expected=1)
                    if 'requires --reference-arena' not in report['commands'][-1]['stderr']:
                        raise RuntimeError('TLS prerequisite rejection missing')
                    if obj.exists(): raise RuntimeError('invalid TLS profile published an object')

            selection = json.loads(run([compiler, '--closed-world', binary, 'Main', obj, *flags]))
            assert any(row['implementation'] == 'tls-v1' for row in selection['nativeBindings'])
            executable = out / (name + ('.exe' if windows else '.native'))
            executable.unlink(missing_ok=True)
            run([*cc, '/DNEOCLR_TLS_AOT', obj, '/Fe:'+str(executable), *libs] if windows else [*cc, '-DNEOCLR_TLS_AOT', obj, *libs, '-o', executable])
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

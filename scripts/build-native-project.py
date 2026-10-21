#!/usr/bin/env python3
"""Build a Raven project into an experimental macOS ARM64 or Windows x64 native executable."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import runpy
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
PROFILE = 'macos-arm64-console-v1'
PROFILES = {'console': PROFILE, 'http': 'macos-arm64-http-v1', 'windows-console': 'windows-x64-console-v1', 'windows-http': 'windows-x64-http-v1'}


def native_text_sources():
    return [ROOT / 'tools/aot-native-text/Cargo.toml', ROOT / 'tools/aot-native-text/Cargo.lock',
            ROOT / 'tools/aot-native-text/src/lib.rs', ROOT / 'runtime/native/text/numeric-format.c',
            *[ROOT / 'src' / name for name in
              ('string_search.rs', 'string_normalization.rs', 'string_normalization_data.rs', 'string_comparison.rs', 'string_case_folding.rs', 'string_casing_data.rs', 'string_casing_kernel.rs', 'numeric_parse_core.rs', 'numeric_format_core.rs', 'time_zone_rules.rs')]]


def native_kernel_archive_name(windows, time_zone_rules=False):
    stem = 'neoclr_aot_native_text' + ('_time_zone' if time_zone_rules else '')
    return stem + '.lib' if windows else 'lib' + stem + '.a'


def native_kernel_build(target, time_zone_rules=False):
    manifest = ROOT / 'tools/aot-native-text/Cargo.toml'
    directory = manifest.parent / 'target' / ('time-zone-rules' if time_zone_rules else 'text-only')
    command = ['cargo', 'build', '--locked', '--release', '--target', target,
               '--target-dir', directory, '--manifest-path', manifest]
    if time_zone_rules:
        command += ['--features', 'time-zone-rules']
    archive = directory / target / 'release' / native_kernel_archive_name('windows' in target)
    return command, archive


def native_tls_sources():
    return [ROOT / 'tools/native-tls/Cargo.toml', ROOT / 'Cargo.toml', ROOT / 'Cargo.lock',
            *(ROOT / 'tools/native-tls/src').glob('*.rs'),
            ROOT / 'docs/experiments/https-server/native-tls.h',
            ROOT / 'docs/experiments/aot-console/tls-managed.c',
            ROOT / 'docs/experiments/aot-console/tls-managed.h']


def native_tls_build(windows=False):
    manifest = ROOT / 'tools/native-tls/Cargo.toml'
    target = 'x86_64-pc-windows-msvc' if windows else 'aarch64-apple-darwin'
    directory = manifest.parent / 'target'
    command = ['cargo', 'build', '--locked', '--profile', 'native-ffi', '--target', target,
               '--target-dir', directory, '--manifest-path', manifest]
    if windows:
        # Match the host's /MT CRT; TLS must not add a redistributable dependency.
        command += ['--config', 'target.x86_64-pc-windows-msvc.rustflags=["-C","target-feature=+crt-static"]']
    return command, directory / target / 'native-ffi' / ('neoclr_native_tls.lib' if windows else 'libneoclr_native_tls.a')


def native_crypto_sources():
    return [ROOT / 'tools/native-crypto/Cargo.toml', ROOT / 'tools/native-crypto/Cargo.lock',
            ROOT / 'tools/native-crypto/src/lib.rs', ROOT / 'tools/native-crypto/include/neoclr_crypto.h',
            ROOT / 'tools/security-provider/Cargo.toml', ROOT / 'tools/security-provider/src/lib.rs',
            ROOT / 'docs/experiments/aot-console/cryptography.c']


def native_crypto_build(windows=False):
    manifest = ROOT / 'tools/native-crypto/Cargo.toml'
    target = 'x86_64-pc-windows-msvc' if windows else 'aarch64-apple-darwin'
    directory = manifest.parent / 'target'
    command = ['cargo', 'build', '--locked', '--release', '--target', target,
               '--target-dir', directory, '--manifest-path', manifest]
    if windows:
        command += ['--config', 'target.x86_64-pc-windows-msvc.rustflags=["-C","target-feature=+crt-static"]']
    return command, directory / target / 'release' / ('neoclr_native_crypto.lib' if windows else 'libneoclr_native_crypto.a')


def validate_https_server(enabled, profile, bootstrap_root, packaged):
    if not enabled:
        return
    if profile not in (PROFILES['http'], PROFILES['windows-http']):
        raise ValueError('--https-server requires a source HTTP profile')
    if bootstrap_root:
        raise ValueError('--https-server does not yet support retained session bootstrap')
    if packaged:
        raise ValueError('--https-server currently requires a source checkout, not a packaged build kit')


def support_files():
    base = ROOT / 'docs/experiments'
    return [ROOT / 'scripts/build-native-project.py', ROOT / 'tools/native/console-host.c', ROOT / 'tools/native/entry-arguments.c',
            ROOT / 'benchmarks/native-web/http-host.c', ROOT / 'benchmarks/native-web/http-session-host.c',
            base / 'aot-console/native-session.c',
            *sorted((ROOT / 'runtime/native').rglob('*.[ch]')),
            *sorted((ROOT / 'tools/native').glob('windows-*.h')),
            *[ROOT / 'tools/native' / name for name in
              ('windows-console-host.c', 'windows-gc-host.c', 'windows-host-memory.c', 'windows-native-stack.c')],
            *[base / 'aot-console' / name for name in
              ('root-probe.c', 'native-gc.c', 'text-arena.c', 'console.c',
               'native-stack.c', 'task-queue.c', 'socket-listener.c', 'string-unicode.c', 'system-culture.c', 'time-zone.c', 'file-input.c')],
            base / 'aot-scalar/console.c', base / 'aot-fault-details/render.c',
            *sorted((base / 'aot-console').glob('*.h')),
            *sorted((base / 'aot-scalar').glob('*.h')),
            *sorted((base / 'aot-fault-details').glob('*.h'))]


def verify_files(root, files):
    for relative, expected in files.items():
        path = (root / relative).resolve()
        if Path(relative).is_absolute() or root not in path.parents or not path.is_file() or sha(path) != expected:
            raise ValueError('Package file mismatch: ' + relative)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build(project, bundle, aot, output, profile=PROFILE, bootstrap_root=None, reflection_roots=None, byte_array_limit=65536, http_heap_bytes=1048576, command_timeout=180, https_server=False, reflection_types=None, reflection_constructor_types=None):
    validate_https_server(https_server, profile, bootstrap_root, (ROOT / 'native-build-kit.json').exists())
    if not 0 < command_timeout <= 3600:
        raise ValueError("Command timeout must be between 1 and 3600 seconds")
    if not 0 < byte_array_limit <= 2147483647:
        raise ValueError("Byte array limit must be a positive Int32")
    if not 8 <= http_heap_bytes <= 2147483640 or http_heap_bytes % 8:
        raise ValueError("HTTP heap bytes must be a multiple of eight from 8 through 2147483640")
    if http_heap_bytes != 1048576 and profile not in (PROFILES['http'], PROFILES['windows-http']):
        raise ValueError("HTTP heap configuration requires an HTTP profile")
    if sum(bool(value) for value in (reflection_roots, reflection_types, reflection_constructor_types)) > 1:
        raise ValueError('Choose one reflection policy: roots, property types or constructor types')
    windows = profile in (PROFILES['windows-console'], PROFILES['windows-http'])
    http = profile in (PROFILES['http'], PROFILES['windows-http'])
    if bootstrap_root and not http:
        raise ValueError('Private session bootstrap requires an HTTP profile')
    if windows:
        if platform.system() != 'Windows' or platform.machine().lower() not in ('amd64', 'x86_64'):
            raise ValueError('Windows profiles require a Windows x64 MSVC build host')
    elif platform.system() != 'Darwin' or platform.machine() != 'arm64':
        raise ValueError('This profile requires a macOS ARM64 build host')
    compiler = bundle / 'sdk/tools/rvnc/rvnc.dll'
    for path in (project, compiler, aot, *([reflection_roots] if reflection_roots else [])):
        if not path.is_file():
            raise ValueError('Required input is missing: ' + str(path))
    if project.suffix != '.rvnproj':
        raise ValueError('--project must identify a Raven .rvnproj')
    for tool in (('dotnet', 'cl', 'dumpbin') if windows else ('dotnet', 'xcrun')):
        if shutil.which(tool) is None:
            raise ValueError('Required build tool is missing: ' + tool)
    output.mkdir(parents=True, exist_ok=False)
    report = dict(profile=profile, passed=False, project=str(project),
                  host=platform.platform(), httpsServer=https_server, byteArrayLimit=byte_array_limit, httpHeapBytes=http_heap_bytes, commandTimeoutSeconds=command_timeout, bootstrapRoot=bootstrap_root, commands=[], inputs={})

    def save():
        (output / 'build.json').write_text(json.dumps(report, indent=2) + '\n')

    build_env = {**os.environ, "NeoClrBundleRoot": str(bundle)}

    def run(command, phase="toolchain"):
        command = list(map(str, command))
        started = time.perf_counter()
        result = subprocess.run(command, cwd=project.parent, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=command_timeout,
                                env=build_env)
        report['commands'].append(dict(command=command, phase=phase, elapsedSeconds=round(time.perf_counter() - started, 6), exitCode=result.returncode,
                                       stdout=result.stdout, stderr=result.stderr))
        save()
        if result.returncode:
            raise RuntimeError(result.stdout + result.stderr or 'Command failed: ' + command[0])
        return result.stdout

    try:
        kit_path = ROOT / 'native-build-kit.json'
        if kit_path.exists():
            kit = json.loads(kit_path.read_text())
            if (kit.get('format') != 'neoclr-native-build-kit-v1'
                    or profile not in kit.get('profiles', [kit.get('profile')])):
                raise ValueError('Unsupported native build kit')
            if bundle != ROOT / 'bundle' or aot != ROOT / 'bin/neoclr-codegen':
                raise ValueError('Packaged builds require the kit-owned bundle and AOT tool')
            required_files = {p.relative_to(ROOT).as_posix() for p in support_files()}
            required_files |= {'bin/neoclr-codegen', 'bundle/manifest.json', 'bundle/lib/bundle.json'}
            if not required_files <= kit['files'].keys():
                raise ValueError('Native build kit omits required inputs')
            verify_files(ROOT, kit['files'])
            bundle_manifest = json.loads((bundle / 'manifest.json').read_text())
            if any(kit['files'].get('bundle/' + p) != digest for p, digest in bundle_manifest['files'].items()):
                raise ValueError('Native build kit differs from bundled toolchain manifest')
            report['kitManifestSha256'] = sha(kit_path)
        if windows:
            manifest = bundle / 'manifest.json'
            verify_files(bundle, json.loads(manifest.read_text())['files'])
            report['bundleManifestSha256'] = sha(manifest)
            report['toolchain'] = dict(cl=run(['where.exe', 'cl']).strip())
        else:
            clang = run(['xcrun', '--sdk', 'macosx', '--find', 'clang']).strip()
            sdk = run(['xcrun', '--sdk', 'macosx', '--show-sdk-path']).strip()
            build_env['SDKROOT'] = sdk
            report['toolchain'] = dict(clang=clang, sdk=sdk, version=run([clang, '--version']))
        lib = bundle / 'lib'
        catalog_path = lib / 'bundle.json'
        catalog = json.loads(catalog_path.read_text())
        core_assemblies = ['System.Runtime', 'System.Data', 'System.Networking', 'System.Web']
        if catalog.get('version') != 1 or catalog.get('assemblyNames') not in (
                core_assemblies, core_assemblies + ['System.Data.PostgreSql']):
            raise ValueError('This profile requires the split native class-library bundle v1 (optionally with PostgreSql)')
        if catalog.get('objectAssembly') != 'System.Runtime':
            raise ValueError('Unsupported bundle Object owner')
        for relative, expected in catalog['files'].items():
            path = (lib / relative).resolve()
            if lib not in path.parents or not path.is_file() or sha(path) != expected:
                raise ValueError('Bundle file mismatch: ' + relative)
        libraries = [lib / (name + '.dll') for name in catalog['assemblyNames']]
        seed = lib / catalog['runtimeSeed']
        required = [*libraries, seed, lib / catalog['projectConfiguration'], lib / 'ownership.json']
        if catalog.get('kind') != 'native-source-core-class-library':
            required.append(lib / 'Core.dll')
        if any(path.relative_to(lib).as_posix() not in catalog['files'] for path in required):
            raise ValueError('Bundle catalog omits a required input')
        base = ROOT / 'docs/experiments/aot-console'
        adapters = [ROOT / 'tools/native/console-host.c', *[base / name for name in
                    ('root-probe.c', 'native-gc.c', 'text-arena.c', 'console.c', 'native-stack.c')],
                    base.parent / 'aot-scalar/console.c', base.parent / 'aot-fault-details/render.c']
        if windows:
            adapters = [ROOT / 'tools/native' / name for name in
                        ('windows-console-host.c', 'windows-gc-host.c', 'windows-host-memory.c', 'windows-native-stack.c')]
            adapters += [base / name for name in ('root-probe.c', 'native-gc.c', 'text-arena.c', 'console.c')]
            adapters += [base.parent / 'aot-fault-details/render.c']
        if http:
            if windows:
                adapters += [ROOT / 'benchmarks/native-web/http-host.c']
            else:
                adapters[0] = ROOT / 'benchmarks/native-web/http-host.c'
            adapters += [base / name for name in ('task-queue.c', 'socket-listener.c')]
        if bootstrap_root:
            adapters = [ROOT / 'benchmarks/native-web/http-session-host.c' if p == ROOT / 'benchmarks/native-web/http-host.c' else p for p in adapters]
            adapters += [base / 'native-session.c']
        if not windows:
            adapters.append(base / 'file-input.c')
        adapters.append(ROOT / 'tools/native/entry-arguments.c')
        inputs = [*sorted((ROOT / 'runtime/native').rglob('*.[ch]')), Path(__file__).resolve(), project, catalog_path, aot, *required, *adapters,
                  *base.glob('*.h'), *base.parent.joinpath('aot-scalar').glob('*.h'),
                  *base.parent.joinpath('aot-fault-details').glob('*.h'),
                  *compiler.parent.glob('*.dll'), *compiler.parent.glob('*.json')]
        if windows:
            inputs += sorted((ROOT / 'tools/native').glob('windows-*.h'))
        report['inputs'] = {str(path): sha(path) for path in inputs}
        # Use Raven's evaluated project, including imports and project references.
        # The current driver reports the root output last, after dependency builds.
        stdout = run(['dotnet', compiler, 'neoclr', '--project', project], phase='source-compilation')
        paths = [line.removeprefix('Native build output: ') for line in stdout.splitlines()
                 if line.startswith('Native build output: ')]
        if not paths or not Path(paths[-1]).is_absolute() or not Path(paths[-1]).is_file():
            raise RuntimeError('Raven did not report a valid native project output')
        assembly = output / 'app.dll'
        shutil.copy2(paths[-1], assembly)
        if reflection_types or reflection_constructor_types:
            bundle_manifest = json.loads((bundle / 'manifest.json').read_text())
            runtime = bundle / bundle_manifest['runtime']
            helper = bundle / 'tools/run-native-sample.py'
            for path in (runtime, helper):
                report['inputs'][str(path)] = sha(path)
            reflection_roots = runpy.run_path(str(helper))['reflection_roots'](
                runtime, assembly, reflection_types or reflection_constructor_types, run,
                **({'constructors': True} if reflection_constructor_types else {}))[1]
        context = ['--system', seed, *[arg for path in libraries for arg in ('--module', path)],
                   '--object-root', libraries[0]]
        flags = ['--compile-system', '--bind-user-fault', '--reference-arena', '--native-gc',
                 '--bind-console-read-byte', '--bind-console-write-line', '--bind-console-stream-output',
                 '--bind-int32-to-string', '--bind-utf8-text', '--native-stack-budget', '--bind-integer-text', '--bind-character-text', '--bind-system-culture', '--bind-time-zone-rules']
        if not kit_path.exists():
            flags += ['--bind-cryptography', '--bind-tls']
        flags += ['--bind-scheduler', '--bind-task-queue', '--bind-timers']
        flags += ['--bind-runtime-async']
        if not windows:
            flags += ['--bind-file-input']
        if http:
            flags += [
                      '--bind-socket-listener', '--bind-socket-accept', '--bind-socket-transfer', '--bind-socket-client']
        if windows:
            flags += ['--target', 'x86_64-pc-windows-msvc', '--windows-http-experiment' if http else '--windows-console-experiment']
        selected_root = '@entry'
        if bootstrap_root:
            # Temporary Raven CLI bridge spells assembly-level functions as F_<UTF8 hex>.
            # Resolve from the actual inventory; never assume an assembly hash prefix.
            inventory = json.loads(run([aot, '--inspect', assembly, bootstrap_root]))
            spelling = 'F_' + bootstrap_root.encode('utf-8').hex().upper()
            matches = [f['name'] for f in inventory['functions']
                       if f['name'] == bootstrap_root or f['name'].rsplit('.', 1)[-1] == spelling]
            if len(matches) != 1:
                raise ValueError('Bootstrap must identify one assembly-level function: ' + bootstrap_root)
            selected_root = matches[0]
            report['resolvedBootstrapRoot'] = selected_root
            flags += ['--native-host-bootstrap']
        obj = output / ('app.obj' if windows else 'app.o')
        if reflection_roots:
            report['inputs'][str(reflection_roots)] = sha(reflection_roots)
            flags += ['--reflection-roots', reflection_roots]
        selection = json.loads(run([aot, '--closed-world', assembly, selected_root, obj, *context, *flags], phase='native-compilation'))
        runtime_async = any(binding.get('implementation', '').startswith('runtime-async-')
                            for binding in selection.get('nativeBindings', []))
        # Synchronous cancellation/disposal can retain timer services even when
        # no async method or task frame is reachable. Those services still need
        # invocation-owned scheduler state and the timer host implementation.
        timer_host = any(binding.get('implementation', '').startswith('timer-')
                         for binding in selection.get('nativeBindings', []))
        async_host = runtime_async or timer_host or bool(selection.get('taskQueueFrames'))
        no_default_queue = 'Drain' not in selection.get('taskQueueFrames', {})
        if async_host:
            # Async console applications need the same invocation ownership and
            # task/scheduler services as HTTP, without requiring a different profile.
            carrier = ROOT / 'runtime/native/async/native-runtime.c'
            if runtime_async:
                adapters.append(carrier)
            if not http:
                if windows:
                    adapters.append(ROOT / 'benchmarks/native-web/http-host.c')
                else:
                    adapters[0] = ROOT / 'benchmarks/native-web/http-host.c'
                adapters += [base / name for name in ('task-queue.c', 'socket-listener.c')]
            for path in adapters:
                report['inputs'][str(path)] = sha(path)
        system_culture = any(binding.get('implementation') == 'system-culture-name-v1'
                             for binding in selection.get('nativeBindings', []))
        if system_culture:
            adapters.append(base / 'system-culture.c')
        time_zone_rules = any(binding.get('implementation', '').startswith('time-zone-')
                              for binding in selection.get('nativeBindings', []))
        if time_zone_rules:
            adapters.append(base / 'time-zone.c')
        text_services = {'neoclr_char_is_single_grapheme_v1',
                         'neoclr_string_contains_canonical_v1', 'neoclr_string_starts_with_canonical_v1',
                         'neoclr_string_ends_with_canonical_v1', 'neoclr_string_contains_ignore_case_v1',
                         'neoclr_string_starts_with_ignore_case_v1', 'neoclr_string_ends_with_ignore_case_v1',
                         'neoclr_string_compare_canonical_v1', 'neoclr_string_hash_canonical_v1',
                         'neoclr_string_compare_ignore_case_v1', 'neoclr_string_hash_ordinal_v1',
                         'neoclr_string_hash_ignore_case_v1', 'neoclr_string_upper_v1',
                         'neoclr_string_lower_v1', 'neoclr_string_grapheme_count_v1',
                         'neoclr_string_slice_graphemes_v1', 'neoclr_string_grapheme_at_v1', 'neoclr_string_graphemes_v1',
                         'neoclr_string_scalars_v1', 'neoclr_string_from_chars_v1'}
        native_text = []
        if time_zone_rules or any(binding.get('nativePrimitiveParser') or binding.get('symbol') in text_services or binding.get('validationSymbol') == 'neoclr_is_single_grapheme_v1'
               for binding in selection.get('nativeBindings', [])):
            adapters.append(base / 'string-unicode.c')
            archive_name = native_kernel_archive_name(windows, time_zone_rules)
            if kit_path.exists():
                archive = ROOT / 'lib' / archive_name
                if kit['files'].get(archive.relative_to(ROOT).as_posix()) != sha(archive):
                    raise ValueError('Native build kit omits matching native text archive')
            else:
                for path in native_text_sources():
                    report['inputs'][str(path)] = sha(path)
                target = 'x86_64-pc-windows-msvc' if windows else 'aarch64-apple-darwin'
                command, archive = native_kernel_build(target, time_zone_rules)
                run(command, phase="native-kernel-build")
            report['inputs'][str(archive)] = sha(archive)
            native_text.append(archive)
        native_crypto = []
        if any(binding.get('implementation') == 'cryptography-v1'
               for binding in selection.get('nativeBindings', [])):
            if kit_path.exists():
                raise ValueError('Cryptography currently requires a source checkout, not a packaged build kit')
            for path in native_crypto_sources():
                report['inputs'][str(path)] = sha(path)
            adapters.append(base / 'cryptography.c')
            command, archive = native_crypto_build(windows)
            run(command, phase='native-crypto-build')
            report['inputs'][str(archive)] = sha(archive)
            native_crypto.append(archive)
        report['cryptography'] = bool(native_crypto)
        native_tls = []
        managed_tls = any(binding.get('implementation') == 'tls-v1'
                          for binding in selection.get('nativeBindings', []))
        if managed_tls:
            if kit_path.exists():
                raise ValueError('Managed TLS currently requires a source checkout, not a packaged build kit')
            adapters.append(base / 'tls-managed.c')
        report['managedTls'] = managed_tls
        if https_server or managed_tls:
            for path in native_tls_sources():
                report['inputs'][str(path)] = sha(path)
            command, archive = native_tls_build(windows)
            run(command, phase='native-tls-build')
            report['inputs'][str(archive)] = sha(archive)
            native_tls.append(archive)
        if https_server:
            report['httpsConfiguration'] = dict(scope='all-accepted-connections',
                certificateEnvironment='NEOCLR_HTTPS_CERTIFICATE', privateKeyEnvironment='NEOCLR_HTTPS_PRIVATE_KEY',
                credentialLoading='runtime-only', api='private-host-capability')
        # Publish the executable only after link and dependency checks succeed.
        pending = output / ('app.pending.exe' if windows else 'app.pending')
        if windows:
            run(['cl', '/nologo', '/W4', '/WX', '/std:c11', '/experimental:c11atomics', '/O2', '/MT',
                 '/DNEOCLR_NATIVE_GC', *(['/DNEOCLR_NO_DEFAULT_QUEUE'] if no_default_queue else []), *(['/DNEOCLR_MANAGED_TLS'] if managed_tls else []), *(['/DNEOCLR_HTTPS_SERVER'] if https_server else []), '/DNEOCLR_HTTP_HEAP_BYTES=' + str(http_heap_bytes), '/DNEOCLR_NATIVE_BYTE_ARRAY_LIMIT=' + str(byte_array_limit), *(['/DNEOCLR_ENTRY_ARGUMENTS'] if selection.get('entryArguments') else []), *(['/DNEOCLR_HTTP_HOST'] if http or async_host else []), *(['/DNEOCLR_RUNTIME_ASYNC', '/I' + str(ROOT / 'runtime/native/async')] if runtime_async else []), '/I' + str(base), '/Fo' + str(output) + '/',
                 '/Fe:' + str(pending), *adapters, obj, *native_text, *native_tls, *native_crypto, '/link', '/STACK:1048576', *(['Ws2_32.lib'] if http or async_host or native_crypto or managed_tls else []),
                 *(['Bcrypt.lib', 'Ntdll.lib', 'Userenv.lib', 'Advapi32.lib'] if https_server or native_crypto or managed_tls else [])], phase='host-compile-link')
            import re
            dependencies = re.findall(r'^\s+([A-Za-z0-9_.-]+\.dll)\s*$', run(['dumpbin', '/dependents', pending], phase='dependency-check'), re.MULTILINE | re.IGNORECASE)
            allowed = {'kernel32.dll', *(['ws2_32.dll'] if http or async_host or native_crypto or managed_tls else []),
                       *(['bcrypt.dll', 'bcryptprimitives.dll', 'api-ms-win-core-synch-l1-2-0.dll',
                          'ntdll.dll', 'userenv.dll', 'advapi32.dll'] if https_server or native_crypto or managed_tls else [])}
            if not dependencies or any(name.lower() not in allowed for name in dependencies):
                raise RuntimeError('Unexpected native dependencies: ' + repr(dependencies))
        else:
            run([clang, '-isysroot', sdk, '-arch', 'arm64', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror',
                 '-DNEOCLR_NATIVE_GC', *(['-DNEOCLR_NO_DEFAULT_QUEUE'] if no_default_queue else []), *(['-DNEOCLR_RUNTIME_ASYNC', '-I', ROOT / 'runtime/native/async', '-Wno-deprecated-declarations'] if runtime_async else []), *(['-DNEOCLR_MANAGED_TLS'] if managed_tls else []), *(['-DNEOCLR_HTTPS_SERVER'] if https_server else []), '-DNEOCLR_HTTP_HEAP_BYTES=' + str(http_heap_bytes), '-DNEOCLR_NATIVE_BYTE_ARRAY_LIMIT=' + str(byte_array_limit), *(['-DNEOCLR_ENTRY_ARGUMENTS'] if selection.get('entryArguments') else []), '-I', base, *adapters, obj, *native_text, *native_tls, *native_crypto, *(['-framework', 'CoreFoundation'] if system_culture else []), '-o', pending], phase='host-compile-link')
            dependencies = [line.split()[0] for line in run(['xcrun', 'otool', '-L', pending], phase='dependency-check').splitlines()[1:]]
            allowed = {'/usr/lib/libSystem.B.dylib'}
            if system_culture:
                allowed.add('/System/Library/Frameworks/CoreFoundation.framework/Versions/A/CoreFoundation')
            if set(dependencies) != allowed:
                raise RuntimeError('Unexpected native dependencies: ' + repr(dependencies))
        if any(sha(Path(path)) != digest for path, digest in report['inputs'].items()):
            raise RuntimeError('Build inputs changed during compilation')
        executable = 'app.exe' if windows else 'app'
        pending.rename(output / executable)
        report.update(passed=True, dependencies=dependencies,
                      artifacts={name: sha(output / name) for name in ('app.dll', obj.name, executable)})
        save()
    except Exception as error:
        (output / ('app.pending.exe' if windows else 'app.pending')).unlink(missing_ok=True)
        report['error'] = str(error)
        save()
        raise
    return output / ('app.exe' if windows else 'app')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--profile', choices=PROFILES, default='console',
                        help='Explicit native service/host profile (default: console)')
    parser.add_argument('--https-server', action='store_true', help='Source-only macOS/Windows HTTP host: TLS for all accepted connections; PEM credentials are supplied at runtime')
    parser.add_argument('--reflection-type', action='append', default=[], help='Application type name retained from the freshly compiled assembly; repeat for multiple types')
    parser.add_argument('--reflection-constructor-type', action='append', default=[], help='Exact application type name whose public constructors may be invoked; resolved after compilation')
    parser.add_argument('--reflection-roots', type=Path, help='Private versioned source-identity reflection retention JSON')
    parser.add_argument('--byte-array-limit', type=int, default=65536, help='Private native byte-array allocation limit; heap capacity remains independent')
    parser.add_argument('--bootstrap-root', help='Private retained HTTP experiment: fn<Void> factory, three host dispatches')
    for name in ('project', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    packaged = (ROOT / 'native-build-kit.json').is_file()
    parser.add_argument('--bundle', type=Path, required=not packaged, default=ROOT / 'bundle' if packaged else None)
    parser.add_argument('--aot', type=Path, required=not packaged, default=ROOT / 'bin/neoclr-codegen' if packaged else None)
    parser.add_argument('--command-timeout', type=int, default=180, help='Per-build-command timeout in seconds, from 1 through 3600')
    parser.add_argument('--http-heap-bytes', type=int, default=1048576, help='HTTP native host managed heap capacity, in bytes (multiple of eight)')
    args = parser.parse_args()
    try:
        print(build(*(getattr(args, name).resolve() for name in ('project', 'bundle', 'aot', 'output')),
                    profile=PROFILES[args.profile], bootstrap_root=args.bootstrap_root,
                    reflection_roots=args.reflection_roots.resolve() if args.reflection_roots else None,
                    reflection_types=args.reflection_type,
                    reflection_constructor_types=args.reflection_constructor_type,
                    byte_array_limit=args.byte_array_limit, http_heap_bytes=args.http_heap_bytes, command_timeout=args.command_timeout, https_server=args.https_server))
    except (OSError, ValueError, KeyError, TypeError, RuntimeError, subprocess.SubprocessError) as error:
        print('native build: ' + str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())

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
PROFILES = {'console': PROFILE, 'http': 'macos-arm64-http-v1', 'native-interop': 'macos-arm64-native-interop-v1', 'native-library': 'macos-arm64-native-library-v1', 'windows-console': 'windows-x64-console-v1', 'windows-http': 'windows-x64-http-v1'}


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


def validate_http_workers(count, profile, bootstrap_root, https_server, packaged):
    if not 0 <= count <= 4:
        raise ValueError('HTTP worker count must be 0 (default host) or 1 through 4')
    if count and (profile != PROFILES['http'] or bootstrap_root or https_server or packaged):
        raise ValueError('HTTP workers require a source macOS HTTP profile without TLS or retained bootstrap')


def scalar_interop_profile(profile):
    return profile in (PROFILES['native-interop'], PROFILES['native-library'])


def validate_scalar_interop(profile, bootstrap_root, reflection, byte_array_limit):
    if scalar_interop_profile(profile) and (bootstrap_root or reflection or byte_array_limit != 65536):
        raise ValueError('Native interop profiles do not support managed reflection, bootstrap or heap configuration')


def compile_scalar_interop(aot, assembly, context, output, profile, clang, sdk, run):
    """Use the existing scalar ABI; never pass managed-profile bindings here."""
    library = profile == PROFILES['native-library']
    obj = output / 'app.o'
    sources = [ROOT / 'runtime/native/interop/imports.c',
               ROOT / 'docs/experiments/aot-scalar/console.c']
    if library:
        report = json.loads(run([aot, '--native-library', assembly, obj,
            '--header', output / 'app.h', '--native-imports', *context], phase='native-compilation'))
    else:
        run([aot, assembly, '@entry', obj, '--native-imports', *context], phase='native-compilation')
        sources.append(ROOT / 'tools/native/interop-host.c')
        report = None
    name = 'libapp.dylib' if library else 'app'
    pending = output / (name + '.pending')
    try:
        run([clang, '-isysroot', sdk, '-arch', 'arm64', '-std=c11', '-O2',
             '-Wall', '-Wextra', '-Werror', '-I', ROOT / 'docs/experiments/aot-scalar',
             *(['-dynamiclib', '-Wl,-install_name,@rpath/libapp.dylib'] if library else []),
             *sources, obj, '-pthread', '-o', pending], phase='host-compile-link')
        dependencies = [line.split()[0] for line in
                        run(['xcrun', 'otool', '-L', pending], phase='dependency-check').splitlines()[1:]]
        expected = {'/usr/lib/libSystem.B.dylib'}
        if library:
            expected.add('@rpath/libapp.dylib')
        if set(dependencies) != expected:
            raise RuntimeError('Unexpected native dependencies: ' + repr(dependencies))
        return pending, name, dependencies, report
    except Exception:
        pending.unlink(missing_ok=True)
        raise


def support_files():
    base = ROOT / 'docs/experiments'
    return [ROOT / 'scripts/build-native-project.py', ROOT / 'tools/native/console-host.c', ROOT / 'tools/native/interop-host.c', ROOT / 'tools/native/entry-arguments.c',
            ROOT / 'benchmarks/native-web/http-host.c', ROOT / 'benchmarks/native-web/http-session-host.c',
            base / 'aot-console/native-session.c',
            *sorted((ROOT / 'runtime/native').rglob('*.[ch]')),
            *sorted((ROOT / 'tools/native').glob('windows-*.h')),
            *[ROOT / 'tools/native' / name for name in
              ('windows-console-host.c', 'windows-gc-host.c', 'windows-host-memory.c', 'windows-native-stack.c')],
            *[base / 'aot-console' / name for name in
              ('root-probe.c', 'native-gc.c', 'text-arena.c', 'console.c',
               'native-stack.c', 'task-queue.c', 'socket-listener.c', 'string-unicode.c', 'system-culture.c', 'time-zone.c', 'environment-platform.c', 'file-input.c')],
            base / 'aot-scalar/console.c', base / 'aot-fault-details/render.c',
            base / 'aot-console/tls-managed.c', base / 'https-server/native-tls.h',
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


def project_retention(project, explicit=None):
    """Read the author-facing policy before compilation; paths belong to the project."""
    path = Path(explicit) if explicit is not None else project.with_suffix('.retention.json')
    if explicit is not None and not path.is_absolute():
        path = project.parent / path
    if explicit is None and not path.exists():
        return None, None, None
    path = path.resolve()
    def unique_object(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError('Duplicate retention JSON key: ' + key)
            result[key] = value
        return result
    contents = path.read_bytes()
    policy = json.loads(contents, object_pairs_hook=unique_object)
    if (not isinstance(policy, dict) or set(policy) != {'schemaVersion', 'types'}
            or type(policy['schemaVersion']) is not int or policy['schemaVersion'] != 1
            or not isinstance(policy['types'], list) or not 1 <= len(policy['types']) <= 64):
        raise ValueError('Retention file requires schemaVersion 1 and 1–64 types')
    seen = set()
    for entry in policy['types']:
        if not isinstance(entry, dict) or set(entry) != {'name', 'retain'}:
            raise ValueError('Retention entries require name and retain')
        name, modes = entry['name'], entry['retain']
        if not isinstance(name, str) or not name or name != name.strip() or name in seen:
            raise ValueError('Retention names must be unique nonempty exact metadata names')
        if (not isinstance(modes, list) or not modes
                or any(not isinstance(mode, str) or mode not in ('constructors', 'properties', 'methods') for mode in modes)
                or len(set(modes)) != len(modes)):
            raise ValueError('retain requires distinct constructors, properties or methods values')
        seen.add(name)
    return path, policy['types'], hashlib.sha256(contents).hexdigest()


def resolve_project_retention(runtime, assembly, entries, run, modules, resolver):
    """Combine existing qualified policies without granting additional capabilities."""
    policy = dict(schemaVersion=8, types=[], moduleCatalogs=[], moduleFunctions=[],
                  typeCatalogs=[], methodTypes=[])
    roots = {}
    for mode in ('properties', 'constructors', 'methods'):
        names = [entry['name'] for entry in entries if mode in entry['retain']]
        if not names:
            continue
        options = {'constructors': True} if mode == 'constructors' else {'methods': True} if mode == 'methods' else {}
        path = resolver(runtime, assembly, names, run, modules=modules, **options)[1]
        resolved = json.loads(path.read_text())
        policy['methodTypes'].extend(resolved.get('methodTypes', []))
        for root in resolved['types']:
            key = json.dumps(root['definition'], sort_keys=True)
            target = roots.setdefault(key, dict(definition=root['definition'], **{
                flag: False for flag in ('construct', 'properties', 'getters', 'setters',
                    'customAttributes', 'fields', 'fieldGetters', 'fieldSetters', 'constructors')}))
            for flag, value in root.items():
                if flag != 'definition':
                    target[flag] |= value
    policy['types'] = list(roots.values())
    path = assembly.with_suffix('.reflection.json')
    path.write_text(json.dumps(policy, indent=2) + '\n')
    return path


def build(project, bundle, aot, output, profile=PROFILE, bootstrap_root=None, reflection_roots=None, byte_array_limit=65536, http_heap_bytes=1048576, command_timeout=180, https_server=False, reflection_types=None, reflection_constructor_types=None, reflection_method_types=None, http_workers=0, retention_file=None):
    retention_path, retention_entries, retention_digest = project_retention(project, retention_file)
    validate_https_server(https_server, profile, bootstrap_root, (ROOT / 'native-build-kit.json').exists())
    validate_http_workers(http_workers, profile, bootstrap_root, https_server, (ROOT / 'native-build-kit.json').exists())
    if not 0 < command_timeout <= 3600:
        raise ValueError("Command timeout must be between 1 and 3600 seconds")
    if not 0 < byte_array_limit <= 2147483647:
        raise ValueError("Byte array limit must be a positive Int32")
    if not 8 <= http_heap_bytes <= 2147483640 or http_heap_bytes % 8:
        raise ValueError("HTTP heap bytes must be a multiple of eight from 8 through 2147483640")
    if http_heap_bytes != 1048576 and profile not in (PROFILES['http'], PROFILES['windows-http']):
        raise ValueError("HTTP heap configuration requires an HTTP profile")
    if sum(bool(value) for value in (reflection_roots, reflection_types, reflection_constructor_types, reflection_method_types, retention_path)) > 1:
        raise ValueError('Choose one reflection policy: project retention file, roots, property types, constructor types or method types')
    if profile not in PROFILES.values():
        raise ValueError('Unsupported native build profile')
    validate_scalar_interop(profile, bootstrap_root,
        reflection_roots or reflection_types or reflection_constructor_types or reflection_method_types or retention_path,
        byte_array_limit)
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
                  host=platform.platform(), httpWorkers=http_workers, httpsServer=https_server, byteArrayLimit=byte_array_limit, httpHeapBytes=http_heap_bytes, commandTimeoutSeconds=command_timeout, bootstrapRoot=bootstrap_root, commands=[], inputs={})

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
            required_files |= {'bin/neoclr-codegen', 'bundle/manifest.json', 'bundle/lib/bundle.json',
                               'lib/libneoclr_native_tls.a'}
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
                core_assemblies, core_assemblies + ['System.Data.PostgreSql'],
                core_assemblies + ['Neo.DependencyInjection', 'Neo.Localization'],
                core_assemblies + ['Neo.DependencyInjection', 'Neo.Localization', 'System.Data.PostgreSql'],
                core_assemblies + ['Neo.DependencyInjection', 'Neo.Localization', 'Neo.Logging'],
                core_assemblies + ['Neo.DependencyInjection', 'Neo.Localization', 'Neo.Logging', 'System.Data.PostgreSql'],
                core_assemblies + ['Neo.DependencyInjection.Abstractions', 'Neo.DependencyInjection', 'Neo.Localization.Abstractions', 'Neo.Localization', 'Neo.Logging.Abstractions', 'Neo.Logging'],
                core_assemblies + ['Neo.DependencyInjection.Abstractions', 'Neo.DependencyInjection', 'Neo.Localization.Abstractions', 'Neo.Localization', 'Neo.Logging.Abstractions', 'Neo.Logging', 'System.Data.PostgreSql']):
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
        if http_workers:
            adapters += [ROOT / 'runtime/native/workers' / name for name in ('native-workers.c', 'http-workers.c')]
        if bootstrap_root:
            adapters = [ROOT / 'benchmarks/native-web/http-session-host.c' if p == ROOT / 'benchmarks/native-web/http-host.c' else p for p in adapters]
            adapters += [base / 'native-session.c']
        if not windows:
            adapters.append(base / 'file-input.c')
        adapters.append(ROOT / 'tools/native/entry-arguments.c')
        inputs = [ROOT / 'tools/native/interop-host.c', *sorted((ROOT / 'runtime/native').rglob('*.[ch]')), Path(__file__).resolve(), project, catalog_path, aot, *required, *adapters,
                  *base.glob('*.h'), *base.parent.joinpath('aot-scalar').glob('*.h'),
                  *base.parent.joinpath('aot-fault-details').glob('*.h'),
                  *compiler.parent.glob('*.dll'), *compiler.parent.glob('*.json')]
        if retention_path:
            inputs.append(retention_path)
            report['retentionFile'] = str(retention_path)
        if windows:
            inputs += sorted((ROOT / 'tools/native').glob('windows-*.h'))
        report['inputs'] = {str(path): sha(path) for path in inputs}
        if retention_path:
            report['inputs'][str(retention_path)] = retention_digest
        # Use Raven's evaluated project, including imports and project references.
        # The current driver reports the root output last, after dependency builds.
        stdout = run(['dotnet', compiler, 'neoclr', '--project', project], phase='source-compilation')
        paths = [line.removeprefix('Native build output: ') for line in stdout.splitlines()
                 if line.startswith('Native build output: ')]
        if not paths or not Path(paths[-1]).is_absolute() or not Path(paths[-1]).is_file():
            raise RuntimeError('Raven did not report a valid native project output')
        assembly = output / 'app.dll'
        shutil.copy2(paths[-1], assembly)
        if reflection_types or reflection_constructor_types or reflection_method_types or retention_entries:
            bundle_manifest = json.loads((bundle / 'manifest.json').read_text())
            runtime = bundle / bundle_manifest['runtime']
            helper = bundle / 'tools/run-native-sample.py'
            for path in (runtime, helper):
                report['inputs'][str(path)] = sha(path)
            resolver = runpy.run_path(str(helper))['reflection_roots']
            if retention_entries:
                reflection_roots = resolve_project_retention(
                    runtime, assembly, retention_entries, run, libraries, resolver)
            else:
                reflection_roots = resolver(
                    runtime, assembly, reflection_types or reflection_constructor_types or reflection_method_types, run,
                    modules=libraries,
                    **({'constructors': True} if reflection_constructor_types else {'methods': True} if reflection_method_types else {}))[1]

        context = ['--system', seed, *[arg for path in libraries for arg in ('--module', path)],
                   '--object-root', libraries[0]]
        if scalar_interop_profile(profile):
            pending, name, dependencies, exports = compile_scalar_interop(
                aot, assembly, context, output, profile, clang, sdk, run)
            if any(sha(Path(path)) != digest for path, digest in report['inputs'].items()):
                pending.unlink(missing_ok=True)
                raise RuntimeError('Build inputs changed during compilation')
            pending.rename(output / name)
            artifacts = ['app.dll', 'app.o', name]
            if exports is not None:
                report['nativeExports'] = exports
                artifacts.append('app.h')
            report.update(passed=True, dependencies=dependencies,
                          nativeInterop=True, artifacts={item: sha(output / item) for item in artifacts})
            save()
            return output / name
        flags = ['--compile-system', '--bind-user-fault', '--reference-arena', '--native-gc',
                 '--bind-console-read-byte', '--bind-console-write-line', '--bind-console-stream-output',
                 '--bind-int32-to-string', '--bind-utf8-text', '--bind-stack-trace', '--native-stack-budget', '--bind-integer-text', '--bind-character-text', '--bind-system-culture', '--bind-environment-platform', '--bind-time-zone-rules']
        if not kit_path.exists():
            flags += ['--bind-cryptography']
        flags += ['--bind-tls']
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
        if any(binding.get('implementation') == 'environment-platform-v1'
               for binding in selection.get('nativeBindings', [])):
            adapters += [base / 'environment-platform.c', ROOT / 'runtime/native/environment/platform.c']
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
            adapters.append(base / 'tls-managed.c')
        report['managedTls'] = managed_tls
        if https_server or managed_tls:
            if kit_path.exists():
                archive = ROOT / 'lib/libneoclr_native_tls.a'
            else:
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
                 '-DNEOCLR_NATIVE_GC', *(['-pthread', '-DNEOCLR_HTTP_WORKERS=' + str(http_workers)] if http_workers else []), *(['-DNEOCLR_NO_DEFAULT_QUEUE'] if no_default_queue else []), *(['-DNEOCLR_RUNTIME_ASYNC', '-I', ROOT / 'runtime/native/async', '-Wno-deprecated-declarations'] if runtime_async else []), *(['-DNEOCLR_MANAGED_TLS'] if managed_tls else []), *(['-DNEOCLR_HTTPS_SERVER'] if https_server else []), '-DNEOCLR_HTTP_HEAP_BYTES=' + str(http_heap_bytes), '-DNEOCLR_NATIVE_BYTE_ARRAY_LIMIT=' + str(byte_array_limit), *(['-DNEOCLR_ENTRY_ARGUMENTS'] if selection.get('entryArguments') else []), '-I', base, *adapters, obj, *native_text, *native_tls, *native_crypto, *(['-framework', 'CoreFoundation'] if system_culture else []), '-o', pending], phase='host-compile-link')
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
    parser.add_argument('--http-workers', type=int, default=0, help='Experimental source macOS HTTP host: 1–4 isolated application workers sharing their first listener; 0 keeps the default host')
    parser.add_argument('--https-server', action='store_true', help='Source-only macOS/Windows HTTP host: TLS for all accepted connections; PEM credentials are supplied at runtime')
    parser.add_argument('--reflection-type', action='append', default=[], help='Application type name retained from the freshly compiled assembly; repeat for multiple types')
    parser.add_argument('--reflection-method-type', action='append', default=[], help='Exact application type whose declared methods may be inspected; does not retain invocation')
    parser.add_argument('--reflection-constructor-type', action='append', default=[], help='Exact application type name whose public constructors may be invoked; resolved after compilation')
    parser.add_argument('--retention-file', type=Path, help='Named retention JSON, relative to the project directory; default: <project>.retention.json')
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
                    retention_file=args.retention_file,
                    reflection_types=args.reflection_type,
                    reflection_constructor_types=args.reflection_constructor_type,
                    reflection_method_types=args.reflection_method_type,
                    byte_array_limit=args.byte_array_limit, http_heap_bytes=args.http_heap_bytes, command_timeout=args.command_timeout, https_server=args.https_server, http_workers=args.http_workers))
    except (OSError, ValueError, KeyError, TypeError, RuntimeError, subprocess.SubprocessError) as error:
        print('native build: ' + str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())

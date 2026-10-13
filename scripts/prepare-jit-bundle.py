#!/usr/bin/env python3
"""Stage a relocatable development JIT bundle with a freshly rebuilt native helper."""
import argparse
import hashlib
import importlib.util
import json
import platform
from pathlib import Path
import shutil
import subprocess
from native_bundle_notices import refresh_notices
from native_bundle_samples import refresh_samples, sample_definitions, configure_editor_tasks

ROOT = Path(__file__).resolve().parents[1]
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--bundle', type=Path, required=True, help='Verified compiler/class-library bundle')
p.add_argument('--runtime', type=Path, required=True)
p.add_argument('--jit-compiler', type=Path, required=True)
p.add_argument('--host-recipe', type=Path, required=True, help='Passing native async validation or Windows HTTP build')
p.add_argument('--libraries', type=Path, help='Replace libraries with a coherent rebuilt native bundle')
p.add_argument('--compiler-directory', type=Path, help='Matching Raven compiler directory for replacement libraries')
p.add_argument('--output', type=Path, required=True)
p.add_argument('--refresh-samples', action='store_true', help='Refresh packaged Raven samples against replacement libraries')
p.add_argument('--heap-bytes', type=int, default=33554432)
p.add_argument('--byte-array-limit', type=int, default=4194304)
a = p.parse_args()
if a.refresh_samples and not a.libraries:
    p.error('--refresh-samples requires replacement --libraries')
windows = platform.system() == 'Windows' and platform.machine().lower() in ('amd64', 'x86_64')
if not windows and not (platform.system() == 'Darwin' and platform.machine() == 'arm64'):
    p.error('JIT bundles currently support macOS ARM64 and Windows x64')
if not 0 < a.byte_array_limit <= 2147483647 or not 0 < a.heap_bytes <= 2147483647:
    p.error('Heap and byte-array budgets must be positive Int32 values')
source, out = a.bundle.resolve(), a.output.resolve()
manifest = json.loads((source / 'manifest.json').read_text())
retained = {relative: digest for relative, digest in manifest['files'].items()
            if not (a.libraries and (relative.startswith('lib/') or relative.startswith('sdk/tools/rvnc/')))}
for relative, expected in retained.items():
    path = (source / relative).resolve()
    if source not in path.parents or sha(path) != expected:
        raise ValueError('Bundle mismatch: ' + relative)
recipe = json.loads(a.host_recipe.read_text())
if not recipe['passed']:
    raise ValueError('Passing host recipe required')
out.mkdir(parents=True, exist_ok=False)
bundle = out / 'bundle'
# Copy only catalogued inputs, not arbitrary previous build outputs.
for relative in retained:
    destination = bundle / relative
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source / relative, destination)
if a.libraries:
    catalog = json.loads((a.libraries / 'bundle.json').read_text())
    for relative, expected in catalog['files'].items():
        path = (a.libraries / relative).resolve()
        if a.libraries.resolve() not in path.parents or sha(path) != expected:
            raise ValueError('Replacement library mismatch: ' + relative)
    if not a.compiler_directory:
        raise ValueError('Replacement libraries require their matching --compiler-directory')
    evidence = json.loads((a.libraries / 'build-evidence.json').read_text())
    producer = Path(evidence['commands'][0]['command'][1]).parent
    for name in ('rvnc.dll', 'Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll'):
        if sha(a.compiler_directory / name) != evidence['hashes'][str((producer / name).resolve())]:
            raise ValueError('Replacement compiler does not match library producer: ' + name)
    if (bundle / 'lib').exists():
        shutil.rmtree(bundle / 'lib')
    shutil.copytree(a.libraries, bundle / 'lib')
    if (bundle / 'sdk/tools/rvnc').exists():
        shutil.rmtree(bundle / 'sdk/tools/rvnc')
    shutil.copytree(a.compiler_directory, bundle / 'sdk/tools/rvnc')
report = dict(passed=False, commands=[], originalManifestSha256=sha(source / 'manifest.json'),
    replacementLibraries=bool(a.libraries), inputsSha256={str(a.host_recipe.resolve()): sha(a.host_recipe),
    str(a.runtime.resolve()): sha(a.runtime), str(a.jit_compiler.resolve()): sha(a.jit_compiler),
    str(Path(__file__).resolve()): sha(__file__)})
for path in (ROOT / 'runtime/native').rglob('*.[ch]'):
    report['inputsSha256'][str(path)] = sha(path)
for directory in ('aot-console', 'aot-fault-details', 'runtime-async', 'jit-managed'):
    for path in (ROOT / 'docs/experiments' / directory).iterdir():
        if path.suffix in ('.c', '.h'):
            report['inputsSha256'][str(path)] = sha(path)


def run(command):
    command = list(map(str, command))
    for value in command:
        path = Path(value)
        if path.is_file() and not path.resolve().is_relative_to(out):
            report['inputsSha256'][str(path.resolve())] = sha(path)
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=180)
    report['commands'].append(dict(command=command, exitCode=result.returncode,
                                   stdout=result.stdout, stderr=result.stderr))
    if result.returncode:
        raise RuntimeError(result.stderr)
    return result


def budget_command(command, prefix):
    names = ('NEOCLR_HTTP_HEAP_BYTES', 'NEOCLR_NATIVE_BYTE_ARRAY_LIMIT')
    command = [str(arg) for arg in command if not any(str(arg).startswith(prefix + name + '=') for name in names)]
    flags = [prefix + names[0] + '=' + str(a.heap_bytes), prefix + names[1] + '=' + str(a.byte_array_limit)]
    return command[:1] + flags + command[1:]


def complete_services(command):
    spec = importlib.util.spec_from_file_location('native_project', ROOT / 'scripts/build-native-project.py')
    project = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(project)
    for path in project.native_text_sources():
        report['inputsSha256'][str(path)] = sha(path)
    target = 'x86_64-pc-windows-msvc' if windows else 'aarch64-apple-darwin'
    build, archive = project.native_kernel_build(target, time_zone_rules=True)
    run(build)
    # One kernel supplies both text and time-zone symbols; do not link the
    # text-only archive alongside it and introduce duplicate definitions.
    command = [str(arg) for arg in command if 'neoclr_aot_native_text' not in str(arg)]
    additions = [ROOT / 'docs/experiments/aot-console' / name
                 for name in ('string-unicode.c', 'system-culture.c', 'time-zone.c')]
    additions.append(archive)
    for path in project.native_crypto_sources():
        report['inputsSha256'][str(path)] = sha(path)
    crypto_build, crypto_archive = project.native_crypto_build(windows)
    run(crypto_build)
    # A qualified source recipe may already contain the cryptography archive.
    command = [arg for arg in command if 'neoclr_native_crypto' not in str(arg)]
    additions += [ROOT / 'docs/experiments/aot-console/cryptography.c', crypto_archive]
    for path in project.native_tls_sources():
        report['inputsSha256'][str(path)] = sha(path)
    tls_build, tls_archive = project.native_tls_build(windows)
    run(tls_build)
    command = [arg for arg in command if 'neoclr_native_tls' not in str(arg)]
    additions += [ROOT / 'docs/experiments/aot-console/tls-managed.c', tls_archive]
    tls_flag = '/DNEOCLR_MANAGED_TLS' if windows else '-DNEOCLR_MANAGED_TLS'
    if tls_flag not in command:
        command.insert(1, tls_flag)

    if windows:
        additions += [name for name in ('Bcrypt.lib', 'Ntdll.lib', 'Userenv.lib', 'Advapi32.lib', 'Ws2_32.lib')
                      if name.lower() not in {str(arg).lower() for arg in command}]
    for path in additions:
        if str(path) not in command:
            command.insert(command.index('/link') if windows else len(command), str(path))
    if not windows and 'CoreFoundation' not in command:
        command += ['-framework', 'CoreFoundation']
    return command


try:
    if windows:
        spec = importlib.util.spec_from_file_location('windows_host', ROOT / 'docs/experiments/jit-http/windows.py')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        _, command = module.recipe(recipe, ROOT, out)
        command = complete_services(command)
        command = budget_command(command, '/D')
        helper = module.build_host(command, out, run)
    else:
        command = next(row['command'].copy() for row in recipe['commands'] if row.get('label') == 'jit-host-build')
        helper = out / 'host.dylib'
        command[command.index('-o') + 1] = str(helper)
        command = complete_services(command)
        command = budget_command(command, '-D')
        run(command)
    suffix = '.exe' if windows else ''
    shutil.copy2(a.runtime, bundle / ('bin/neoclr' + suffix))
    shutil.copy2(a.jit_compiler, bundle / ('bin/neoclr-aot-poc' + suffix))
    helper_name = 'neoclr-jit-host.dll' if windows else 'neoclr-jit-host.dylib'
    shutil.copy2(helper, bundle / 'bin' / helper_name)
    original_metadata = {key: value for key, value in manifest.items() if key != 'files'}
    manifest = {key: manifest[key] for key in ('format', 'objectRoot', 'runtimeSeed', 'modules', 'samples') if key in manifest}
    manifest['baseManifestProvenance'] = original_metadata
    manifest['developmentComposition'] = 'Installed default JIT; binaries and compiler/library inputs identified by file hashes'
    manifest['platform'] = platform.platform()
    manifest['runtime'] = 'bin/neoclr' + suffix
    manifest['jit'] = dict(default=True, compiler='bin/neoclr-aot-poc' + suffix,
                           host='bin/' + helper_name, runtimeAsync=not windows, cryptography=True, tlsOwnership=True,
                           heapBytes=a.heap_bytes, byteArrayLimit=a.byte_array_limit)
    if a.refresh_samples:
        refresh_samples(bundle, manifest)
        for sources, _, _ in sample_definitions().values():
            for source in sources:
                path = ROOT / 'docs/experiments' / source
                report['inputsSha256'][str(path)] = sha(path)
    configure_editor_tasks(bundle, manifest['samples'])
    path = ROOT / 'scripts/native_bundle_samples.py'
    report['inputsSha256'][str(path)] = sha(path)
    # Acceptance defaults to installed JIT when the bundle advertises it.
    for name in ('verify-native-bundle.py', 'verify-native-http-json.py', 'run-native-sample.py'):
        shutil.copy2(ROOT / 'scripts' / name, bundle / 'tools' / name)
    report['inputsSha256'].update(refresh_notices(ROOT, bundle))
    path = ROOT / 'scripts/native_bundle_notices.py'
    report['inputsSha256'][str(path)] = sha(path)
    manifest['files'] = {path.relative_to(bundle).as_posix(): sha(path)
                         for path in sorted(bundle.rglob('*')) if path.is_file()}
    (bundle / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    if any(sha(path) != digest for path, digest in report['inputsSha256'].items()):
        raise RuntimeError('Input changed while preparing bundle')
    report['manifestSha256'] = sha(bundle / 'manifest.json')
    report['passed'] = True
finally:
    (out / 'preparation.json').write_text(json.dumps(report, indent=2) + '\n')
print(bundle)

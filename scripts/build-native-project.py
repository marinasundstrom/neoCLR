#!/usr/bin/env python3
"""Build a Raven project into an experimental macOS ARM64 or Windows x64 native executable."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
PROFILE = 'macos-arm64-console-v1'
PROFILES = {'console': PROFILE, 'http': 'macos-arm64-http-v1', 'windows-console': 'windows-x64-console-v1', 'windows-http': 'windows-x64-http-v1'}


def support_files():
    base = ROOT / 'docs/experiments'
    return [ROOT / 'scripts/build-native-project.py', ROOT / 'tools/native/console-host.c',
            ROOT / 'benchmarks/native-web/http-host.c', ROOT / 'benchmarks/native-web/http-session-host.c',
            base / 'aot-console/native-session.c',
            *sorted((ROOT / 'tools/native').glob('windows-*.h')),
            *[ROOT / 'tools/native' / name for name in
              ('windows-console-host.c', 'windows-gc-host.c', 'windows-host-memory.c', 'windows-native-stack.c')],
            *[base / 'aot-console' / name for name in
              ('root-probe.c', 'native-gc.c', 'text-arena.c', 'console.c',
               'native-stack.c', 'task-queue.c', 'socket-listener.c')],
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


def build(project, bundle, aot, output, profile=PROFILE, bootstrap_root=None, reflection_roots=None):
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
                  host=platform.platform(), bootstrapRoot=bootstrap_root, commands=[], inputs={})

    def save():
        (output / 'build.json').write_text(json.dumps(report, indent=2) + '\n')

    def run(command):
        command = list(map(str, command))
        result = subprocess.run(command, cwd=project.parent, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=180,
                                env={**os.environ, "NeoClrBundleRoot": str(bundle)})
        report['commands'].append(dict(command=command, exitCode=result.returncode,
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
            if bundle != ROOT / 'bundle' or aot != ROOT / 'bin/neoclr-aot-poc':
                raise ValueError('Packaged builds require the kit-owned bundle and AOT tool')
            required_files = {p.relative_to(ROOT).as_posix() for p in support_files()}
            required_files |= {'bin/neoclr-aot-poc', 'bundle/manifest.json', 'bundle/lib/bundle.json'}
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
            report['toolchain'] = dict(clang=clang, sdk=sdk, version=run([clang, '--version']))
        lib = bundle / 'lib'
        catalog_path = lib / 'bundle.json'
        catalog = json.loads(catalog_path.read_text())
        if catalog.get('version') != 1 or catalog.get('assemblyNames') != [
                'System.Runtime', 'System.Data', 'System.Networking', 'System.Web']:
            raise ValueError('This profile requires the split native class-library bundle v1')
        if catalog.get('objectAssembly') != 'System.Runtime':
            raise ValueError('Unsupported bundle Object owner')
        for relative, expected in catalog['files'].items():
            path = (lib / relative).resolve()
            if lib not in path.parents or not path.is_file() or sha(path) != expected:
                raise ValueError('Bundle file mismatch: ' + relative)
        libraries = [lib / (name + '.dll') for name in catalog['assemblyNames']]
        seed = lib / catalog['runtimeSeed']
        required = [*libraries, seed, lib / catalog['projectConfiguration'], lib / 'Core.dll', lib / 'ownership.json']
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
        inputs = [Path(__file__).resolve(), project, catalog_path, aot, *required, *adapters,
                  *base.glob('*.h'), *base.parent.joinpath('aot-scalar').glob('*.h'),
                  *base.parent.joinpath('aot-fault-details').glob('*.h'),
                  *compiler.parent.glob('*.dll'), *compiler.parent.glob('*.json')]
        if windows:
            inputs += sorted((ROOT / 'tools/native').glob('windows-*.h'))
        report['inputs'] = {str(path): sha(path) for path in inputs}
        # Use Raven's evaluated project, including imports and project references.
        # The current driver reports the root output last, after dependency builds.
        stdout = run(['dotnet', compiler, 'neoclr', '--project', project])
        paths = [line.removeprefix('Native build output: ') for line in stdout.splitlines()
                 if line.startswith('Native build output: ')]
        if not paths or not Path(paths[-1]).is_absolute() or not Path(paths[-1]).is_file():
            raise RuntimeError('Raven did not report a valid native project output')
        assembly = output / 'app.dll'
        shutil.copy2(paths[-1], assembly)
        context = ['--system', seed, *[arg for path in libraries for arg in ('--module', path)],
                   '--object-root', libraries[0]]
        flags = ['--compile-system', '--bind-user-fault', '--reference-arena', '--native-gc',
                 '--bind-console-read-byte', '--bind-console-write-line', '--bind-console-stream-output',
                 '--bind-int32-to-string', '--bind-utf8-text', '--native-stack-budget', '--bind-integer-text']
        if http:
            flags += ['--bind-task-queue',
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
        run([aot, '--closed-world', assembly, selected_root, obj, *context, *flags])
        # Publish the executable only after link and dependency checks succeed.
        pending = output / ('app.pending.exe' if windows else 'app.pending')
        if windows:
            run(['cl', '/nologo', '/W4', '/WX', '/std:c11', '/experimental:c11atomics', '/O2', '/MT',
                 '/DNEOCLR_NATIVE_GC', *(['/DNEOCLR_HTTP_HOST'] if http else []), '/I' + str(base), '/Fo' + str(output) + '/',
                 '/Fe:' + str(pending), *adapters, obj, '/link', '/STACK:1048576', *(['Ws2_32.lib'] if http else [])])
            import re
            dependencies = re.findall(r'^\s+([A-Za-z0-9_.-]+\.dll)\s*$', run(['dumpbin', '/dependents', pending]), re.MULTILINE | re.IGNORECASE)
            allowed = {'kernel32.dll', *(['ws2_32.dll'] if http else [])}
            if not dependencies or any(name.lower() not in allowed for name in dependencies):
                raise RuntimeError('Unexpected native dependencies: ' + repr(dependencies))
        else:
            run([clang, '-isysroot', sdk, '-arch', 'arm64', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror',
                 '-DNEOCLR_NATIVE_GC', '-I', base, *adapters, obj, '-o', pending])
            dependencies = [line.split()[0] for line in run(['xcrun', 'otool', '-L', pending]).splitlines()[1:]]
            if dependencies != ['/usr/lib/libSystem.B.dylib']:
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
    parser.add_argument('--reflection-roots', type=Path, help='Private versioned source-identity reflection retention JSON')
    parser.add_argument('--bootstrap-root', help='Private retained HTTP experiment: fn<Void> factory, three host dispatches')
    for name in ('project', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    packaged = (ROOT / 'native-build-kit.json').is_file()
    parser.add_argument('--bundle', type=Path, required=not packaged, default=ROOT / 'bundle' if packaged else None)
    parser.add_argument('--aot', type=Path, required=not packaged, default=ROOT / 'bin/neoclr-aot-poc' if packaged else None)
    args = parser.parse_args()
    try:
        print(build(*(getattr(args, name).resolve() for name in ('project', 'bundle', 'aot', 'output')),
                    profile=PROFILES[args.profile], bootstrap_root=args.bootstrap_root,
                    reflection_roots=args.reflection_roots.resolve() if args.reflection_roots else None))
    except (OSError, ValueError, KeyError, TypeError, RuntimeError, subprocess.SubprocessError) as error:
        print('native build: ' + str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())

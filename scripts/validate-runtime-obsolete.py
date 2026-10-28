#!/usr/bin/env python3
"""Qualify the source runtime obsolete attribute with isolated native projects."""
import argparse
import hashlib
import json
import os
import platform
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'libraries', 'runtime', 'aot', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument("--skip-native", action="store_true", help="Run portable compiler/interpreter checks only")
    args = parser.parse_args()
    if not args.skip_native and (sys.platform != "darwin" or platform.machine() != "arm64"):
        raise RuntimeError("Native execution qualification currently requires macOS ARM64")
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    libraries = args.libraries.resolve()
    environment = dict(os.environ, NeoClrBundleRoot=str(out))
    # Copy the matching library input so Windows does not require symlink privileges.
    shutil.copytree(libraries, out / 'lib')
    project = out / 'projects'
    shutil.copytree(ROOT / 'docs/experiments/obsolete-runtime', project,
                    ignore=shutil.ignore_patterns('bin', 'obj'))
    source_files = [ROOT / 'runtime/System.Runtime/src/System.Runtime/System/ObsoleteAttribute.rvn', ROOT / 'runtime/System.Runtime/src/System.Runtime/System/String.rvn', ROOT / 'docs/experiments/obsolete-runtime/Main.rvn']
    report = {'passed': False, 'platform': platform.system() + '-' + platform.machine(),
              'sourceFiles': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in source_files},
              'runtimeSha256': hashlib.sha256(args.runtime.read_bytes()).hexdigest(),
              'aotSha256': hashlib.sha256(args.aot.read_bytes()).hexdigest(),
              'libraryArtifacts': {k: v for k, v in json.loads((libraries / 'bundle.json').read_text())['files'].items()
                                   if k.endswith(('.dll', '.neox'))},
              'commands': []}

    def run(command, expected=0, env=environment):
        result = subprocess.run(list(map(str, command)), env=env, cwd=ROOT,
                                capture_output=True, text=True, timeout=180)
        report['commands'].append(dict(command=result.args, exitCode=result.returncode,
                                       stdout=result.stdout, stderr=result.stderr))
        (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
        if result.returncode != expected:
            raise RuntimeError(result.stdout + result.stderr)
        return result

    compiler = ['dotnet', args.compiler.resolve(), 'neoclr', '--project']
    template = '''<Project Sdk="Microsoft.NET.Sdk">
  <Import Project="$(NeoClrBundleRoot)/lib/NeoCLR.ClassLibrary.props" />
  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType>
    <AssemblyName>{name}</AssemblyName><EnableDefaultCompileItems>false</EnableDefaultCompileItems>
  </PropertyGroup>
  <ItemGroup><Compile Include="{name}.rvn" /></ItemGroup>
</Project>'''
    def compile_case(name, source, expected, diagnostic=None):
        (project / (name + '.rvn')).write_text(source)
        path = project / (name + '.rvnproj')
        path.write_text(template.format(name=name))
        result = run([*compiler, path], expected)
        if diagnostic:
            assert diagnostic in result.stdout + result.stderr, result.stdout + result.stderr
        image = project / 'bin/neoclr' / (name + '.dll')
        assert image.exists() == (expected == 0)
        return image

    consumer = compile_case('Main', (project / 'Main.rvn').read_text(), 0, 'warning RAV0505')
    warnings = report['commands'][-1]['stdout'] + report['commands'][-1]['stderr']
    assert 'GetUtf8ByteCount' in warnings and 'SliceUtf8' in warnings
    compile_case('DefaultWarning', 'import System.*\n[Obsolete]\nfunc Old() {}\nfunc Main() { Old() }\n', 0, 'warning RAV0505')
    context = ['--system', libraries / 'System.runtime.neox']
    for name in ('System.Runtime', 'System.Data', 'System.Networking', 'System.Web'):
        context += ['--module', libraries / (name + '.dll')]
    context += ['--object-root', libraries / 'System.Runtime.dll']
    run([args.runtime.resolve(), 'run', consumer, *context, '--show-result'], 42)
    compile_case('Error', 'import System.*\n[Obsolete("Use Current", true)]\nfunc Old() {}\nfunc Main() { Old() }\n', 1, 'error RAV0505')
    compile_case('InvalidTarget', 'import System.*\nfunc Check([Obsolete] value: int) {}\nfunc Main() { Check(1) }\n', 1, 'RAV0502')
    compile_case('Repeated', 'import System.*\n[Obsolete][Obsolete]\nfunc Old() {}\nfunc Main() {}\n', 1, 'RAV0503')
    if not args.skip_native:
        run([args.aot.resolve(), '--closed-world', consumer, '@entry', out / 'app.o',
             '--reference-arena', '--compile-system', '--bind-utf8-text', '--bind-user-fault', *context])
        (out / 'host.c').write_text('''#include "text-arena.h"
    int main(void) {
        uint64_t storage[256] = {0};
        neoclr_aot_context context = { .text = {(unsigned char*)storage, sizeof(storage), 0} };
        int32_t result = -1;
        return neoclr_entry_v4(0, &result, &context) || result != 42;
    }
    ''')
        environment['SDKROOT'] = subprocess.check_output(['xcrun', '--sdk', 'macosx', '--show-sdk-path'], text=True).strip()
        base = ROOT / 'docs/experiments'
        run(['clang', '-arch', 'arm64', '-std=c11', '-Wall', '-Wextra', '-Werror', '-I', base / 'aot-console',
             out / 'host.c', base / 'aot-console/text-arena.c', base / 'aot-fault-details/render.c',
             out / 'app.o', '-o', out / 'app'])
        run([out / 'app'], env={})
    report['nativeExecution'] = not args.skip_native
    report['passed'] = True
    (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS obsolete attribute: constructors, nullable messages, diagnostics and interpreter' +
          ('' if args.skip_native else ', plus macOS ARM64 AOT'))


if __name__ == '__main__':
    main()

#!/usr/bin/env python3
"""Qualify the source runtime friend attribute with isolated native projects."""
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
    args = parser.parse_args()
    if sys.platform != "darwin" or platform.machine() != "arm64":
        raise RuntimeError("Native execution qualification currently requires macOS ARM64")
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    libraries = args.libraries.resolve()
    environment = dict(os.environ, NeoClrBundleRoot=str(out))
    # The projects use the usual bundle import; link only the matching library input.
    (out / 'lib').symlink_to(libraries, target_is_directory=True)
    project = out / 'projects'
    shutil.copytree(ROOT / 'docs/experiments/friend-runtime', project,
                    ignore=shutil.ignore_patterns('bin', 'obj'))
    source_files = [ROOT / 'runtime/raven/src/System/Runtime/CompilerServices/InternalsVisibleToAttribute.rvn',
                    ROOT / 'docs/experiments/friend-runtime/Provider.rvn', ROOT / 'docs/experiments/friend-runtime/Main.rvn']
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
    run([*compiler, project / 'Consumer.rvnproj'])
    artifacts = project / 'bin/neoclr'
    provider = artifacts / 'RuntimeFriendProvider.dll'
    consumer = artifacts / 'RuntimeFriendConsumer.dll'
    context = ['--system', libraries / 'System.runtime.neox']
    for name in ('System.Runtime', 'System.Data', 'System.Networking', 'System.Web'):
        context += ['--module', libraries / (name + '.dll')]
    context += ['--module', provider, '--object-root', libraries / 'System.Runtime.dll']
    run([args.runtime.resolve(), 'run', consumer, *context, '--show-result'], 42)
    original = (project / 'Consumer.rvnproj').read_text()
    (project / 'Unrelated.rvnproj').write_text(original.replace('RuntimeFriendConsumer', 'Unrelated'))
    denied = run([*compiler, project / 'Unrelated.rvnproj'], 1)
    assert 'RAV0500' in denied.stdout + denied.stderr
    assert not (artifacts / 'Unrelated.dll').exists()
    invalid = (project / 'Provider.rvnproj').read_text().replace('RuntimeFriendProvider', 'InvalidUsage').replace('Provider.rvn', 'Invalid.rvn')
    (project / 'Invalid.rvnproj').write_text(invalid)
    (project / 'Invalid.rvn').write_text('import System.Runtime.CompilerServices.*\n[InternalsVisibleTo("Friend")]\npublic class Invalid {}\n')
    denied = run([*compiler, project / 'Invalid.rvnproj'], 1)
    assert 'RAV0502' in denied.stdout + denied.stderr
    assert not (artifacts / 'InvalidUsage.dll').exists()
    run([args.aot.resolve(), '--closed-world', consumer, '@entry', out / 'app.o',
         '--reference-arena', '--compile-system', '--bind-utf8-text', *context])
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
    report['passed'] = True
    (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS runtime friend attribute: repeated grants, property, access rejection, interpreter and macOS ARM64 AOT')


if __name__ == '__main__':
    main()

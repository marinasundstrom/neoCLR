"""Qualify fresh Raven Task union delivery in interpreter, AOT and managed JIT."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', required=True, type=Path)
    parser.add_argument('--bundle', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--closed-generic', action='store_true')
    args = parser.parse_args()
    if (platform.system(), platform.machine()) != ('Darwin', 'arm64'):
        parser.error('This qualification requires macOS ARM64')
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    compiler, bundle = args.compiler.resolve(), args.bundle.resolve()
    fixture = 'closed-generic-task-project' if args.closed_generic else 'direct-task-project'
    project = output / 'project'
    shutil.copytree(ROOT / 'docs/experiments/value-unions' / fixture, project,
                    ignore=shutil.ignore_patterns('bin', 'obj'))
    project_file = project / 'App.rvnproj'
    project_file.write_text(project_file.read_text().replace(
        '../../../../target/structural-integrated-libraries-v3/NeoCLR.ClassLibrary.props',
        str(bundle / 'NeoCLR.ClassLibrary.props')))
    commands = []

    def run(label, command, expected=0):
        command = list(map(str, command))
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=180)
        (output / (label + '.stdout')).write_text(result.stdout)
        (output / (label + '.stderr')).write_text(result.stderr)
        commands.append({'label': label, 'command': command, 'exitCode': result.returncode})
        if result.returncode != expected:
            raise RuntimeError(f'{label}: expected {expected}, got {result.returncode}; see {output}')
        return result.stdout

    compiled = run('compile', ['dotnet', compiler, 'neoclr', '--project', project_file])
    artifact = Path(next(line.removeprefix('Native build output: ') for line in compiled.splitlines()
                         if line.startswith('Native build output: ')))
    runtime, codegen = (ROOT / 'target/debug' / name for name in ('neoclr', 'neoclr-codegen'))
    context = ['--system', bundle / 'System.runtime.neox', '--object-root', bundle / 'System.Runtime.dll']
    for name in ['System.Runtime', 'System.Data', 'System.Networking', 'System.Web']:
        context += ['--module', bundle / (name + '.dll')]
    run('interpreter', [runtime, 'run', artifact, '--execution', 'interpreter', *context], 42)
    flags = ['--compile-system', '--reference-arena', '--native-gc', '--native-stack-budget',
             '--bind-user-fault', '--bind-scheduler', '--bind-task-queue', '--bind-timers', '--bind-runtime-async']
    obj = output / 'app.o'
    run('aot', [codegen, '--closed-world', artifact, '@entry', obj, *context, *flags])
    imports = run('imports', ['nm', '-u', obj]).split()
    base = ROOT / 'docs/experiments/aot-console'
    sources = [base / name for name in ('root-probe.c', 'native-gc.c', 'text-arena.c', 'console.c',
                                       'native-stack.c', 'task-queue.c', 'socket-listener.c')]
    sources += [ROOT / 'docs/experiments/aot-scalar/console.c',
                ROOT / 'docs/experiments/aot-fault-details/render.c',
                ROOT / 'tools/native/entry-arguments.c', ROOT / 'runtime/native/async/native-runtime.c']
    aot_host = ROOT / 'benchmarks/native-web/http-host.c'
    jit_host = ROOT / 'docs/experiments/jit-managed/host.c'
    sdk = os.environ.get('SDKROOT') or subprocess.check_output(['xcrun', '--show-sdk-path'], text=True).strip()
    link = ['clang', '-isysroot', sdk, '-arch', 'arm64', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror',
            '-Wno-deprecated-declarations', '-DNEOCLR_NATIVE_GC', '-DNEOCLR_RUNTIME_ASYNC',
            '-DNEOCLR_HTTP_HEAP_BYTES=65536', '-I', ROOT / 'runtime/native/async', '-I', base]
    runs = []
    binaries = []
    for paced in [False, True]:
        policy = 'paced' if paced else 'eager'
        options = ['-DNEOCLR_GC_PACED'] if paced else []
        executable = output / ('aot-' + policy)
        run('link-aot-' + policy, [*link, *options, '-DNEOCLR_HTTP_INVOCATIONS=8',
                                 '-fsanitize=undefined,bounds', aot_host, *sources, obj, '-o', executable])
        run('aot-' + policy, [executable], 42)
        assert not (output / ('aot-' + policy + '.stderr')).read_text(), 'Native fault or sanitizer output'
        dylib = output / ('jit-' + policy + '.dylib')
        run('link-jit-' + policy, [*link, *options, '-dynamiclib', jit_host, *sources, '-o', dylib])
        run('jit-' + policy, [codegen, '--jit-managed', '--closed-world', artifact, '@entry', dylib,
                             *context, *flags])
        lines = (output / ('jit-' + policy + '.stderr')).read_text().splitlines()
        assert len(lines) == 1 and lines[0].startswith('JIT_RESULT '), lines
        measurement = json.loads(lines[0].removeprefix('JIT_RESULT '))
        assert measurement['guestExitCode'] == 42, measurement
        runs.append({'policy': policy, 'aotInvocations': 8, 'jit': measurement})
        binaries += [executable, dylib]
    files = [Path(__file__).resolve(), compiler, runtime, codegen, project_file, project / 'Main.rvn',
             artifact, obj, aot_host, jit_host, *sources, *binaries]
    files += sorted((ROOT / 'runtime/native').rglob('*.[ch]'))
    files += sorted(bundle.glob('*.dll')) + [bundle / name for name in ('System.runtime.neox', 'ownership.json', 'build-evidence.json')]
    files += sorted(compiler.parent.glob('Raven*.dll')) + sorted(compiler.parent.glob('NeoCLR*.dll'))
    report = {'schemaVersion': 1, 'fixture': fixture, 'platform': platform.platform(),
              'runtimeRevision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              'compilerRevision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=compiler.parent, text=True).strip(),
              'bundle': str(bundle), 'commands': commands, 'imports': imports, 'runs': runs,
              'checks': ['ten completed/queued/async-forwarded alternative and identity checks per entry',
                         'interpreter exit 42', '16 AOT entries with undefined/bounds sanitizers',
                         '2 managed JIT entries; cleanup before generated code release',
                         '65536-byte native heap; eager/paced collector policies',
                         'host task/socket scopes, roots and heap cleaned after each entry'],
              'limitations': ['macOS ARM64 only', 'bounded serial invocation; no escaped callbacks',
                              'JIT host not sanitizer-instrumented', 'no throughput or exact allocation claim'],
              'files': {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in files}}
    (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'result': 'PASS', 'evidence': str(output / 'results.json')}))


if __name__ == '__main__':
    main()

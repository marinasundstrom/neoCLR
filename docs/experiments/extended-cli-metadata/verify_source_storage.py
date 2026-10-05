"""Build unchanged storage sources, then run the artifact-only storage sample."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'library', 'ownership', 'seed', 'core', 'runtime', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    compiler, library, ownership, seed, core, runtime, output = (
        getattr(args, name).resolve() for name in
        ('compiler', 'library', 'ownership', 'seed', 'core', 'runtime', 'output'))
    output.mkdir(parents=True, exist_ok=False)
    fixture = output / 'storage-demo'
    (fixture / 'examples').mkdir(parents=True)
    common = ['dotnet', compiler, 'neoclr', '--core-reference', core, '--runtime-seed', seed,
              '--bootstrap-intrinsics', '--bootstrap-ownership', ownership, '--reference', library]
    names = ('EntryKind', 'FileReadError', 'FileSystem', 'FileText', 'FileWriteError',
             'InvalidPathError', 'Metadata', 'Path', 'StorageItems', 'StorageLookupError', 'StorageProvider')
    sources = [ROOT / 'runtime/raven/src/System/Storage' / (name + '.rvn') for name in names]
    sources += [ROOT / 'runtime/raven/native' / (name + '.rvn') for name in
                ('RuntimeStorageCalls', 'RuntimeStorageServices', 'RuntimeFileTextServices')]
    sample = ROOT / 'docs/experiments/storage-poc/Main.rvn'
    storage, app = output / 'Storage.dll', output / 'StorageSample.dll'
    commands = []

    def run(command, expected_output=None):
        command = [str(part) for part in command]
        result = subprocess.run(command, cwd=output, capture_output=True, text=True, timeout=180)
        commands.append(dict(command=command, cwd=str(output), exitCode=result.returncode,
                             stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != 0 or expected_output is not None and result.stdout != expected_output:
            raise RuntimeError(json.dumps(commands[-1], indent=2))

    run(common + ['--library', '-o', storage] + sources)
    run(common + ['--reference', storage, '-o', app, sample])
    dependencies = ['--system', seed, '--module', library, '--module', storage]
    run([runtime, 'verify', app] + dependencies)
    run([runtime, 'run', app, '--instructions', '100000000'] + dependencies,
        'Created message.txt\nRead: Hello, värld!\nRe-read after seek: Hello, värld!\n'
        'Existing file: AlreadyExists\nItems:\ndirectory: examples\nfile: message.txt\n'
        'Directory: /examples\nItem: message.txt\nMissing file: NotFound\n')
    text_sample = Path(__file__).resolve().parent / 'bootstrap/storage-text-consumer.rvn'
    text_app = output / 'StorageText.dll'
    run(common + ['--reference', storage, '-o', text_app, text_sample])
    run([runtime, 'verify', text_app] + dependencies)
    run([runtime, 'run', text_app, '--instructions', '100000000'] + dependencies,
        'Native bounded file text passed\n')
    assert (fixture / 'text.txt').read_bytes() == 'värld'.encode('utf-8')
    message = fixture / 'message.txt'
    assert message.read_bytes() == 'Hello, värld!'.encode('utf-8')
    inputs = [library, ownership, seed, core, compiler, runtime, Path(__file__), sample, storage, app, text_sample, text_app] + sources
    inputs += [compiler.parent / name for name in
               ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    revision = lambda path: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=path, text=True).strip()
    evidence = dict(runtimeRevision=revision(ROOT), compilerRevision=revision(compiler.parent),
                    instructionBudget=100000000, commands=commands,
                    fileContentUtf8=message.read_text(),
                    hashes={str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in inputs})
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()

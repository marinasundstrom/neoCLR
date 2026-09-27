"""Verify Raven entry return shapes, async dispatch and observable process results."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
for name in ('runtime', 'bridge', 'system', 'reference'):
    parser.add_argument('--' + name, required=True, type=Path)
parser.add_argument('--evidence', type=Path)
parser.add_argument('--case', action='append')
args = parser.parse_args()
artifacts = {name: getattr(args, name).resolve() for name in ('runtime', 'bridge', 'system', 'reference')}
here = Path(__file__).resolve().parent
prefix = 'import System.*\nimport System.Tasks.*\n'


def run(command):
    return subprocess.run([str(x) for x in command], capture_output=True, text=True, timeout=180)


def require(result, code=0, stdout=None, stderr=None):
    assert result.returncode == code, (result.returncode, code, result.stdout, result.stderr)
    if stdout is not None:
        assert result.stdout == stdout, (result.stdout, stdout)
    if stderr is not None:
        assert result.stderr == stderr, (result.stderr, stderr)


cases = [
    ('unit', 'func Main() { Console.WriteLine("unit") }', 0, 'unit\n', ''),
    ('unit-args', 'func Main(args: string[]) { Console.WriteLine(args.Length) }', 0, '2\n', ''),
    ('int', 'func Main() -> int { return 7 }', 7, '', ''),
    ('int-args', 'func Main(args: string[]) -> int { return args.Length }', 2, '', ''),
    ('result-int', 'func Main() -> Result<int, string> { return .Ok(9) }', 9, '', ''),
    ('result-error', 'func Main() -> Result<int, string> { return .Error("failure") }', 1, '', 'failure\n'),
    ('result-unit', 'func Main() -> Result<(), string> { return .Ok(()) }', 0, '', ''),
    ('result-unit-error', 'func Main() -> Result<(), int> { return .Error(17) }', 1, '', '17\n'),
    ('task-int', 'async func Main() -> Task<int> { return 11 }', 11, '', ''),
    ('task-unit', 'async func Main() -> Task<()> { return () }', 0, '', ''),
    ('task-result', 'async func Main() -> Task<Result<int, string>> { return .Ok(13) }', 13, '', ''),
    ('task-result-error', 'async func Main() -> Task<Result<int, string>> { return .Error("async failure") }', 1, '', 'async failure\n'),
    ('task-result-unit', 'async func Main() -> Task<Result<(), string>> { return .Ok(()) }', 0, '', ''),
    ('task-result-unit-error', 'async func Main() -> Task<Result<(), string>> { return .Error("unit failure") }', 1, '', 'unit failure\n'),
    ('pending-args', (here / 'Main.rvn').read_text(), 0, 'Completed with 2\n', ''),
    ('pending-int', '''async func Main() -> Task<int> {
        let p = Promise<int>()
        TaskQueue.Default.Post(() => { _ = p.Complete(19) })
        return await p.Task
    }''', 19, '', ''),
    ('sync-task', '''func Main() -> Task<int> {
        let p = Promise<int>()
        TaskQueue.Default.Post(() => { _ = p.Complete(23) })
        return p.Task
    }''', 23, '', ''),
]
for name, expected in [('library-async-default-queue', 'Hello on a worker\n'), ('library-async', 'Suspended\n42\n')]:
    cases.append((name, (here.parent / 'raven-target/samples' / (name + '.rvn')).read_text(), 0, expected, ''))
results = []
with tempfile.TemporaryDirectory(prefix='neoclr-entry-results-') as folder:
    root = Path(folder)
    shutil.copyfile(here / 'Contracts.rvnproj', root / 'Contracts.rvnproj')
    shutil.copyfile(artifacts['reference'], root / 'NeoCLR.CoreProbe.dll')
    for name, source, code, stdout, stderr in cases:
        if args.case and name not in args.case:
            continue
        (root / 'Main.rvn').write_text(prefix + source + '\n')
        output = root / name
        compiled = run(['dotnet', artifacts['bridge'], '--project', root / 'Contracts.rvnproj', output])
        require(compiled)
        app = output / 'App.neoil'
        require(run([artifacts['runtime'], 'verify', app, '--system', artifacts['system']]))
        require(run([artifacts['runtime'], 'run', app, '--system', artifacts['system'], '--', 'café', '']), code, stdout, stderr)
        results.append(name)
        print(name + ': passed', flush=True)
    for name, source, diagnostic in [
        ('cancelled', 'func Main() -> Task<int> { let p = Promise<int>(); _ = p.Cancel(); return p.Task }', 'Task is cancelled'),
        ('unresolved', 'func Main() -> Task<int> { return Promise<int>().Task }', 'Task is still pending'),
    ]:
        if args.case and name not in args.case:
            continue
        (root / 'Main.rvn').write_text(prefix + source + '\n')
        output = root / name
        require(run(['dotnet', artifacts['bridge'], '--project', root / 'Contracts.rvnproj', output]))
        result = run([artifacts['runtime'], 'run', output / 'App.neoil', '--system', artifacts['system']])
        require(result, 1, '')
        assert diagnostic in result.stderr, result.stderr
        results.append(name)
        print(name + ': expected fault', flush=True)
    for name, signature in [('bool', 'bool'), ('task-string', 'Task<string>'), ('result-string', 'Result<string,string>'), ('inactive-result', 'Result<int,string>')]:
        if args.case and name not in args.case:
            continue
        (root / 'Main.rvn').write_text(prefix + f'func Main() -> {signature} {{ return default }}\n')
        result = run(['dotnet', artifacts['bridge'], '--project', root / 'Contracts.rvnproj', root / name])
        expected = ('RAV0406',) if name == 'inactive-result' else ('RAV1022', 'RAV1014')
        assert result.returncode != 0 and any(code in result.stderr for code in expected), result.stderr
        results.append('reject-' + name)
        print(name + ': rejected', flush=True)
if args.evidence:
    args.evidence.write_text(json.dumps({'passed': results, 'sourceSha256': {name: hashlib.sha256((prefix + source + '\n').encode()).hexdigest() for name, source, *_ in cases}, 'sha256': {name: hashlib.sha256(path.read_bytes()).hexdigest() for name, path in artifacts.items()}}, indent=2) + '\n')

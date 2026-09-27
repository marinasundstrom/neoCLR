"""Focused public Task.Run consumer: captures, unit/typed results, unwrap and faults."""
import argparse
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
for name in ('runtime', 'bridge', 'system', 'reference'):
    parser.add_argument('--' + name, type=Path, required=True)
parser.add_argument('--case', action='append', help='Run only selected consumer cases')
args = parser.parse_args()
tools = {name: getattr(args, name).resolve() for name in ('runtime', 'bridge', 'system', 'reference')}
here = Path(__file__).resolve().parent
prefix = 'import System.*\nimport System.Tasks.*\nalias Task = System.Tasks.Task\n'
cases = [
    ('block-lambda', (here / 'BlockLambda.rvn').read_text(), 42, '', None),
    ('unqualified-run', (here / 'Unqualified.rvn').read_text(), 42, '', None),
    ('unit-await', (here / 'UnitAwait.rvn').read_text(), 0, 'completed\n', None),
    ('mutable-capture', (here / 'MutableCapture.rvn').read_text(), 42, '', None),
    ('mutable-capture-inline', (here / 'MutableCaptureInline.rvn').read_text(), 42, '', None),
    ('capture-and-unwrap', (here / 'Main.rvn').read_text(), 0, '42\nTrue\n41\n21\n7\n', None),
    ('unobserved-work', prefix + 'func Main() { _ = Task.Run(() => Console.WriteLine("finished")) }', 0, 'finished\n', None),
    ('unwrap-unit', prefix + '''func Main() -> Task<int> {
        return Task.Run(Complete).Map(_ => 17)
    }
    func Complete() -> Task<()> {
        return Task.Run(() => ())
    }''', 17, '', None),
    ('cancelled-inner', prefix + '''func Main() -> Task<int> {
        return Task.Run(Cancelled)
    }
    func Cancelled() -> Task<int> {
        let source = Promise<int>()
        _ = source.Cancel()
        return source.Task
    }''', 1, '', 'Task is cancelled'),
    ('callback-fault', prefix + '''func Main() -> Task<int> {
        return Task.Run(Fail)
    }
    func Fail() -> int {
        System.Fault("task-run-test-fault")
        return 0
    }''', 1, '', 'task-run-test-fault'),
]

def run(command):
    return subprocess.run([str(part) for part in command], capture_output=True, text=True, timeout=180)

with tempfile.TemporaryDirectory(prefix='neoclr-task-run-') as directory:
    root = Path(directory)
    shutil.copyfile(here.parent / 'entry-results/Contracts.rvnproj', root / 'Contracts.rvnproj')
    shutil.copyfile(tools['reference'], root / 'NeoCLR.CoreProbe.dll')
    for name, source, code, output, error in cases:
        if args.case and name not in args.case:
            continue
        (root / 'Main.rvn').write_text(source)
        build = root / name
        compiled = run(['dotnet', tools['bridge'], '--project', root / 'Contracts.rvnproj', build])
        assert compiled.returncode == 0, (name, compiled.stdout, compiled.stderr)
        app = build / 'App.neoil'
        verified = run([tools['runtime'], 'verify', app, '--system', tools['system']])
        assert verified.returncode == 0, (name, verified.stdout, verified.stderr)
        result = run([tools['runtime'], 'run', app, '--system', tools['system']])
        assert result.returncode == code and result.stdout == output, (name, result.returncode, result.stdout, result.stderr)
        assert (error in result.stderr if error else result.stderr == ''), (name, result.stderr)
        print(name + ': passed', flush=True)

    if not args.case or 'generic-capture-import-gap' in args.case:
        (root / 'Main.rvn').write_text((here / 'compiler-gaps/GenericCapture.rvn').read_text())
        compiled = run(['dotnet', tools['bridge'], '--project', root / 'Contracts.rvnproj', root / 'generic-capture'])
        assert (root / 'generic-capture/App.raw.dll').stat().st_size > 0
        expected = 'Unsupported Result profile type: NamespaceMembers/<>c__AsyncStateMachine'
        assert compiled.returncode != 0 and expected in compiled.stderr, (compiled.stdout, compiled.stderr)
        assert '<System.Int32>' in compiled.stderr, compiled.stderr
        print('generic-capture-import-gap: confirmed current importer limitation', flush=True)

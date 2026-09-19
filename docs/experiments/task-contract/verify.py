"""Run experimental completion/continuation contracts on neoCLR, without async lowering."""
import argparse
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve().parent
BRIDGE = HERE.parent / 'raven-target'
sys.path.insert(0, str(BRIDGE))
from runner_options import add_toolchain_arguments, runner_arguments

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('project', type=Path)
add_toolchain_arguments(parser)
parser.add_argument('--runtime', type=Path, required=True)
args = parser.parse_args()
model = (HERE / 'completion.rvn').read_text().split('func Main() {')[0]
cases = {
    'Immediate completion': '''
    let executor = Executor()
    let source = Completion(executor)
    let destination = Completion(executor)
    Check(source.TryComplete(Ok(2)))
    Start(source, destination)
    Check(destination.IsCompleted)
    Read(destination, 42)
    Read(destination, 42)
''',
    'Pending state and multiple consumers survive collection': '''
    let executor = Executor()
    let source = Completion(executor)
    let first = Completion(executor)
    let second = Completion(executor)
    Start(source, first)
    Start(source, second)
    Check(!first.IsCompleted)
    Check(!second.IsCompleted)
    var remaining = 1000
    while remaining != 0 {
        ArrayList<int>()
        remaining = remaining - 1
    }
    Check(source.TryComplete(Ok(2)))
    Check(!first.IsCompleted)
    Check(!second.IsCompleted)
    executor.Drain()
    Read(first, 42)
    Read(second, 42)
''',
    'Expected error is a completion value': '''
    let executor = Executor()
    let source = Completion(executor)
    let destination = Completion(executor)
    Start(source, destination)
    Check(source.TryComplete(Result<int, string>.Error("Expected failure")))
    executor.Drain()
    Check(destination.IsCompleted)
    match destination.GetResult() {
        Ok(_) => System.Fault("Lost expected error")
        Error(let message) => Check(message == "Expected failure")
    }
''',
    'Duplicate completion cannot replace the result': '''
    let executor = Executor()
    let source = Completion(executor)
    Check(source.TryComplete(Ok(1)))
    Check(!source.TryComplete(Ok(2)))
    Read(source, 1)
''',
    'Registration order and late registration are queued': '''
    let executor = Executor()
    let source = Completion(executor)
    var order = 0
    source.OnCompleted(() => { order = order * 10 + 1 })
    source.OnCompleted(() => { order = order * 10 + 2 })
    Check(source.TryComplete(Ok(0)))
    source.OnCompleted(() => { order = order * 10 + 3 })
    Check(order == 0)
    executor.Drain()
    Check(order == 123)
    executor.Drain()
    Check(order == 123)
''',
    'Nested completion cannot reenter consumers': '''
    let executor = Executor()
    let first = Completion(executor)
    let second = Completion(executor)
    var order = 0
    second.OnCompleted(() => { order = order * 10 + 3 })
    first.OnCompleted(() => {
        order = order * 10 + 1
        Check(second.TryComplete(Ok(0)))
        Check(order == 1)
    })
    first.OnCompleted(() => { order = order * 10 + 2 })
    Check(first.TryComplete(Ok(0)))
    executor.Drain()
    Check(order == 123)
''',
}
faults = {
    'Pending read never blocks': ('Completion is still pending', '''
    let source = Completion(Executor())
    source.GetResult()
'''),
    'Recursive executor pump is rejected': ('Executor cannot be pumped recursively', '''
    let executor = Executor()
    executor.Post(() => { executor.Drain() })
    executor.Drain()
'''),
    'Callback faults stay terminal': ('Terminal callback', '''
    let executor = Executor()
    executor.Post(() => { System.Fault("Terminal callback") })
    executor.Drain()
'''),
}
results = {}
with tempfile.TemporaryDirectory(prefix='neoclr-task-contract-') as temporary:
    root = Path(temporary)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    command = [sys.executable, str(BRIDGE / 'run_project.py'), str(root / 'Demo.rvnproj'),
               *runner_arguments(args), '--runtime', str(args.runtime.resolve()), '--build-only']

    def execute(source):
        (root / 'Main.rvn').write_text(source)
        built = subprocess.run(command, capture_output=True, text=True, timeout=120)
        assert built.returncode == 0, built.stdout + built.stderr
        artifact = next(line.split(': ', 1)[1] for line in built.stdout.splitlines()
                        if line.startswith('Verified saved project:'))
        return subprocess.run([str(args.runtime.resolve()), 'run', artifact, '--system',
                               str(Path(artifact).parent / 'System.Collections.neoil'), '--gc-stats'],
                              capture_output=True, text=True, timeout=30)

    for label, body in cases.items():
        run = execute(model + 'func Main() {\n' + body + '\n    WriteLine("Passed")\n}\n')
        assert run.returncode == 0 and run.stdout == 'Passed\n', run.stdout + run.stderr
        if 'survive collection' in label:
            collections = int(re.search(r'collections=(\d+)', run.stderr).group(1))
            assert collections > 0, run.stderr
            results[label] = {'passed': True, 'collections': collections}
        else:
            results[label] = 'passed'
    # Isolate the same completion storage/queue with a nominal unit payload.
    completion_only = model.split('// Heap identity')[0].replace('Result<int, string>', 'Result<unit, string>')
    run = execute(completion_only + '''
func Main() {
    let executor = Executor()
    let source = Completion(executor)
    source.OnCompleted(() => {
        match source.GetResult() {
            Ok(_) => WriteLine("Completed unit")
            Error(_) => System.Fault("Expected completion")
        }
    })
    source.TryComplete(Ok(()))
    executor.Drain()
}
''')
    assert run.returncode == 0 and run.stdout == 'Completed unit\n', run.stdout + run.stderr
    results['Unit payload uses the same completion model'] = 'passed'
    for label, (message, body) in faults.items():
        run = execute(model + 'func Main() {\n' + body + '\n    WriteLine("Must not continue")\n}\n')
        assert run.returncode != 0 and message in run.stderr, run.stdout + run.stderr
        assert 'Must not continue' not in run.stdout
        results[label] = 'faulted as expected'
print(json.dumps(results, indent=2))

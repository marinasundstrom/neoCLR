"""Execute the Task Map/Then composition on neoCLR; no async lowering."""
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
prelude = '''import System.*
import System.Collections.*
import System.Tasks.*
import System.Option.*
import System.Result.*
import System.Tasks.TaskOutcome.*
import System.Console.*

func Check(value: bool) {
    if !value {
        System.Fault("Task composition assertion failed")
    }
}

func FailMap(value: int) -> int {
    System.Fault("Map callback fault")
    return value
}

func FailThen(value: int) -> Task<int> {
    System.Fault("Then callback fault")
    return Promise<int>(TaskQueue()).Task
}
'''
cases = {
    'Map waits for pending input and queues its transform': '''
    let queue = TaskQueue()
    let promise = Promise<int>(queue)
    var calls = 0
    let mapped = promise.Task.Map((value: int) -> string => {
        calls = calls + 1
        return value.ToString()
    })
    Check(mapped.State == TaskState.Pending)
    promise.Complete(42)
    Check(calls == 0)
    Check(mapped.State == TaskState.Pending)
    queue.Drain()
    Check(calls == 1)
    Check(mapped.GetResult() == "42")
    queue.Drain()
    Check(calls == 1)
''',
    'Late Map registration is queued': '''
    let queue = TaskQueue()
    let promise = Promise<int>(queue)
    promise.Complete(41)
    let mapped = promise.Task.Map(value => value + 1)
    Check(mapped.State == TaskState.Pending)
    queue.Drain()
    Check(mapped.GetResult() == 42)
''',
    'Map and Then skip callbacks after cancellation': '''
    let queue = TaskQueue()
    let promise = Promise<int>(queue)
    let mapped = promise.Task.Map(FailMap)
    let chained = promise.Task.Then(FailThen)
    promise.Cancel()
    let late = promise.Task.Map(FailMap)
    Check(mapped.State == TaskState.Pending)
    queue.Drain()
    Check(mapped.Outcome is Some(Cancelled))
    Check(chained.Outcome is Some(Cancelled))
    Check(late.Outcome is Some(Cancelled))
''',
    'Then flattens a pending task': '''
    let queue = TaskQueue()
    let first = Promise<int>(queue)
    let second = Promise<string>(queue)
    var calls = 0
    let chained = first.Task.Then((value: int) -> Task<string> => {
        Check(value == 42)
        calls = calls + 1
        return second.Task
    })
    first.Complete(42)
    queue.Drain()
    Check(calls == 1)
    Check(chained.State == TaskState.Pending)
    second.Complete("Done")
    Check(chained.State == TaskState.Pending)
    queue.Drain()
    Check(chained.GetResult() == "Done")
    Check(!second.Cancel())
''',
    'Then queues completed inputs and completed inner tasks': '''
    let queue = TaskQueue()
    let first = Promise<int>(queue)
    let second = Promise<int>(queue)
    first.Complete(1)
    second.Complete(42)
    let chained = first.Task.Then(value => second.Task)
    Check(chained.State == TaskState.Pending)
    queue.Drain()
    Check(chained.GetResult() == 42)
''',
    'Then propagates pending inner cancellation': '''
    let queue = TaskQueue()
    let first = Promise<int>(queue)
    let second = Promise<int>(queue)
    let chained = first.Task.Then(value => second.Task)
    first.Complete(42)
    queue.Drain()
    Check(chained.State == TaskState.Pending)
    second.Cancel()
    queue.Drain()
    Check(chained.Outcome is Some(Cancelled))
''',
    'Then propagates already cancelled inner tasks': '''
    let queue = TaskQueue()
    let first = Promise<int>(queue)
    let second = Promise<int>(queue)
    second.Cancel()
    let chained = first.Task.Then(value => second.Task)
    first.Complete(42)
    queue.Drain()
    Check(chained.Outcome is Some(Cancelled))
''',
    'Result Error is passed to both operators as a normal value': '''
    let queue = TaskQueue()
    let first = Promise<Result<int, string>>(queue)
    let second = Promise<Result<int, string>>(queue)
    var calls = 0
    let result = first.Task.Map((value: Result<int, string>) -> Result<int, string> => {
        Check(value is Error(_))
        calls = calls + 1
        return value
    }).Then((value: Result<int, string>) -> Task<Result<int, string>> => {
        Check(value is Error(_))
        calls = calls + 1
        second.Complete(value)
        return second.Task
    })
    first.Complete(Error("Unavailable"))
    queue.Drain()
    Check(calls == 2)
    Check(result.State == TaskState.Completed)
    if result.GetResult() is Error(let message) {
        Check(message == "Unavailable")
    } else {
        System.Fault("Result was changed")
    }
''',
    'Unit is an ordinary mapped payload': '''
    let queue = TaskQueue()
    let first = Promise<int>(queue)
    let mapped = first.Task.Map(value => ())
    first.Complete(42)
    queue.Drain()
    Check(mapped.State == TaskState.Completed)
    Check(mapped.Outcome is Some(Completed(_)))
''',
    'Cross queue composition requires both queues': '''
    let origin = TaskQueue()
    let inner = TaskQueue()
    let first = Promise<int>(origin)
    let second = Promise<int>(inner)
    let chained = first.Task.Then(value => second.Task)
    let mapped = chained.Map(value => value + 1)
    first.Complete(41)
    origin.Drain()
    second.Complete(41)
    origin.Drain()
    Check(chained.State == TaskState.Pending)
    inner.Drain()
    Check(chained.GetResult() == 41)
    Check(mapped.State == TaskState.Pending)
    origin.Drain()
    Check(mapped.GetResult() == 42)
''',
    'Generic continuation objects survive collection': '''
    let queue = TaskQueue()
    let first = Promise<int>(queue)
    let second = Promise<string>(queue)
    let chained = first.Task.Map(value => value + 1).Then((value: int) -> Task<string> => {
        Check(value == 42)
        return second.Task
    }).Map(value => value.Length)
    var remaining = 1000
    while remaining != 0 {
        ArrayList<int>()
        remaining = remaining - 1
    }
    first.Complete(41)
    queue.Drain()
    remaining = 1000
    while remaining != 0 {
        ArrayList<int>()
        remaining = remaining - 1
    }
    second.Complete("Done")
    queue.Drain()
    Check(chained.GetResult() == 4)
''',
}
faults = {
    'Map callback faults remain terminal': ('Map callback fault', '''
    let queue = TaskQueue()
    let source = Promise<int>(queue)
    let task = source.Task.Map(FailMap)
    source.Complete(42)
    queue.Drain()
'''),
    'Then callback faults remain terminal': ('Then callback fault', '''
    let queue = TaskQueue()
    let source = Promise<int>(queue)
    let task = source.Task.Then(FailThen)
    source.Complete(42)
    queue.Drain()
'''),
}
rejections = {
    'Dispatcher is internal': 'source.Task.Dispatcher()',
    'Promise dispatcher is internal': 'source.Dispatcher()',
}
results = {}
with tempfile.TemporaryDirectory(prefix='neoclr-task-composition-') as temporary:
    root = Path(temporary)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    command = [sys.executable, str(BRIDGE / 'run_project.py'), str(root / 'Demo.rvnproj'),
               *runner_arguments(args), '--runtime', str(args.runtime.resolve()), '--build-only']

    def build(body):
        (root / 'Main.rvn').write_text(prelude + '\nfunc Main() {\n' + body + '\n    WriteLine("Passed")\n}\n')
        return subprocess.run(command, capture_output=True, text=True, timeout=120)

    def execute(body):
        built = build(body)
        assert built.returncode == 0, built.stdout + built.stderr
        artifact = next(line.split(': ', 1)[1] for line in built.stdout.splitlines()
                        if line.startswith('Verified saved project:'))
        return subprocess.run([str(args.runtime.resolve()), 'run', artifact, '--system',
                               str(Path(artifact).parent / 'System.Collections.neoil'), '--gc-stats'],
                              capture_output=True, text=True, timeout=30)

    for label, body in cases.items():
        print("Checking: " + label, file=sys.stderr, flush=True)
        run = execute(body)
        assert run.returncode == 0 and run.stdout == 'Passed\n', label + ': ' + run.stdout + run.stderr
        results[label] = 'passed'
        if 'survive collection' in label:
            collections = int(re.search(r'collections=(\d+)', run.stderr).group(1))
            assert collections > 0, run.stderr
            results[label] = {'passed': True, 'collections': collections}
    for label, (message, body) in faults.items():
        run = execute(body)
        assert run.returncode != 0 and message in run.stderr and 'Passed' not in run.stdout, run.stdout + run.stderr
        results[label] = 'faulted as expected'
    for label, expression in rejections.items():
        built = build('    let source = Promise<int>(TaskQueue())\n    ' + expression)
        assert built.returncode != 0 and ('RAV' in built.stdout + built.stderr), built.stdout + built.stderr
        results[label] = 'rejected'
print(json.dumps(results, indent=2))

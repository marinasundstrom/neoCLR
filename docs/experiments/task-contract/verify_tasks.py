"""Execute the provisional generic Task/TCS library on neoCLR; no async lowering."""
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
import System.Result.*
import System.Console.*

public func Check(value: bool) {
    if !value {
        System.Fault("Task assertion failed")
    }
}

public func Begin(queue: TaskQueue) -> Task<int> {
    let source = TaskCompletionSource<int>(queue)
    queue.Post(() => { source.TrySetResult(42) })
    return source.Task
}
'''
cases = {
    'Consumer identity and first completion wins': '''
    let queue = TaskQueue()
    let source = TaskCompletionSource<int>(queue)
    let task = source.Task
    let alias = source.Task
    Check(!task.IsCompleted)
    Check(source.TrySetResult(42))
    Check(task.IsCompleted)
    Check(alias.IsCompleted)
    Check(!source.TrySetResult(99))
    Check(task.GetResult() == 42)
    Check(task.GetResult() == 42)
''',
    'Pending, multiple and late callbacks are queued in order': '''
    let queue = TaskQueue()
    let source = TaskCompletionSource<int>(queue)
    let task = source.Task
    var order = 0
    task.OnCompleted(() => {
        Check(task.GetResult() == 42)
        order = order * 10 + 1
    })
    task.OnCompleted(() => { order = order * 10 + 2 })
    Check(source.TrySetResult(42))
    task.OnCompleted(() => { order = order * 10 + 3 })
    Check(order == 0)
    queue.Drain()
    Check(order == 123)
    queue.Drain()
    Check(order == 123)
''',
    'Nested completion does not reenter a consumer': '''
    let queue = TaskQueue()
    let first = TaskCompletionSource<int>(queue)
    let second = TaskCompletionSource<int>(queue)
    var order = 0
    second.Task.OnCompleted(() => { order = order * 10 + 3 })
    first.Task.OnCompleted(() => {
        order = order * 10 + 1
        second.TrySetResult(2)
        Check(order == 1)
    })
    first.Task.OnCompleted(() => { order = order * 10 + 2 })
    first.TrySetResult(1)
    queue.Drain()
    Check(order == 123)
''',
    'Returned task and producer survive collection after API returns': '''
    let queue = TaskQueue()
    let task = Begin(queue)
    Check(!task.IsCompleted)
    var remaining = 1000
    while remaining != 0 {
        ArrayList<int>()
        remaining = remaining - 1
    }
    queue.Drain()
    Check(task.IsCompleted)
    Check(task.GetResult() == 42)
''',
    'String and Boolean payloads': '''
    let queue = TaskQueue()
    let text = TaskCompletionSource<string>(queue)
    let flag = TaskCompletionSource<bool>(queue)
    text.TrySetResult("Ready")
    flag.TrySetResult(true)
    Check(text.Task.GetResult() == "Ready")
    Check(flag.Task.GetResult())
''',
    'Result failure is an ordinary completed payload': '''
    let queue = TaskQueue()
    let source = TaskCompletionSource<Result<int, string>>(queue)
    var called = false
    source.Task.OnCompleted(() => {
        match source.Task.GetResult() {
            Ok(_) => System.Fault("Expected Error payload")
            Error(let message) => Check(message == "Unavailable")
        }
        called = true
    })
    Check(source.TrySetResult(Error("Unavailable")))
    Check(source.Task.IsCompleted)
    Check(!called)
    queue.Drain()
    Check(called)
''',
    'Unit is a normal payload': '''
    let queue = TaskQueue()
    let source = TaskCompletionSource<unit>(queue)
    var called = false
    source.Task.OnCompleted(() => { called = true })
    Check(source.TrySetResult(()))
    Check(!source.TrySetResult(()))
    source.Task.GetResult()
    queue.Drain()
    Check(called)
''',
    'Reference payload and pending continuation survive collection': '''
    let queue = TaskQueue()
    let source = TaskCompletionSource<ArrayList<int>>(queue)
    let task = source.Task
    var called = false
    task.OnCompleted(() => {
        Check(task.GetResult()[0] == 42)
        called = true
    })
    let value = ArrayList<int>()
    value.Add(42)
    source.TrySetResult(value)
    var remaining = 1000
    while remaining != 0 {
        ArrayList<int>()
        remaining = remaining - 1
    }
    queue.Drain()
    Check(called)
    value.Add(7)
    Check(task.GetResult()[1] == 7)
''',
}
faults = {
    'Pending read faults without blocking': ('Task is still pending', '''
    let source = TaskCompletionSource<int>(TaskQueue())
    source.Task.GetResult()
'''),
    'Recursive pumping faults': ('Task queue cannot be pumped recursively', '''
    let queue = TaskQueue()
    queue.Post(() => { queue.Drain() })
    queue.Drain()
'''),
    'Callback faults remain terminal': ('Terminal callback', '''
    let queue = TaskQueue()
    queue.Post(() => { System.Fault("Terminal callback") })
    queue.Drain()
'''),
}
rejections = {
    'Consumers cannot complete a Task': 'source.Task.TrySetResult(42)',
    'Task constructor is internal': 'Task<int>(source)',
    'Completion storage methods are internal': 'source.Read()',
    'Completion storage fields are private': 'source.slot.Add(42)',
}
results = {}
with tempfile.TemporaryDirectory(prefix='neoclr-generic-tasks-') as temporary:
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
        built = build('    let source = TaskCompletionSource<int>(TaskQueue())\n    ' + expression)
        assert built.returncode != 0 and ('RAV' in built.stdout + built.stderr), built.stdout + built.stderr
        results[label] = 'rejected'
print(json.dumps(results, indent=2))

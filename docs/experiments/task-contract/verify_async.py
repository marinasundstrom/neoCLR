"""Execute compiler-generated Task async on neoCLR's explicit queue."""
import argparse
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('project', type=Path)
parser.add_argument('--bridge', type=Path, required=True)
parser.add_argument('--system', type=Path, required=True)
parser.add_argument('--runtime', type=Path, required=True)
parser.add_argument('--case', help='Run cases whose label contains this text')
args = parser.parse_args()
prelude = '''import System.*
import System.Collections.*
import System.Tasks.*
import System.Result.*
import System.Console.*
public func Check(value: bool) {
    if !value { System.Fault("Async assertion failed") }
}
'''
cases = {
    'Pending and completed awaits': ('''
public async func Add(input: Task<int>) -> Task<int> {
    let seed = 40
    let first = await input
    let second = await input
    return seed + first + second
}
''', '''
    let queue = TaskQueue()
    let source = TaskCompletionSource<int>(queue)
    var answer = source.Task
    queue.Run(() => {
        answer = Add(source.Task)
        Check(!answer.IsCompleted)
        source.TrySetResult(1)
        Check(!answer.IsCompleted)
    })
    Check(answer.GetResult() == 42)
    queue.Run(() => { answer = Add(source.Task) })
    Check(answer.GetResult() == 42)
'''),
    'Two pending awaits retain state across GC': ('''
public async func Add(first: Task<int>, second: Task<int>) -> Task<int> {
    let seed = "retained"
    let a = await first
    let b = await second
    Check(seed == "retained")
    return a + b
}
''', '''
    let queue = TaskQueue()
    let first = TaskCompletionSource<int>(queue)
    let second = TaskCompletionSource<int>(queue)
    var answer = first.Task
    queue.Run(() => { answer = Add(first.Task, second.Task) })
    Check(!answer.IsCompleted)
    for i in 0..<1000 { let garbage = TaskQueue() }
    first.TrySetResult(20)
    queue.Drain()
    Check(!answer.IsCompleted)
    for i in 0..<1000 { let garbage = TaskQueue() }
    second.TrySetResult(22)
    queue.Drain()
    Check(answer.GetResult() == 42)
'''),
    'Unit remains a generic payload': ('''
public async func Complete(input: Task<unit>) -> Task<unit> {
    await input
    return ()
}
''', '''
    let queue = TaskQueue()
    let source = TaskCompletionSource<unit>(queue)
    var answer = source.Task
    queue.Run(() => {
        answer = Complete(source.Task)
        Check(!answer.IsCompleted)
        source.TrySetResult(())
    })
    Check(answer.IsCompleted)
    answer.GetResult()
'''),
    'Result propagation after await': ('''
public async func Read(input: Task<Result<int, string>>) -> Task<Result<int, string>> {
    let result = await input
    let value = result?
    return Ok(value + 1)
}
''', '''
    let queue = TaskQueue()
    let source = TaskCompletionSource<Result<int, string>>(queue)
    var answer = source.Task
    queue.Run(() => {
        answer = Read(source.Task)
        source.TrySetResult(Error("Unavailable"))
    })
    match answer.GetResult() {
        Ok(_) => System.Fault("Failure was lost")
        Error(let message) => Check(message == "Unavailable")
    }
'''),
    'Nested async composition': ('''
public async func Inner(input: Task<int>) -> Task<int> {
    let value = await input
    return value + 1
}
public async func Outer(input: Task<int>) -> Task<int> {
    let value = await Inner(input)
    return value + 1
}
''', '''
    let queue = TaskQueue()
    let source = TaskCompletionSource<int>(queue)
    var answer = source.Task
    queue.Run(() => {
        answer = Outer(source.Task)
        source.TrySetResult(40)
    })
    Check(answer.GetResult() == 42)
'''),
    'Awaitless async completion': ('''
public async func Ready() -> Task<int> { return 42 }
''', '''
    let queue = TaskQueue()
    let placeholder = TaskCompletionSource<int>(queue)
    var answer = placeholder.Task
    queue.Run(() => { answer = Ready() })
    Check(answer.GetResult() == 42)
'''),
}

cases['Result propagation before await skips pending input'] = ('''
public async func Read(initial: Result<int, string>, input: Task<int>) -> Task<Result<int, string>> {
    let a = initial?
    let b = await input
    return Ok(a + b)
}
''', '''
    let queue = TaskQueue()
    let input = TaskCompletionSource<int>(queue)
    let placeholder = TaskCompletionSource<Result<int, string>>(queue)
    var answer = placeholder.Task
    queue.Run(() => { answer = Read(Error("early"), input.Task) })
    Check(answer.IsCompleted)
    Check(!input.Task.IsCompleted)
    match answer.GetResult() {
        Ok(_) => System.Fault("Failure was lost")
        Error(let message) => Check(message == "early")
    }
''')
cases['Nested queue scopes restore outer context'] = ('''
public async func Ready() -> Task<int> { return 42 }
''', '''
    let outer = TaskQueue()
    let inner = TaskQueue()
    let placeholder = TaskCompletionSource<int>(outer)
    var answer = placeholder.Task
    outer.Run(() => {
        inner.Run(() => { Ready() })
        answer = Ready()
    })
    var observed = false
    answer.OnCompleted(() => { observed = true })
    inner.Drain()
    Check(!observed)
    outer.Drain()
    Check(observed)
''')

with tempfile.TemporaryDirectory(prefix='neoclr-generated-async-') as directory:
    root = Path(directory)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    for index, (label, (helpers, body)) in enumerate(cases.items()):
        if args.case and args.case.lower() not in label.lower():
            continue
        (root / 'Main.rvn').write_text(prelude + helpers + '\nfunc Main() {\n' + body + '\nWriteLine("Passed")\n}\n')
        output = root / str(index)
        built = subprocess.run(['dotnet', str(args.bridge.resolve()), '--project', str(root / 'Demo.rvnproj'), str(output)], capture_output=True, text=True, timeout=120)
        assert built.returncode == 0, label + ': ' + built.stdout + built.stderr
        run = subprocess.run([str(args.runtime.resolve()), 'run', str(output / 'App.neoil'), '--system', str(args.system.resolve()), '--gc-stats'], capture_output=True, text=True, timeout=30)
        assert run.returncode == 0 and run.stdout == 'Passed\n', label + ': ' + run.stdout + run.stderr
        if 'GC' in label:
            assert int(re.search(r'collections=(\d+)', run.stderr).group(1)) > 0, run.stderr
        print(label + ': Passed', flush=True)
    for index, (label, body, expected) in enumerate([
        ('Missing queue scope', 'Ready()', 'requires an active TaskQueue'),
        ('Recursive queue entry', 'let queue = TaskQueue()\nqueue.Run(() => { queue.Run(() => { }) })', 'cannot be pumped recursively'),
    ]):
        if args.case:
            continue
        (root / 'Main.rvn').write_text(prelude + '\npublic async func Ready() -> Task<int> { return 1 }\nfunc Main() {\n' + body + '\n}\n')
        output = root / ('fault-' + str(index))
        built = subprocess.run(['dotnet', str(args.bridge.resolve()), '--project', str(root / 'Demo.rvnproj'), str(output)], capture_output=True, text=True, timeout=120)
        assert built.returncode == 0, label + ': ' + built.stderr
        run = subprocess.run([str(args.runtime.resolve()), 'run', str(output / 'App.neoil'), '--system', str(args.system.resolve())], capture_output=True, text=True, timeout=30)
        assert run.returncode != 0 and expected in run.stderr, label + ': ' + run.stdout + run.stderr
        print(label + ': Passed', flush=True)

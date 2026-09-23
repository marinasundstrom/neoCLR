"""Verify invocation-owned default dispatch without application-side pumping."""
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
args = parser.parse_args()
prelude = '''import System.*
import System.Tasks.*
import System.Concurrency.*
import System.Console.*
public func Check(value: bool) { if !value { System.Fault("Default queue assertion failed") } }
'''
cases = [
    ('Promise defaults and observer order', '', '''
    let promise = Promise<int>()
    promise.Task.OnCompleted(() => WriteLine(promise.Task.GetResult()))
    promise.Complete(42)
    WriteLine("entry")
''', 'entry\n42\n'),
    ('Pending async completion', '''
public async func Read(input: Task<int>) -> Task<int> {
    let value = await input
    return value + 1
}
''', '''
    let promise = Promise<int>()
    let answer = Read(promise.Task)
    answer.OnCompleted(() => WriteLine(answer.GetResult()))
    TaskQueue.Default.Post(() => { promise.Complete(41) })
    Check(!answer.IsCompleted)
    WriteLine("entry")
''', 'entry\n42\n'),
    ('Async cancellation', '''
public async func Read(input: Task<int>) -> Task<int> {
    let value = await input
    System.Fault("Cancelled body continued")
    return value
}
''', '''
    let promise = Promise<int>()
    let answer = Read(promise.Task)
    answer.OnCompleted(() => { Check(answer.IsCancelled); WriteLine("cancelled") })
    TaskQueue.Default.Post(() => { promise.Cancel() })
''', 'cancelled\n'),
    ('Awaitless async needs no queue setup', '''
public async func Ready() -> Task<int> { return 42 }
''', '''
    let answer = Ready()
    Check(answer.GetResult() == 42)
    answer.OnCompleted(() => WriteLine("ready"))
''', 'ready\n'),
    ('Default queue retains work through GC', '''
public func Enqueue() {
    let promise = Promise<string>()
    promise.Task.OnCompleted(() => WriteLine(promise.Task.GetResult()))
    TaskQueue.Default.Post(() => { promise.Complete("retained") })
}
''', '''
    Enqueue()
    for i in 0..<1200 { let garbage = TaskQueue() }
''', 'retained\n'),
    ('Explicit queue is selected while active', '', '''
    let queue = TaskQueue()
    queue.Run(() => {
        let promise = Promise<int>()
        promise.Task.OnCompleted(() => WriteLine("explicit"))
        promise.Complete(1)
    })
    WriteLine("entry")
''', 'explicit\nentry\n'),
    ('Callbacks posted by callbacks also run', '', '''
    TaskQueue.Default.Post(() => {
        WriteLine("first")
        TaskQueue.Default.Post(() => WriteLine("last"))
    })
    TaskQueue.Default.Post(() => WriteLine("second"))
    WriteLine("entry")
''', 'entry\nfirst\nsecond\nlast\n'),
    ('Dedicated and pooled workers need no queue setup', '''
public func Echo(value: string) -> string => value + " worker"
''', '''
    let first = Thread.Run(Echo, "dedicated")
    let second = ThreadPool.Queue(Echo, "pooled")
    first.OnCompleted(() => WriteLine(first.GetResult()))
    second.OnCompleted(() => WriteLine(second.GetResult()))
''', 'dedicated worker\npooled worker\n'),
    ('Worker dispatch is isolated from caller dispatch', '''
public func Echo(value: string) -> string {
    TaskQueue.Default.Post(() => WriteLine("worker callback"))
    return value
}
''', '''
    let answer = Thread.Run(Echo, "caller callback")
    answer.OnCompleted(() => WriteLine(answer.GetResult()))
''', 'worker callback\ncaller callback\n'),
]
with tempfile.TemporaryDirectory(prefix='neoclr-default-queue-') as directory:
    root = Path(directory)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    for index, (label, helpers, body, expected) in enumerate(cases):
        (root / 'Main.rvn').write_text(prelude + helpers + '\nfunc Main() {\n' + body + '\n}\n')
        output = root / str(index)
        built = subprocess.run(['dotnet', str(args.bridge.resolve()), '--project', str(root / 'Demo.rvnproj'), str(output)], capture_output=True, text=True, timeout=120)
        assert built.returncode == 0, label + ': ' + built.stdout + built.stderr
        run = subprocess.run([str(args.runtime.resolve()), 'run', str(output / 'App.neoil'), '--system', str(args.system.resolve()), '--gc-stats'], capture_output=True, text=True, timeout=30)
        assert run.returncode == 0 and run.stdout == expected, label + ': ' + run.stdout + run.stderr
        if 'GC' in label:
            assert int(re.search(r'collections=(\d+)', run.stderr).group(1)) > 0, run.stderr
        print(label + ': Passed', flush=True)
    for index, (label, helpers, body, expected) in enumerate([
        ('Callback faults remain terminal', '', 'TaskQueue.Default.Post(() => System.Fault("callback fault"))', 'callback fault'),
        ('Automatic dispatch obeys execution budget', 'public func Again() { TaskQueue.Default.Post(Again) }', 'Again()', 'instruction limit exceeded'),
    ]):
        (root / 'Main.rvn').write_text(prelude + helpers + '\nfunc Main() {\n' + body + '\n}\n')
        output = root / ('fault-' + str(index))
        built = subprocess.run(['dotnet', str(args.bridge.resolve()), '--project', str(root / 'Demo.rvnproj'), str(output)], capture_output=True, text=True, timeout=120)
        assert built.returncode == 0, label + ': ' + built.stdout + built.stderr
        run = subprocess.run([str(args.runtime.resolve()), 'run', str(output / 'App.neoil'), '--system', str(args.system.resolve())], capture_output=True, text=True, timeout=30)
        assert run.returncode != 0 and expected in run.stderr, label + ': ' + run.stdout + run.stderr
        print(label + ': Passed', flush=True)

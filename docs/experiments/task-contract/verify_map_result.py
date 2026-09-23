"""Verify explicit Task<Result<T, E>> mapping and its terminal outcomes."""
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
prelude = """import System.*
import System.Tasks.*
import System.Result.*
import System.Console.*
func Check(value: bool) {
    if !value { System.Fault("MapResult assertion failed") }
}
"""
cases = [
    ('Pending Ok maps once across value types', '', """
    let source = Promise<Result<int, string>>()
    var calls = 0
    let mapped = source.Task.MapResult<string>(value => {
        calls = calls + 1
        return value.ToString()
    })
    Check(!mapped.IsCompleted)
    mapped.OnCompleted(() => {
        Check(calls == 1)
        if mapped.GetResult() is Ok(let value) { WriteLine(value) }
    })
    source.Complete(Ok(42))
    Check(calls == 0)
""", '42\n'),
    ('Already completed Error bypasses mapper', '', """
    let source = Promise<Result<int, string>>()
    source.Complete(Error("unavailable"))
    let mapped = source.Task.MapResult<int>(value => {
        System.Fault("Error invoked mapper")
        return value + 1
    })
    Check(!mapped.IsCompleted)
    mapped.OnCompleted(() => {
        if mapped.GetResult() is Error(let error) { WriteLine(error) }
    })
""", 'unavailable\n'),
    ('Cancellation bypasses mapper', '', """
    let source = Promise<Result<int, string>>()
    let mapped = source.Task.MapResult<int>(value => {
        System.Fault("Cancellation invoked mapper")
        return value + 1
    })
    mapped.OnCompleted(() => { Check(mapped.IsCancelled); WriteLine("cancelled") })
    source.Cancel()
""", 'cancelled\n'),
    ('Await composes with MapResult', """
async func Print(task: Task<Result<int, string>>) -> Task<unit> {
    let result = await task
    if result is Ok(let value) { WriteLine(value) }
    return ()
}
""", """
    let source = Promise<Result<int, string>>()
    Print(source.Task.MapResult(value => value + 1))
    source.Complete(Ok(41))
""", '42\n'),
    ('Explicit source queue is retained through GC', '', """
    let queue = TaskQueue()
    let source = Promise<Result<int, string>>(queue)
    let mapped = source.Task.MapResult(value => value + 1)
    mapped.OnCompleted(() => {
        if mapped.GetResult() is Ok(let value) { WriteLine(value) }
    })
    source.Complete(Ok(41))
    for i in 0..<1200 { let garbage = TaskQueue() }
    Check(!mapped.IsCompleted)
    queue.Drain()
""", '42\n'),
]
with tempfile.TemporaryDirectory(prefix='neoclr-mapresult-') as directory:
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
    (root / 'Main.rvn').write_text(prelude + """
func Main() {
    let source = Promise<Result<int, string>>()
    source.Task.MapResult<int>(value => {
        System.Fault("MapResult mapper fault")
        return value
    })
    source.Complete(Ok(42))
}
""")
    output = root / 'fault'
    built = subprocess.run(['dotnet', str(args.bridge.resolve()), '--project', str(root / 'Demo.rvnproj'), str(output)], capture_output=True, text=True, timeout=120)
    assert built.returncode == 0, built.stdout + built.stderr
    run = subprocess.run([str(args.runtime.resolve()), 'run', str(output / 'App.neoil'), '--system', str(args.system.resolve())], capture_output=True, text=True, timeout=30)
    assert run.returncode != 0 and 'MapResult mapper fault' in run.stderr, run.stdout + run.stderr
    print('Mapper faults remain terminal: Passed')

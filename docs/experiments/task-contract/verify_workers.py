"""Run the isolated-worker Task PoC with matching development artifacts."""
import argparse
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('project', type=Path)
parser.add_argument('--bridge', type=Path, required=True)
parser.add_argument('--system', type=Path, required=True)
parser.add_argument('--runtime', type=Path, required=True)
args = parser.parse_args()
parent = Path(__file__).resolve().parents[1]
samples = parent / 'samples' if (parent / 'samples').is_dir() else parent / 'raven-target/samples'
sample = samples / 'library-workers.rvn'
prelude = '''import System.*
import System.Tasks.*
import System.Concurrency.*
import System.Console.*
public func Echo(value: string) -> string => value
'''
cases = [
    ('Retained Thread lifecycle', prelude + '''public async func Work() -> Task<unit> {
        let thread = Thread(Echo, "tracked thread")
        let completion = thread.Task
        if thread.IsStarted || completion.IsCompleted {
            System.Fault("started too soon")
        }
        thread.Start()
        if !thread.IsStarted {
            System.Fault("missing start state")
        }
        WriteLine(await completion)
        WriteLine(await thread.Task)
        return ()
    }
    func Main() {
        let queue = TaskQueue()
        queue.Run(() => {
            _ = Work()
        })
    }''', 'tracked thread\ntracked thread\n', None),
    ('Construction queue retained across start scope', prelude + '''func Main() {
        let thread = Thread(Echo, "construction queue")
        let other = TaskQueue()
        other.Run(() => {
            thread.Start()
        })
        if thread.Task.IsCompleted {
            System.Fault("completion moved to start queue")
        }
        TaskQueue.Default.Drain()
        WriteLine(thread.Task.GetResult())
    }''', 'construction queue\n', None),
    ('Never-started thread stays pending', prelude + '''func Main() {
        let thread = Thread(Echo, "not started")
        TaskQueue.Default.Drain()
        if thread.IsStarted || thread.Task.IsCompleted {
            System.Fault("unstarted thread made progress")
        }
    }''', '', None),
    ('Starting twice rejected', prelude + '''func Main() {
        let thread = Thread(Echo, "input")
        thread.Start()
        thread.Start()
    }''', '', 'Thread has already been started'),
    ('Await dedicated and pooled tasks', sample.read_text(), 'Hello, thread\nHello, pool\n', None),
    ('Captured state rejected', prelude + '''func Main() {
        let queue = TaskQueue()
        let suffix = "!"
        queue.Run(() => { _ = Thread.Run(value => value + suffix, "input") })
    }''', '', 'cannot capture guest state'),
    ('Default queue without explicit scope', prelude + 'func Main() { _ = Thread.Run(Echo, "input") }', '', None),
    ('Multiple pool jobs', prelude + '''
public async func Work() -> Task<unit> {
    let first = ThreadPool.Queue(Echo, "one")
    let second = ThreadPool.Queue(Echo, "two")
    let third = ThreadPool.Queue(Echo, "three")
    WriteLine(await first)
    WriteLine(await second)
    WriteLine(await third)
    return ()
}
func Main() {
    let queue = TaskQueue()
    queue.Run(() => { _ = Work() })
}''', 'one\ntwo\nthree\n', None),
]
with tempfile.TemporaryDirectory(prefix='neoclr-workers-') as directory:
    root = Path(directory)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    for index, (label, source, expected, fault) in enumerate(cases):
        (root / 'Main.rvn').write_text(source)
        output = root / str(index)
        built = subprocess.run(['dotnet', str(args.bridge.resolve()), '--project', str(root / 'Demo.rvnproj'), str(output)], capture_output=True, text=True, timeout=120)
        assert built.returncode == 0, label + ': ' + built.stdout + built.stderr
        run = subprocess.run([str(args.runtime.resolve()), 'run', str(output / 'App.neoil'), '--system', str(args.system.resolve())], capture_output=True, text=True, timeout=60)
        assert (run.returncode != 0 and fault in run.stderr) if fault else (run.returncode == 0 and run.stdout == expected), label + ': ' + run.stdout + run.stderr
        print(label + ': Passed', flush=True)

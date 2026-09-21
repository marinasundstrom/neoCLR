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
sample = Path(__file__).resolve().parents[1] / 'raven-target/samples/library-workers.rvn'
prelude = '''import System.*
import System.Tasks.*
import System.Threading.*
import System.Console.*
public func Echo(value: string) -> string => value
'''
cases = [
    ('Await dedicated and pooled tasks', sample.read_text(), 'Hello, thread\nHello, pool\n', None),
    ('Captured state rejected', prelude + '''func Main() {
        let queue = TaskQueue()
        let suffix = "!"
        queue.Run(() => { _ = Thread.Start(value => value + suffix, "input") })
    }''', '', 'cannot capture guest state'),
    ('Scope required', prelude + 'func Main() { _ = Thread.Start(Echo, "input") }', '', 'requires an active TaskQueue'),
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

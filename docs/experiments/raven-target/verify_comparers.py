"""Focused comparer contracts, HashMap policy reentry, and source rejection checks."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
from runner_options import add_toolchain_arguments, runner_arguments

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('project', type=Path)
parser.add_argument('--runtime', required=True, type=Path)
add_toolchain_arguments(parser)
args = parser.parse_args()
tools = Path(__file__).resolve().parent
with tempfile.TemporaryDirectory(prefix='neoclr-comparers-') as temporary:
    root = Path(temporary)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    command = [sys.executable, str(tools / 'run_project.py'), str(root / 'Demo.rvnproj'),
               *runner_arguments(args), '--runtime', str(args.runtime.resolve())]
    results = {}

    def check(name, source, expected, success=True):
        (root / 'Main.rvn').write_text(source)
        result = subprocess.run(command, cwd=root, capture_output=True, text=True, timeout=180)
        if (result.returncode == 0) != success or expected not in result.stdout + result.stderr:
            raise AssertionError(name + ': ' + result.stdout + result.stderr)
        results[name] = 'passed'

    check('contracts', (tools / 'samples/library-comparers.rvn').read_text(), 'Comparer contract passed\n')
    check('legacy maps', (tools / 'samples/library-maps.rvn').read_text(), 'Shipped\n23\n2\n1\n')
    reentry = '''import System.*
import System.Collections.*
class State : EqualityComparer<int> {
    var Map: HashMap<int, int>
    var Reenter: bool = false
    init() {
        Map = HashMap<int, int>(self)
    }
    func Equals(left: int, right: int) -> bool {
        EQUALITY
        return left == right
    }
    func GetHashCode(value: int) -> int {
        HASH
        return 0
    }
}
func Main() {
    let state = State()
    state.Map.Set(1, 42)
    state.Reenter = true
    state.Map.ContainsKey(1)
}
'''
    for method in ('hash', 'equality'):
        source = reentry.replace('HASH', 'if Reenter { Map.ContainsKey(value) }' if method == 'hash' else '')
        source = source.replace('EQUALITY', 'if Reenter { Map.ContainsKey(left) }' if method == 'equality' else '')
        check(method + ' reentry', source, 'HashMap callbacks must not reenter the map', success=False)
    check('wrong key policy', '''import System.*
import System.Collections.*
func Main() { let map = HashMap<int, string>(StringComparer.Ordinal) }
''', 'RAV', success=False)
    check('ordering is not equality', '''import System.*
import System.Collections.*
func Main() {
    let ordering = DelegateComparer<int>((left, right) => left.CompareTo(right))
    let map = HashMap<int, int>(ordering)
}
''', 'RAV', success=False)
    print(json.dumps(results, indent=2))

"""Check Raven-authored list callback snapshots and short-circuit behavior end to end."""
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
header = '''import System.*
import System.Collections.*
import System.Option.*
import System.Console.*
'''
results = {}
with tempfile.TemporaryDirectory(prefix='neoclr-arraylist-library-') as temporary:
    root = Path(temporary)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    command = [sys.executable, str(Path(__file__).with_name('run_project.py')),
               str(root / 'Demo.rvnproj'), *runner_arguments(args),
               '--runtime', str(args.runtime.resolve())]
    # Every scan captures its extent once. Grow beyond capacity inside the callback;
    # the second visit must still read the captured old buffer, not the new buffer.
    for method, expected in [('Find', 'old\n2\n'), ('FindLast', 'first\n2\n'),
                             ('FindIndex', '1\n2\n'), ('FindLastIndex', '0\n2\n'),
                             ('Exists', 'matched\n2\n'), ('TrueForAll', 'stopped\n2\n'),
                             ('FindAll', 'old\n2\n')]:
        output = ('for value in found { WriteLine(value) }' if method == 'FindAll' else
                  'if found { WriteLine("matched") }' if method == 'Exists' else
                  'if !found { WriteLine("stopped") }' if method == 'TrueForAll' else
                  'match found { Some(let value) => WriteLine(value)\nNone => WriteLine("missing") }')
        condition = 'calls == 1' if method == 'TrueForAll' else 'calls == 2'
        source = header + f'''
func Main() {{
    let values = ArrayList<string>(2)
    values.Add("first")
    values.Add("old")
    var calls = 0
    let found = values.{method}((value: string) -> bool => {{
        calls = calls + 1
        if calls == 1 {{
            values.Add("appended")
            values[0] = "replaced"
            values[1] = "replaced"
        }}
        return {condition}
    }})
    {output}
    WriteLine(calls)
}}
'''
        (root / 'Main.rvn').write_text(source)
        result = subprocess.run(command, capture_output=True, text=True, timeout=120)
        assert result.returncode == 0 and result.stdout.endswith(expected), method + ': ' + result.stdout + result.stderr
        results[method] = 'captured buffer/extent and callback order preserved'
    (root / 'Main.rvn').write_text(header + '''
func Main() {
    let values = ArrayList<string>(4)
    values.Add("first")
    values.Add("second")
    let found = values.FindAll((value: string) -> bool => {
        values[1] = "edited in place"
        return true
    })
    for value in found {
        WriteLine(value)
    }
}
''')
    result = subprocess.run(command, capture_output=True, text=True, timeout=120)
    assert result.returncode == 0 and result.stdout.endswith('first\nedited in place\n'), result.stdout + result.stderr
    results['In-place changes'] = 'later reads observe edits within the captured buffer'
print(json.dumps(results, indent=2))

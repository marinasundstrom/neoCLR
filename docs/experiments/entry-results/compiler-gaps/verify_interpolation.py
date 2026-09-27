"""Run the release interpolation/Concat contract with the selected bridge and library."""
import argparse
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
for name in ('runtime', 'bridge', 'system', 'reference'):
    parser.add_argument('--' + name, type=Path, required=True)
args = parser.parse_args()
tools = {name: getattr(args, name).resolve() for name in ('runtime', 'bridge', 'system', 'reference')}
here = Path(__file__).resolve().parent
cases = [
    ('async-interpolation', (here / 'AsyncInterpolation.rvn').read_text(), 'Value 1\n'),
    ('object-concat', '''import System.*
class Label {
    override func ToString() -> string { return "custom" }
}
func Main() {
    let missing: Object? = null
    Console.WriteLine(String.Concat(missing, missing))
    Console.WriteLine(String.Concat(missing, Label()))
    Console.WriteLine(String.Concat(Label(), missing))
    Console.WriteLine(String.Concat(12, true))
    Console.WriteLine("value ${Label()} ${42}")
    Console.WriteLine(String.Concat("left", "right"))
}
''', '\ncustom\ncustom\n12True\nvalue custom 42\nleftright\n'),
]
with tempfile.TemporaryDirectory(prefix='neoclr-interpolation-') as directory:
    root = Path(directory)
    shutil.copyfile(here.parent / 'Contracts.rvnproj', root / 'Contracts.rvnproj')
    shutil.copyfile(tools['reference'], root / 'NeoCLR.CoreProbe.dll')
    for name, source, expected in cases:
        (root / 'Main.rvn').write_text(source, encoding='utf-8')
        output = root / name
        commands = [
            ['dotnet', str(tools['bridge']), '--project', str(root / 'Contracts.rvnproj'), str(output)],
            [str(tools['runtime']), 'verify', str(output / 'App.neoil'), '--system', str(tools['system'])],
            [str(tools['runtime']), 'run', str(output / 'App.neoil'), '--system', str(tools['system'])],
        ]
        for command in commands:
            result = subprocess.run(command, capture_output=True, text=True, timeout=180)
            assert result.returncode == 0, (name, command, result.stdout, result.stderr)
        assert result.stdout == expected and result.stderr == '', (name, result.stdout, result.stderr)
        print(name + ': passed', flush=True)

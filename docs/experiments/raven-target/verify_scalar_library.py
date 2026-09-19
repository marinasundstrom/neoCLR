"""Execute Raven-authored scalar APIs through a separately compiled CLI consumer."""
import argparse
from pathlib import Path
import subprocess
import tempfile
from xml.sax.saxutils import escape
from build_runtime_library import ROOT, check_snapshot
from collection_library import build

parser = argparse.ArgumentParser(description=__doc__)
for name in ('compiler', 'bridge', 'runtime'):
    parser.add_argument('--' + name, required=True, type=Path)
args = parser.parse_args()
compiler, bridge, runtime = (getattr(args, n).resolve() for n in ('compiler', 'bridge', 'runtime'))


def run(command):
    result = subprocess.run([str(x) for x in command], capture_output=True, text=True, timeout=120)
    assert result.returncode == 0, result.stdout + result.stderr
    return result.stdout


check_snapshot()
with tempfile.TemporaryDirectory(prefix='neoclr-scalar-library-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    core = root / 'demo/NeoCLR.CoreProbe.dll'
    run(['dotnet', bridge, '--reference-core', core])
    statements, expected = [], []
    for left, right in [(7, 2), (-7, 2), (7, -2), (0, 2), (-2147483648, 1),
                        (-2147483648, -1), (1, 0), (0, 0), (2147483647, -1)]:
        statements.append(f'ShowDivision(Int32.Divide({left}, {right}))')
        expected.append('zero' if right == 0 else 'overflow' if (left, right) == (-2147483648, -1)
                        else str((abs(left) // abs(right)) * (-1 if (left < 0) != (right < 0) else 1)))
    for code in [0, 8, 9, 13, 14, 32, 47, 48, 57, 58, 65, 127, 128, 133, 160,
                 233, 1639, 0x200b, 0x2028, 0xd7ff, 0xe000, 0xffff, 0x10400, 0x1d7ce, 0x1f600, 0x10ffff]:
        predicates = {
            'IsAscii': code <= 127,
            'IsAsciiDigit': 48 <= code <= 57,
            'IsLetterOrDigit': code in [48, 57, 65, 233, 1639, 0x10400, 0x1d7ce],
            'IsWhiteSpace': code in [9, 13, 32, 133, 160, 0x2028],
        }
        for name, value in predicates.items():
            statements.append(f'Show(Char.{name}((char){code}))')
            expected.append('true' if value else 'false')
    source = '''import System.*
import System.Result.*
import System.Console.*
func Show(value: bool) {
    if value { WriteLine("true") } else { WriteLine("false") }
}
func ShowDivision(value: Result<int, IntegerDivisionError>) {
    match value {
        Ok(let result) => WriteLine(result)
        Error(let error) => {
            if error.IsDivisionByZero { WriteLine("zero") } else { WriteLine("overflow") }
        }
    }
}
class Flag {
    private var enabled: bool
    public init(value: bool) { enabled = value }
    public func Read(other: bool) -> bool { return other || enabled }
}
func Identity(value: bool) -> bool { return value }
func Local(value: bool, other: bool) -> bool {
    let saved = value
    return other || saved
}
func Call(value: bool, other: bool) -> bool { return other || Identity(value) }
func Main() {
''' + '\n'.join('    ' + s for s in statements)
    for left in ('false', 'true'):
        for right in ('false', 'true'):
            source += f'\n    Show(Flag({left}).Read({right}))\n    Show(Local({left}, {right}))\n    Show(Call({left}, {right}))'
            expected.extend(['true' if 'true' in (left, right) else 'false'] * 3)
    source += '\n}\n'
    (root / 'Main.rvn').write_text(source)
    project = root / 'Demo.rvnproj'
    project.write_text(f'''<Project><PropertyGroup><OutputType>Exe</OutputType>
<AssemblyName>Consumer</AssemblyName><NeoCLRRoot>{escape(str(root))}</NeoCLRRoot></PropertyGroup>
<Import Project="{escape(str(ROOT / 'build/NeoCLR.Raven.props'))}" />
<ItemGroup><Compile Include="Main.rvn" /></ItemGroup></Project>''')
    run(['dotnet', compiler, project, '--no-project-restore', '-o', root / 'compiled'])
    imported = root / 'imported'
    run(['dotnet', bridge, '--import', root / 'compiled/Consumer.dll', core, imported])
    system = root / 'System.neoil'
    system.write_text(build(ROOT / 'runtime/System.neoil'))
    run([runtime, 'verify', imported / 'App.neoil', '--system', system])
    actual = run([runtime, 'run', imported / 'App.neoil', '--system', system]).splitlines()
    assert actual == expected, (actual, expected)
    print(f'{len(expected)} scalar and Boolean merge outcomes passed')

    # Invalid scalar values must fail even when the cast is immediately widened.
    for code in (0xd800, 0xdfff, 0x110000, -1):
        (root / 'Main.rvn').write_text(f"""import System.Console.*
func Invalid(value: int) -> int {{
    return (int)(char)value
}}
func Main() {{
    WriteLine(Invalid({code}))
}}
""")
        run(['dotnet', compiler, project, '--no-project-restore', '-o', root / 'compiled'])
        rejected = root / ('invalid-' + str(code))
        run(['dotnet', bridge, '--import', root / 'compiled/Consumer.dll', core, rejected])
        failure = subprocess.run([str(runtime), 'run', str(rejected / 'App.neoil'), '--system', str(system)], capture_output=True, text=True, timeout=30)
        assert failure.returncode != 0, (code, failure.stdout, failure.stderr)
        assert 'Char requires a Unicode scalar value' in failure.stderr, (code, failure.stderr)
    print('4 invalid scalar conversion outcomes passed')

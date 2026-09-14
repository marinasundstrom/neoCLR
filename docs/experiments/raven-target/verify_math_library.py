"""Validate the Raven Math bootstrap with a separate consumer and rejected library contracts."""
import argparse
import json
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

def run(command, success=True, diagnostic=None):
    result = subprocess.run([str(x) for x in command], capture_output=True, text=True, timeout=120)
    assert (result.returncode == 0) == success, result.stdout + result.stderr
    if diagnostic:
        assert diagnostic in result.stdout + result.stderr, result.stdout + result.stderr
    return result

check_snapshot()
with tempfile.TemporaryDirectory(prefix='neoclr-math-validation-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    core = root / 'demo/NeoCLR.CoreProbe.dll'
    run(['dotnet', bridge, '--reference-core', core])

    def compile(name, source, library=False):
        folder = root / name
        folder.mkdir()
        (folder / 'Main.rvn').write_text(source)
        project = folder / 'Demo.rvnproj'
        project.write_text(f'''<Project>
  <PropertyGroup>
    <OutputType>{'Library' if library else 'Exe'}</OutputType>
    <AssemblyName>{name}</AssemblyName>
    <NeoCLRRoot>{escape(str(root))}</NeoCLRRoot>
  </PropertyGroup>
  <Import Project="{escape(str(ROOT / 'build/NeoCLR.Raven.props'))}" />
  <ItemGroup><Compile Include="Main.rvn" /></ItemGroup>
</Project>''')
        run(['dotnet', compiler, project, '--no-project-restore', '-o', folder / 'output'])
        return folder / 'output' / (name + '.dll')

    statements, expected = [], []
    values = [-2147483648, -7, 0, 7, 2147483647]
    for left in values:
        for right in values:
            statements.extend([f'WriteLine(System.Math.Min(left: {left}, right: {right}))', f'WriteLine(Max({left}, {right}))'])
            expected.extend([str(min(left, right)), str(max(left, right))])
    for value in values:
        statements += [f'WriteLine(Sign({value}))', f'ShowAbs(Abs({value}))']
        expected += [str((value > 0) - (value < 0)), 'overflow' if value == -2147483648 else str(abs(value))]
    for value, low, high in [(5, 0, 10), (-1, 0, 10), (11, 0, 10), (99, 7, 7),
                             (-2147483648, -2147483648, 2147483647), (2147483647, -2147483648, 2147483647), (5, 10, 0)]:
        statements.append(f'ShowClamp(Clamp({value}, {low}, {high}))')
        expected.append('invalid' if low > high else str(min(max(value, low), high)))
    statements += ['WriteLine(System.Math.PilotDomain.Answer())', 'WriteLine(System.Math.Sqrt(25.0).CompareTo(5.0))']
    expected += ['42', '0']
    source = '''import System.*
import System.Math.*
import System.Result.*
import System.Console.*
namespace System.Math {
    public class PilotDomain {
        public static func Answer() -> int { return 42 }
    }
}
func ShowAbs(value: Result<int, OverflowError>) {
    match value {
        Ok(let result) => WriteLine(result)
        Error(_) => WriteLine("overflow")
    }
}
func ShowClamp(value: Result<int, InvalidRangeError>) {
    match value {
        Ok(let result) => WriteLine(result)
        Error(_) => WriteLine("invalid")
    }
}
func Main() {
''' + '\n'.join('    ' + s for s in statements) + '\n}\n'
    consumer = compile('Consumer', source)
    imported = root / 'imported'
    run(['dotnet', bridge, '--import', consumer, core, imported])
    system = root / 'System.neoil'
    system.write_text(build(ROOT / 'runtime/System.neoil'))
    run([runtime, 'verify', imported / 'App.neoil', '--system', system])
    result = run([runtime, 'run', imported / 'App.neoil', '--system', system])
    assert result.stdout.splitlines() == expected, result.stdout

    for name, code, diagnostic in [
        ('Unknown', 'public func Missing() -> int { return 0 }', 'does not match reference contract'),
        ('WrongResult', 'public func Min(left: int, right: int) -> bool { return true }', 'does not match reference contract'),
        ('WrongParameters', 'public func Min(value: int) -> int { return value }', 'does not match reference contract'),
        ('WrongNames', 'public func Min(a: int, b: int) -> int { return a }', 'does not match reference contract'),
        ('Generic', 'public func Min<T>(left: T, right: T) -> T { return left }', 'Unsupported application signature'),
        ('Unmarked', 'public static class NamespaceMembers { public static func Min(left: int, right: int) -> int { return left } }', 'Missing namespace implementation'),
    ]:
        image = compile(name, 'import System.*\nnamespace System.Math\n' + code, library=True)
        output = root / (name + '-import')
        run(['dotnet', bridge, '--library-implementation', image, core, 'System.Math', output], False, diagnostic)
        assert not (output / 'Implementation.neoil').exists()
    print(json.dumps({'consumerResults': len(expected), 'rejectedContracts': 6,
                      'qualifiedCalls': True, 'wildcardImports': True, 'relatedNamespaceType': True,
                      'resultPayloadsAndErrors': True, 'extractedSystemWithoutSourceIncludes': True}, indent=2))

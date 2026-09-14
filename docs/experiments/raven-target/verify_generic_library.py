"""Prove a bounded generic namespace implementation against a separate reference contract."""
import argparse
from pathlib import Path
import subprocess
import tempfile
from xml.sax.saxutils import escape
from build_runtime_library import ROOT
from collection_library import build

parser = argparse.ArgumentParser(description=__doc__)
for name in ('compiler', 'bridge', 'runtime'):
    parser.add_argument('--' + name, required=True, type=Path)
args = parser.parse_args()
compiler, bridge, runtime = (getattr(args, n).resolve() for n in ('compiler', 'bridge', 'runtime'))

def run(command, success=True):
    result = subprocess.run([str(x) for x in command], capture_output=True, text=True, timeout=120)
    assert (result.returncode == 0) == success, result.stdout + result.stderr
    return result.stdout + result.stderr

with tempfile.TemporaryDirectory(prefix='neoclr-generic-library-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    core = root / 'demo/NeoCLR.CoreProbe.dll'
    run(['dotnet', bridge, '--generic-library-core', core])
    source = '''import System.Collections.*
namespace Probe.Generic

public func Choose<T>(first: bool, left: T, right: T) -> T {
    let selected = if first {
        left
    } else {
        right
    }
    return selected
}

public func ChooseInt(first: bool, left: int, right: int) -> int {
    return Choose<int>(first, left, right)
}

public func CountItems<T>(source: Iterable<T>) -> int {
    var count = 0
    for item in source {
        count += 1
    }
    return count
}

public func UseCollection() -> int {
    let values = ArrayList<string>()
    values.Add("first")
    values.Add("second")
    return CountItems<string>(values)
}

public func UseUnit() -> int {
    let selected = Choose<()>(false, (), ())
    Choose<()>(true, selected, ())
    return 42
}

public func ChooseText(first: bool, left: string, right: string) -> string {
    return Choose<string>(first, left, right)
}
'''
    def compile(name, text):
        folder = root / name
        folder.mkdir()
        (folder / 'Main.rvn').write_text(text)
        project = folder / 'Library.rvnproj'
        project.write_text(f'''<Project><PropertyGroup><OutputType>Library</OutputType>
<AssemblyName>{name}</AssemblyName><NeoCLRRoot>{escape(str(root))}</NeoCLRRoot></PropertyGroup>
<Import Project="{escape(str(ROOT / 'build/NeoCLR.Raven.props'))}" />
<ItemGroup><Compile Include="Main.rvn" /></ItemGroup></Project>''')
        run(['dotnet', compiler, project, '--no-project-restore', '-o', folder / 'compiled'])
        return folder / 'compiled' / (name + '.dll')
    image = compile('GenericLibrary', source)
    imported = root / 'imported'
    run(['dotnet', bridge, '--library-implementation', image, core, 'Probe.Generic', imported])
    body = (imported / 'Implementation.neoil').read_text()
    assert '.function Probe.Generic.Choose<T0>(' in body, body
    driver = '.module GenericLibrary\n.entry Main\n' + body + '\n.function Main() -> void\n'
    for first in (True, False):
        driver += f'ldc.bool {str(first).lower()}\nldc.i4 11\nldc.i4 22\ncall Probe.Generic.ChooseInt(Boolean,Int32,Int32)\ncall System.Console::WriteLine(Int32)\npop\n'
        driver += f'ldc.bool {str(first).lower()}\nldstr "left"\nldstr "right"\ncall Probe.Generic.ChooseText(Boolean,String,String)\ncall System.Console::WriteLine(String)\npop\n'
    driver += 'call Probe.Generic.UseCollection()\ncall System.Console::WriteLine(Int32)\npop\ncall Probe.Generic.UseUnit()\ncall System.Console::WriteLine(Int32)\npop\nret\n.end\n'
    application = root / 'App.neoil'
    application.write_text(driver)
    system = root / 'System.neoil'
    system.write_text(build(ROOT / 'runtime/System.neoil'))
    run([runtime, 'verify', application, '--system', system])
    assert run([runtime, 'run', application, '--system', system]).splitlines() == ['11', 'left', '22', 'right', '2', '42']
    for name, invalid in [('WrongName', source.replace('left: T', 'value: T').replace('        left', '        value')),
                          ('WrongShape', source.replace('source: Iterable<T>', 'source: List<T>')),
                          ('WrongPosition', source.replace('Choose<T>', 'Choose<T, U>').replace('Choose<int>(', 'Choose<int, int>(').replace('Choose<string>(', 'Choose<string, string>(').replace('Choose<()>(', 'Choose<(), ()>('))]:
        bad = compile(name, invalid)
        output = root / name / 'imported'
        diagnostic = run(['dotnet', bridge, '--library-implementation', bad, core, 'Probe.Generic', output], False)
        assert 'does not match reference contract' in diagnostic, diagnostic
        assert not (output / 'Implementation.neoil').exists()
    print('Generic namespace body: Int32/String/Void, generic Iterable iteration, consumed/discarded unit results, and three invalid contracts passed')

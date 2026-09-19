"""Validate Raven ParameterInfo authoring and reject incompatible snapshot layouts."""
import argparse
from pathlib import Path
import subprocess
import tempfile
from xml.sax.saxutils import escape
from build_runtime_library import ROOT

parser = argparse.ArgumentParser(description=__doc__)
for name in ('compiler', 'bridge'):
    parser.add_argument('--' + name, required=True, type=Path)
args = parser.parse_args()


def run(command, success=True, diagnostic=None):
    result = subprocess.run([str(x) for x in command], capture_output=True,
                            text=True, timeout=120)
    assert (result.returncode == 0) == success, result.stdout + result.stderr
    if diagnostic:
        assert diagnostic in result.stdout + result.stderr, result.stdout + result.stderr
    return result


with tempfile.TemporaryDirectory(prefix='neoclr-parameter-info-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    core = root / 'demo/NeoCLR.CoreProbe.dll'
    run(['dotnet', args.bridge.resolve(), '--reference-library-core', core])
    source = (ROOT / 'runtime/raven/src/System/Introspection/ParameterInfo.rvn').read_text()
    for name, text, diagnostic in [
        ('Valid', source, None),
        ('Extra', source.replace('private field StoredName:', 'private field Extra: int = 0\n    private field StoredName:'), 'ParameterInfo library layout'),
        ('Reordered', source.replace('private field StoredIsOut: bool\n    private field StoredIsOutWhenTrue: bool', 'private field StoredIsOutWhenTrue: bool\n    private field StoredIsOut: bool'), 'ParameterInfo library layout'),
        ('Constructor', source.replace('private init', 'public init'), 'does not match reference contract'),
        ('Missing', source.replace('val Position:', 'val MissingPosition:'), 'does not match reference contract'),
    ]:
        folder = root / name
        folder.mkdir()
        (folder / 'Main.rvn').write_text(text)
        project = folder / 'Probe.rvnproj'
        project.write_text(f'''<Project>
  <PropertyGroup><OutputType>Library</OutputType><AssemblyName>{name}</AssemblyName><NeoCLRRoot>{escape(str(root))}</NeoCLRRoot></PropertyGroup>
  <Import Project="{escape(str(ROOT / 'build/NeoCLR.Raven.props'))}" />
  <ItemGroup><Compile Include="Main.rvn" /></ItemGroup>
</Project>''')
        run(['dotnet', args.compiler.resolve(), project, '--no-project-restore', '-o', folder / 'bin'])
        output = folder / 'imported'
        run(['dotnet', args.bridge.resolve(), '--library-implementation', folder / 'bin' / (name + '.dll'),
             core, 'System.Introspection.ParameterInfo', output], diagnostic is None, diagnostic)
        if diagnostic:
            assert not (output / 'Implementation.neoil').exists()
        else:
            result = (output / 'Implementation.neoil').read_text()
            assert '.interface System.Introspection.ParameterInfo' in result
            assert '.type internal class System.Introspection.RuntimeParameterInfo' in result
            assert '.method private instance .ctor(' in result
print('ParameterInfo imports; extra/reordered storage, public construction and missing exports rejected.')

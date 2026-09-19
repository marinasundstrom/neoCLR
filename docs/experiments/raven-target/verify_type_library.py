"""Check TypeInfo authoring exports, constructor visibility and native descriptor layout."""
import argparse
import re
from pathlib import Path
import subprocess
import tempfile
from xml.sax.saxutils import escape
from build_runtime_library import ROOT

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--type-info', action='store_true', help=argparse.SUPPRESS)  # Legacy invocation alias.
for name in ('compiler', 'bridge'):
    parser.add_argument('--' + name, required=True, type=Path)
args = parser.parse_args()


def run(command, success=True, diagnostic=None):
    result = subprocess.run([str(x) for x in command], capture_output=True, text=True, timeout=120)
    assert (result.returncode == 0) == success, result.stdout + result.stderr
    if diagnostic:
        assert diagnostic in result.stdout + result.stderr, result.stdout + result.stderr
    return result


with tempfile.TemporaryDirectory(prefix='neoclr-declaration-library-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    core = root / 'demo/NeoCLR.CoreProbe.dll'
    run(['dotnet', args.bridge.resolve(), '--reference-library-core', core])
    owner = 'System.Introspection.TypeInfo'
    source_path = 'Introspection/Descriptors.rvn'
    source = (ROOT / 'runtime/raven/src/System' / source_path).read_text()
    prefix, source = source.split('public sealed interface TypeInfo', 1)
    source = 'public sealed interface TypeInfo' + source
    for name, text, diagnostic in [
        ('Valid', source, None),
        ('Storage', source.replace('private field Handle: RuntimeTypeHandle', 'private field Handle: RuntimeTypeHandle\n    private field Extra: int = 0'), 'Type library layout'),
        ('Constructor', source.replace('private init', 'public init'), 'does not match reference contract'),
        ('Missing', source.replace('val BaseType:', 'val MissingBaseType:'), 'does not match reference contract'),
        ('Argument', source.replace('GetMethods(flags:', 'GetMethods(wrongName:').replace('TypeMethods(Handle, (int)flags)', 'TypeMethods(Handle, (int)wrongName)'), 'does not match reference contract'),
    ] + [('ProviderVisibility', source.replace('internal class RuntimeTypeInfo', 'public class RuntimeTypeInfo'), 'Runtime descriptor providers must remain internal'),
          ('FactoryVisibility', source.replace('internal static func FromHandle', 'private static func FromHandle'),
           'does not match reference contract')]:
        folder = root / name
        folder.mkdir()
        (folder / 'Main.rvn').write_text(prefix + text)
        project = folder / 'Probe.rvnproj'
        project.write_text(f'''<Project>
  <PropertyGroup><OutputType>Library</OutputType><AssemblyName>{name}</AssemblyName><NeoCLRRoot>{escape(str(root))}</NeoCLRRoot></PropertyGroup>
  <Import Project="{escape(str(ROOT / 'build/NeoCLR.Raven.props'))}" />
  <PropertyGroup><RavenTypeOfAssemblyName/><RavenTypeOfInfoType/><RavenTypeOfContextType/></PropertyGroup>
  <ItemGroup><Compile Include="Main.rvn" /></ItemGroup>
</Project>''')
        run(['dotnet', args.compiler.resolve(), project, '--no-project-restore', '-o', folder / 'bin'])
        output = folder / 'imported'
        run(['dotnet', args.bridge.resolve(), '--library-implementation', folder / 'bin' / (name + '.dll'), core, 'System.Introspection.MemberInfo', output], diagnostic is None, diagnostic)
        if diagnostic:
            assert not (output / 'Implementation.neoil').exists()
        else:
            result = (output / 'Implementation.neoil').read_text()
            assert '.interface ' + owner in result
            assert '.method private instance .ctor(System.RuntimeTypeHandle' in result
            assert '.type internal class System.Introspection.RuntimeTypeInfo' in result
            assert '.method internal static FromHandle(' in result
    print('TypeInfo imports; added storage, public construction, missing exports and changed parameter names rejected.')

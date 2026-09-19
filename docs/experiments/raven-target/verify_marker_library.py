"""Check empty Object and union-attribute declaration admission."""
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


def run(command, diagnostic=None):
    result = subprocess.run([str(x) for x in command], capture_output=True, text=True, timeout=120)
    output = result.stdout + result.stderr
    assert (result.returncode == 0) == (diagnostic is None), output
    if diagnostic:
        assert diagnostic in output, (command, output)
    return result


with tempfile.TemporaryDirectory(prefix='neoclr-flags-library-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    core = root / 'demo/NeoCLR.CoreProbe.dll'
    run(['dotnet', args.bridge.resolve(), '--reference-library-core', core])
    object_source = (ROOT / 'runtime/raven/src/System/Object.rvn').read_text()
    marker_source = (ROOT / 'runtime/raven/src/System/Runtime/CompilerServices/UnionAttribute.rvn').read_text()
    bad = 'Unsupported empty marker declaration'
    cases = [
        ('Object', 'System.Object', object_source, None),
        ('UnionAttribute', 'System.Runtime.CompilerServices.UnionAttribute', marker_source, None),
        ('ObjectStorage', 'System.Object', object_source.replace('class Object {', 'class Object { private field value: int'), bad),
        ('ObjectMethod', 'System.Object', object_source.replace('class Object {', 'class Object { func Added() -> int { return 42 }'), bad),
        ('SealedObject', 'System.Object', object_source.replace('open class', 'class'), bad),
        ('MarkerStorage', 'System.Runtime.CompilerServices.UnionAttribute', marker_source.replace('init()', 'private field value: int\n    init()'), bad),
        ('MarkerBody', 'System.Runtime.CompilerServices.UnionAttribute', marker_source.replace('init() {', 'init() { System.Fault("side effect")'), bad),
        ('MarkerBase', 'System.Runtime.CompilerServices.UnionAttribute', marker_source.replace(' : System.Attribute', ''), bad),
        ('OpenMarker', 'System.Runtime.CompilerServices.UnionAttribute', marker_source.replace('public class', 'public open class'), bad),
    ]
    for name, owner, text, diagnostic in cases:
        folder = root / name
        folder.mkdir()
        (folder / 'Main.rvn').write_text(text)
        project = folder / 'Probe.rvnproj'
        project.write_text(f'''<Project>
  <PropertyGroup><OutputType>Library</OutputType><AssemblyName>{name}</AssemblyName><NeoCLRRoot>{escape(str(root))}</NeoCLRRoot></PropertyGroup>
  <Import Project="{escape(str(ROOT / 'build/NeoCLR.Raven.props'))}" />
  <ItemGroup><Compile Include="Main.rvn" /></ItemGroup>
</Project>''')
        compile_diagnostic = None
        run(['dotnet', args.compiler.resolve(), project, '--no-project-restore', '-o', folder / 'bin'], compile_diagnostic)
        if compile_diagnostic:
            continue
        output = folder / 'imported'
        run(['dotnet', args.bridge.resolve(), '--library-implementation', folder / 'bin' / (name + '.dll'),
             core, owner, output], diagnostic)
        if diagnostic:
            assert not (output / 'Implementation.neoil').exists()
    print(f'{len(cases)} empty marker admission cases passed')

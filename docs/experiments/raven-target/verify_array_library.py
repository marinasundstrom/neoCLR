"""Check intrinsic managed-array authoring boundaries."""
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
    source = (ROOT / 'runtime/raven/src/System/Array.rvn').read_text()
    cases = [
        ('Array', source, None),
        ('WrongStorage', source.replace('m_value', 'other'), 'Unsupported intrinsic array storage'),
        ('ExtraStorage', source.replace('private field m_value:', 'private field extra: int\n    private field m_value:'), 'Unsupported intrinsic array storage'),
        ('PublicConstructor', source.replace('private init()', 'public init()'), 'Unsupported intrinsic array storage'),
        ('ConstructorEffect', source.replace('private init() {', 'private init() { Fault("effect")'), 'Unsupported intrinsic array storage'),
        ('MutateStorage', source.replace('let buffer = m_value', 'm_value = Runtime.CompilerServices.CheckedStorage.Reserve<T>(0)\n        let buffer = m_value'), 'backing storage is readonly'),
        ('AllocateIntrinsic', source.replace('let buffer = m_value', 'let allocated = Array<T>()\n        let buffer = m_value'), 'Managed arrays require intrinsic allocation'),
        ('WrongParameterName', source.replace('action', 'callback'), 'export does not match reference contract'),
    ]
    for name, text, diagnostic in cases:
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
             core, 'System.Array', output], diagnostic)
        if diagnostic:
            assert not (output / 'Implementation.neoil').exists()
    print(f'{len(cases)} array authoring admission cases passed')

"""Check native allocation source contract admission."""
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


with tempfile.TemporaryDirectory(prefix='neoclr-native-library-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    core = root / 'demo/NeoCLR.CoreProbe.dll'
    run(['dotnet', args.bridge.resolve(), '--reference-library-core', core])
    source = (ROOT / 'runtime/raven/src/System/Runtime/InteropServices/NativeMemory/Functions.rvn').read_text()
    cases = [
        ('NativeMemory', source, None),
        ('WrongParameterName', source.replace('byteCount', 'bytes'), 'export does not match'),
        ('WrongParameterType', source.replace('Alloc(byteCount: nuint)', 'Alloc(byteCount: int)').replace('NativeAllocation.Allocate(byteCount)', 'NativeAllocation.Allocate(default(nuint))').replace('return Alloc(byteCount)', 'return Alloc(0)'), 'export does not match'),
        ('MissingFree', source[:source.index('public unsafe func Free')], 'complete overload surface'),
        ('ExtraExport', source + '\npublic func Extra() -> int { return 0 }\n', 'complete overload surface'),
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
             core, 'System.Runtime.InteropServices.NativeMemory', output], diagnostic)
        if diagnostic:
            assert not (output / 'Implementation.neoil').exists()
    print(f'{len(cases)} native allocation admission cases passed')

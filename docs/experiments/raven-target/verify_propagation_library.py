"""Validate the exact conditional-output propagation declaration."""
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
        assert diagnostic in output, output
    return result


with tempfile.TemporaryDirectory(prefix='neoclr-foundation-library-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    core = root / 'demo/NeoCLR.CoreProbe.dll'
    run(['dotnet', args.bridge.resolve(), '--reference-library-core', core])
    source = (ROOT / 'runtime/raven/src/System/Propagatable.rvn').read_text()
    cases = [
        ('Protocol', 'Propagatable', source, None),
        ('RefParameter', 'Propagatable', source.replace('out output:', 'ref output:'), 'Unsupported library interface contract'),
        ('WrongPayload', 'Propagatable', source.replace('output: TOutput', 'output: TResidual'), 'Unsupported library interface contract'),
        ('MissingOutput', 'Propagatable', source.replace('out output:', 'output:'), 'Library interface does not match reference contract'),
        ('WrongName', 'Propagatable', source.replace('TryGetOutput', 'Extract'), 'Unsupported library interface contract'),
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
        run(['dotnet', args.compiler.resolve(), project, '--no-project-restore', '-o', folder / 'bin'])
        output = folder / 'imported'
        run(['dotnet', args.bridge.resolve(), '--library-implementation', folder / 'bin' / (name + '.dll'),
             core, 'System.' + owner, output], diagnostic)
        if diagnostic:
            assert not (output / 'Implementation.neoil').exists()
        else:
            emitted = (output / 'Implementation.neoil').read_text()
            assert '.interface System.Propagatable<T0,T1,T2>' in emitted
            assert 'readonly byref TryGetOutput(out(true) T1& output)' in emitted
            assert 'readonly byref TryGetResidual(out(true) T2& residual)' in emitted
    print(f'{len(cases)} propagation declaration admission cases passed')

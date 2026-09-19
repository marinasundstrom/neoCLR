"""Check the complete invariant Func declaration family."""
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


with tempfile.TemporaryDirectory(prefix='neoclr-delegate-library-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    core = root / 'demo/NeoCLR.CoreProbe.dll'
    run(['dotnet', args.bridge.resolve(), '--reference-library-core', core])
    source = (ROOT / 'runtime/raven/src/System/Func.rvn').read_text()
    cases = [
        ('Func', source, None),
        ('MissingArity', '\n'.join(line for line in source.splitlines() if 'T4' not in line), 'Missing Func arity'),
        ('WrongReturn', source.replace('Func<TResult>() -> TResult', 'Func<TResult>() -> bool'), 'invocation contract'),
        ('ByrefArgument', source.replace('arg1: T1', 'ref arg1: T1', 1), 'invocation contract'),
        ('NonDelegate', source.replace('public delegate Func<TResult>() -> TResult', 'public class Func<TResult> { }'), 'Unsupported Func declaration'),
        ('ExtraArity', source + '\npublic delegate Func<T1,T2,T3,T4,T5,TResult>(arg1: T1,arg2: T2,arg3: T3,arg4: T4,arg5: T5) -> TResult\n', 'Unsupported Func arity'),
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
             core, 'System.Func', output], diagnostic)
        if diagnostic:
            assert not (output / 'Implementation.neoil').exists()
    print(f'{len(cases)} delegate declaration admission cases passed')

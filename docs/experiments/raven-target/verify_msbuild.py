"""Exercise standalone MSBuild assets with an installed Raven SDK and neoCLR bundle."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
from xml.sax.saxutils import escape

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bundle', required=True, type=Path)
parser.add_argument('--sdk', required=True, type=Path)
parser.add_argument('--assets', type=Path, help='Build asset directory; defaults to bundle/build')
args = parser.parse_args()
bundle, sdk = args.bundle.resolve(), args.sdk.resolve()
assets = (args.assets or bundle / 'build').resolve()
checks = {}
with tempfile.TemporaryDirectory(prefix='neoCLR MSBuild ') as directory:
    root = Path(directory)
    shutil.copytree(assets, root / 'build')
    project = root / 'Demo.rvnproj'
    project_text = f'''<Project DefaultTargets="Build">
  <PropertyGroup><NeoCLRRoot>{escape(str(bundle))}</NeoCLRRoot></PropertyGroup>
  <Import Project="build/NeoCLR.Raven.props" />
  <ItemGroup><Compile Include="Main.rvn" /></ItemGroup>
  <Import Project="build/NeoCLR.Raven.targets" />
</Project>'''
    project.write_text(project_text)
    source = root / 'Main.rvn'
    success = 'import System.Console.*\nfunc Main() { WriteLine("BUILD DOES NOT RUN ME") }\n'
    source.write_text(success)

    def build(name, *, configuration='Debug', expected=True, error=None, extra=()):
        completed = subprocess.run(['dotnet', 'msbuild', str(project), '-v:minimal',
            '-p:RavenSdkRoot=' + str(sdk), '-p:Configuration=' + configuration, *extra],
            capture_output=True, text=True, timeout=120)
        log = completed.stdout + completed.stderr
        assert (completed.returncode == 0) == expected, name + '\n' + log
        assert 'BUILD DOES NOT RUN ME' not in log, log
        if error:
            assert error in log, name + '\n' + log
        artifact = root / 'bin/neoclr' / configuration / 'App.neoil'
        if not expected:
            assert not artifact.exists(), name + ': stale runnable output survived'
        checks[name] = 'passed'
        return artifact

    def run(artifact, expected):
        result = subprocess.run([str(bundle / 'bin/neoclr'), 'run', str(artifact),
            '--system', str(artifact.parent / 'System.neoil')],
            capture_output=True, text=True, timeout=60)
        assert result.returncode == 0 and result.stdout == expected, result.stdout + result.stderr
        assert Path(str(artifact) + '.map.json').exists()

    # Project evaluation must not resolve the .NET SDK or run the build targets.
    evaluation = subprocess.run(['dotnet', 'msbuild', str(project),
        '-getProperty:RavenUseHostFrameworkReferences,RavenTargetCoreAssemblyName,TargetFramework',
        '-getItem:Reference,Compile'], capture_output=True, text=True, timeout=60)
    assert evaluation.returncode == 0, evaluation.stdout + evaluation.stderr
    evaluated = json.loads(evaluation.stdout)
    assert evaluated['Properties']['RavenUseHostFrameworkReferences'] == 'false'
    assert evaluated['Properties']['RavenTargetCoreAssemblyName'] == 'NeoCLR.CoreProbe'
    assert evaluated['Properties']['TargetFramework'] == ''
    assert len(evaluated['Items']['Reference']) == 1
    assert not (root / 'obj').exists()
    checks['SDK-free project evaluation'] = 'passed'

    build('Design-time build does not compile', extra=('-p:DesignTimeBuild=true',))
    assert not (root / 'obj').exists()
    artifact = build('Debug compile/import/verify with spaces in project path')
    run(artifact, 'BUILD DOES NOT RUN ME\n')
    source.write_text((bundle / 'tools/samples/library-propagation-workflow.rvn').read_text())
    artifact = build('Release Result/Option/Void', configuration='Release')
    run(artifact, '42\nSaved\nCompleted\nOverflow\nValue found\n42\nAbsent\n')
    source.write_text((bundle / 'tools/samples/library-array-foreach.rvn').read_text())
    artifact = build('Changed source rebuild')
    run(artifact, 'Parse\nDivide\nEquals\nToString\nCompareTo\n42\n1\n2\n3\n')

    source.write_text('func Main() { missing() }\n')
    build('Compiler diagnostics and stale output', expected=False, error='RAV')
    source.write_text(success)
    build('Recovery after compiler failure')
    source.write_text('func Main() { System.Environment.Exit(0) }\n')
    build('Unavailable host API', expected=False, error='RAV')

    source.write_text(success)
    build('Prepare importer rejection')
    # Valid Raven/CLI code whose executable API is outside the admitted guest profile.
    source.write_text('func Main() { let value = System.NotImplementedException() }\n')
    build('Importer rejection and stale output', expected=False,
          error='Unsupported Result profile type: System.NotImplementedException')

    source.write_text(success)
    build('Prepare validation rejection')
    build('Missing SDK', expected=False, error='NEOBUILD001',
          extra=('-p:RavenSdkRoot=' + str(root / 'missing SDK'),))
    project.write_text(project_text.replace('<Compile Include="Main.rvn" />',
        '<Compile Include="Main.rvn" /><ProjectReference Include="Other.rvnproj" />'))
    build('Missing project reference', expected=False, error='NEOBUILD005')
    project.write_text(project_text)
    # A dependency is built before the compiler evaluates its metadata reference.
    library = root / 'library with spaces'
    library.mkdir()
    library_project = library / 'MathLibrary.rvnproj'
    library_text = project_text.replace('build/NeoCLR.', '../build/NeoCLR.').replace(
        '<ItemGroup>', '<PropertyGroup><OutputType>Library</OutputType></PropertyGroup><ItemGroup>')
    library_project.write_text(library_text)
    library_source = library / 'Main.rvn'
    library_source.write_text('public class Arithmetic { public static func Double(value: int) -> int { return value * 2 } }')
    consumer_text = project_text.replace('<Compile Include="Main.rvn" />',
        '<Compile Include="Main.rvn" /><ProjectReference Include="library with spaces/MathLibrary.rvnproj" />')
    project.write_text(consumer_text)
    source.write_text('import System.Console.*\nfunc Main() { WriteLine(Arithmetic.Double(21)) }')
    artifact = build('Project reference build order and dispatch')
    run(artifact, '42\n')
    library_source.write_text('public class Arithmetic { public static func Double(value: int) -> int { return value * 3 } }')
    artifact = build('Rebuild changed library')
    run(artifact, '63\n')
    artifact = build('Release project reference', configuration='Release')
    run(artifact, '63\n')
    library_source.write_text('public class Arithmetic { public static func Double(value: int) -> int { return missing } }')
    build('Dependency compile failure invalidates application', expected=False, error='RAV')
    library_source.write_text('public class Arithmetic { public static func Double(value: int) -> int { return value * 2 } }')
    library_project.write_text(library_text.replace('<OutputType>Library</OutputType>', '<OutputType>Exe</OutputType>'))
    build('Executable dependency rejected', expected=False, error='NEOBUILD006')
    library_project.write_text(library_text)
    alternate = root / 'different pack'
    (alternate / 'demo').mkdir(parents=True)
    for directory in ('bin', 'lib', 'tools'):
        (alternate / directory).symlink_to(bundle / directory, target_is_directory=True)
    (alternate / 'demo/NeoCLR.CoreProbe.dll').write_bytes((bundle / 'demo/NeoCLR.CoreProbe.dll').read_bytes() + b'\0')
    library_project.write_text(library_text.replace(escape(str(bundle)), escape(str(alternate))))
    build('Reference pack mismatch in dependency', expected=False, error='NEOBUILD007')
    library_project.write_text(library_text.replace('<Compile Include="Main.rvn" />',
        '<Compile Include="Main.rvn" /><ProjectReference Include="../Demo.rvnproj" />'))
    build('Cyclic library graph rejected by bounded contract', expected=False, error='NEOBUILD003')
    library_project.write_text(library_text)
    project.write_text(project_text)
    source.write_text(success)
    source.unlink()
    build('Missing source', expected=False, error='NEOBUILD004')
    source.write_text(success)
    build('Final recovery')
print(json.dumps(checks, indent=2))

"""Compile and run provisional date formatting, culture selection and access checks."""
import argparse
import json
from pathlib import Path
import subprocess
import tempfile
from collection_library import build, ROOT

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bridge', type=Path, required=True)
parser.add_argument('--runtime', type=Path, required=True)
parser.add_argument('--runner', type=Path, required=True, help='Built measure_async host runner for the larger contract fixture')
parser.add_argument('--output', type=Path)
args = parser.parse_args()
bridge, runtime, runner = (p.resolve() for p in (args.bridge, args.runtime, args.runner))
results = {}
with tempfile.TemporaryDirectory(prefix='neoclr-globalization-') as directory:
    work = Path(directory)
    (work / 'demo').mkdir()
    subprocess.run(['dotnet', str(bridge), '--reference-core', str(work / 'demo/NeoCLR.CoreProbe.dll')], check=True)
    system = work / 'System.neoil'
    system.write_text(build(ROOT / 'runtime/System.neoil'))
    project = work / 'Example.rvnproj'
    project.write_text(f'''<Project>
  <PropertyGroup><NeoCLRRoot>{work}</NeoCLRRoot></PropertyGroup>
  <Import Project="{ROOT}/build/NeoCLR.Raven.props" />
  <ItemGroup><Compile Include="Main.rvn" /></ItemGroup>
</Project>''')
    def compile_case(name, source, valid=True):
        (work / 'Main.rvn').write_text(source)
        output = work / name
        result = subprocess.run(['dotnet', str(bridge), '--project', str(project), str(output)], capture_output=True, text=True)
        if (result.returncode == 0) != valid or not valid and 'RAV' not in result.stdout + result.stderr:
            raise AssertionError(name + ': ' + result.stdout + result.stderr)
        return output / 'App.neoil'

    samples = ROOT / 'docs/experiments/raven-target/samples'
    for name, large in [('library-date-formatting', False), ('library-globalization', True)]:
        app = compile_case(name, (samples / (name + '.rvn')).read_text())
        subprocess.run([str(runtime), 'verify', str(app), '--system', str(system)], check=True, capture_output=True)
        command = [str(runner), str(app), str(system), '128', '1000000'] if large else [str(runtime), 'run', str(app), '--system', str(system)]
        result = subprocess.run(command, capture_output=True, text=True, timeout=180)
        expected = ('Calendar and culture formatting checks passed' if large else '2023-09-16\nא׳ תשרי ה׳תשפ״ד 12:34:56.0000000\n5784-01-01T12:34:56.0000000\n')
        if result.returncode or expected not in result.stdout:
            raise AssertionError(name + ': ' + result.stdout + result.stderr)
        results[name] = result.stdout.splitlines()

    for name, statement in [
        ('private-calendar-construction', 'let value = Calendar()'),
        ('private-culture-construction', 'let value = Culture()'),
        ('immutable-culture', 'Culture.Invariant.Name = "sv-SE"'),
        ('hidden-calendar-implementation', 'let value = HebrewCalendar()'),
        ('hidden-formatter-implementation', 'let value = HebrewDateTimeFormat()'),
    ]:
        compile_case(name, 'import System.*\nimport System.Globalization.*\nfunc Main() {\n    ' + statement + '\n}\n', False)
        results[name] = 'rejected'
print(json.dumps(results, indent=2, ensure_ascii=False))
if args.output:
    args.output.write_text(json.dumps(results, indent=2, ensure_ascii=False) + '\n')

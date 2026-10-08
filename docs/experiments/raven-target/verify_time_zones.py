"""Compile and run provisional time arithmetic, zone mapping and access checks."""
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
with tempfile.TemporaryDirectory(prefix='neoclr-time-zones-') as directory:
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
    for name, large in [('library-time-zones', False), ('library-time-contracts', True)]:
        app = compile_case(name, (samples / (name + '.rvn')).read_text())
        subprocess.run([str(runtime), 'verify', str(app), '--system', str(system)], check=True, capture_output=True)
        command = [str(runner), str(app), str(system), '128', '1000000'] if large else [str(runtime), 'run', str(app), '--system', str(system)]
        result = subprocess.run(command, capture_output=True, text=True, timeout=180)
        expected = ('Time and timezone checks passed' if large else 'This local time occurs twice:\n2024-10-27T02:30:00.0000000\n7200\n17299890000000000\n2024-10-27T02:30:00.0000000\n3600\n17299926000000000\n')
        if result.returncode or expected not in result.stdout:
            raise AssertionError(name + ': ' + result.stdout + result.stderr)
        results[name] = result.stdout.splitlines()
        if large:
            results["managed-object-statistics"] = result.stderr.splitlines()[-1]

    for name, statement in [
        ('private-zone-construction', 'let value = TimeZone()'),
        ('private-zoned-construction', 'let value = ZonedDateTime()'),
        ('immutable-zone', 'TimeZone.Utc.Id = "Europe/Stockholm"'),
        ('immutable-offset', 'TimeOffset.Zero.Seconds = 3'),
    ]:
        compile_case(name, 'import System.*\nimport System.Time.*\nimport System.Globalization.*\nfunc Main() {\n    ' + statement + '\n}\n', False)
        results[name] = 'rejected'
print(json.dumps(results, indent=2, ensure_ascii=False))
if args.output:
    args.output.write_text(json.dumps(results, indent=2, ensure_ascii=False) + '\n')

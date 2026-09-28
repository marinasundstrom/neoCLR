"""Compile and execute Function migration contracts against matching artifacts."""
import argparse
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
for name in ('runtime', 'bridge', 'system', 'reference'):
    parser.add_argument('--' + name, type=Path, required=True)
args = parser.parse_args()
tools = {name: getattr(args, name).resolve() for name in ('runtime', 'bridge', 'system', 'reference')}
here = Path(__file__).resolve().parent


def run(command, success=True):
    result = subprocess.run([str(x) for x in command], capture_output=True, text=True, timeout=180)
    assert (result.returncode == 0) == success, (command, result.stdout, result.stderr)
    return result


with tempfile.TemporaryDirectory(prefix='neoclr-function-types-') as directory:
    root = Path(directory)
    shutil.copyfile(here / 'Contracts.rvnproj', root / 'Contracts.rvnproj')
    shutil.copyfile(tools['reference'], root / 'NeoCLR.CoreProbe.dll')
    shutil.copyfile(here / 'Main.rvn', root / 'Main.rvn')
    output = root / 'descriptors'
    run(['dotnet', tools['bridge'], '--project', root / 'Contracts.rvnproj', output])
    run([tools['runtime'], 'verify', output / 'App.neoil', '--system', tools['system']])
    executed = run([tools['runtime'], 'run', output / 'App.neoil', '--system', tools['system']])
    assert executed.stdout == 'Nominal and structural descriptors passed\n' and not executed.stderr, executed
    print('descriptor-provider-selection: passed', flush=True)
    for member in ('Name', 'Namespace', 'FullName', 'Module', 'MetadataToken', 'DeclaringType', 'GetCustomAttributesData()'):
        (root / 'Main.rvn').write_text('import System.*\nimport System.Introspection.*\n'
            f'func Probe(info: TypeInfo) {{ _ = info.{member} }}\nfunc Main() -> int {{ return 0 }}\n')
        rejected = run(['dotnet', tools['bridge'], '--project', root / 'Contracts.rvnproj', root / ('reject-' + member.split('(')[0])], False)
        assert 'error' in (rejected.stdout + rejected.stderr).lower(), rejected
        print('common-type-rejects-' + member + ': passed', flush=True)

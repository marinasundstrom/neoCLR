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
    shutil.copyfile(here / 'OfType.rvn', root / 'Main.rvn')
    oftype = root / 'oftype'
    run(['dotnet', tools['bridge'], '--project', root / 'Contracts.rvnproj', oftype])
    run([tools['runtime'], 'verify', oftype / 'App.neoil', '--system', tools['system']])
    executed = run([tools['runtime'], 'run', oftype / 'App.neoil', '--system', tools['system']])
    assert executed.stdout == 'OfType query contracts passed\n' and not executed.stderr, executed
    print('oftype-query-contracts: passed', flush=True)
    shutil.copyfile(here / 'Callbacks.rvn', root / 'Main.rvn')
    callbacks = root / 'callbacks'
    run(['dotnet', tools['bridge'], '--project', root / 'Contracts.rvnproj', callbacks])
    for artifact in (callbacks / 'App.neoil', tools['system']):
        text = artifact.read_text()
        assert '.delegate ' not in text and 'System.Func<' not in text, artifact
        assert 'delegate.bind ' not in text, artifact
    run([tools['runtime'], 'verify', callbacks / 'App.neoil', '--system', tools['system']])
    executed = run([tools['runtime'], 'run', callbacks / 'App.neoil', '--system', tools['system']])
    assert executed.stdout == 'Structural function callbacks passed\n' and not executed.stderr, executed
    print('structural-function-callbacks: passed', flush=True)
    shutil.copyfile(here / 'Async.rvn', root / 'Main.rvn')
    asynchronous = root / 'async'
    run(['dotnet', tools['bridge'], '--project', root / 'Contracts.rvnproj', asynchronous])
    run([tools['runtime'], 'verify', asynchronous / 'App.neoil', '--system', tools['system']])
    executed = run([tools['runtime'], 'run', asynchronous / 'App.neoil', '--system', tools['system']])
    assert executed.stdout == '42\nTrue\n41\n21\n7\n' and not executed.stderr, executed
    print('structural-function-async: passed', flush=True)
    shutil.copyfile(here.parent / 'raven-target/samples/library-comparers.rvn', root / 'Main.rvn')
    comparers = root / 'comparers'
    run(['dotnet', tools['bridge'], '--project', root / 'Contracts.rvnproj', comparers])
    run([tools['runtime'], 'verify', comparers / 'App.neoil', '--system', tools['system']])
    executed = run([tools['runtime'], 'run', comparers / 'App.neoil', '--system', tools['system']])
    assert executed.stdout == 'Comparer contract passed\n' and not executed.stderr, executed
    print('function-comparer-adapters: passed', flush=True)
    (root / 'Main.rvn').write_text('public delegate Named(value: int) -> int\n'
        'public func Identity(value: int) -> int => value\n'
        'func Main() -> int { let f: Named = Identity; return f(42) }\n')
    rejected = run(['dotnet', tools['bridge'], '--project', root / 'Contracts.rvnproj', root / 'reject-named-delegate'], False)
    assert not (root / 'reject-named-delegate/App.neoil').exists(), rejected
    print('named-delegate-admission-rejected: passed', flush=True)
    for member in ('Name', 'Namespace', 'FullName', 'Module', 'MetadataToken', 'DeclaringType', 'GetCustomAttributesData()'):
        (root / 'Main.rvn').write_text('import System.*\nimport System.Introspection.*\n'
            f'func Probe(info: TypeInfo) {{ _ = info.{member} }}\nfunc Main() -> int {{ return 0 }}\n')
        rejected = run(['dotnet', tools['bridge'], '--project', root / 'Contracts.rvnproj', root / ('reject-' + member.split('(')[0])], False)
        assert 'error' in (rejected.stdout + rejected.stderr).lower(), rejected
        print('common-type-rejects-' + member + ': passed', flush=True)

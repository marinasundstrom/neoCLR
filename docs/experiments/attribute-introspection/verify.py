"""Compile and execute metadata-only attribute inspection through public APIs."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
parser.add_argument('--runner', type=Path, required=True)
args = parser.parse_args()
bundle = args.toolchain_root.resolve()
here = Path(__file__).resolve().parent
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
with tempfile.TemporaryDirectory(prefix='neoclr-attributes-') as folder:
    root = Path(folder)
    for name in ('Main.rvn', 'Routes.rvn', 'Attributes.rvnproj'):
        shutil.copyfile(here / name, root / name)
    built = subprocess.run(['dotnet', 'msbuild', str(root / 'Attributes.rvnproj'), '-nologo', '-v:minimal', '-t:NeoCLRImport'], env=env, capture_output=True, text=True, timeout=240)
    assert built.returncode == 0, built.stdout + built.stderr
    imported = next((root / 'obj').glob('**/imported/App.neoil'))
    result = subprocess.run([str(args.runner.resolve()), str(imported), str(bundle / 'lib/System.neoil'), '512', '100000000'], capture_output=True, text=True, timeout=180)
    assert result.returncode == 0, result.stdout + result.stderr
    assert result.stdout == 'CatalogRoutes/ListItems => /items\nCatalogRoutes/GetItem => /items/{id}\nAttribute introspection checks passed\n', result.stdout
    assert 'live=0' in result.stderr, result.stderr
    print(result.stdout + result.stderr)

    # Unsupported user attribute constants must fail import, never vanish.
    source = (root / 'Main.rvn').read_text()
    (root / 'Main.rvn').write_text(source.replace('number: int, enabled: bool', 'number: long, enabled: bool').replace(', 42, true)', ', 42L, true)'))
    rejected = subprocess.run(['dotnet', 'msbuild', str(root / 'Attributes.rvnproj'), '-nologo', '-v:minimal', '-t:NeoCLRImport'], env=env, capture_output=True, text=True, timeout=240)
    assert rejected.returncode != 0 and 'Unsupported attribute argument type: System.Int64' in rejected.stdout + rejected.stderr, rejected.stdout + rejected.stderr
    print('Unsupported Int64 attribute argument rejected at import')

    named = source.replace('public class LabelAttribute : Attribute {', 'public class LabelAttribute : Attribute {\n    var Extra: string { get; set; } = ""')
    named = named.replace('[Label("first"),', '[Label("first", Extra: "value"),')
    (root / 'Main.rvn').write_text(named)
    rejected = subprocess.run(['dotnet', 'msbuild', str(root / 'Attributes.rvnproj'), '-nologo', '-v:minimal', '-t:NeoCLRImport'], env=env, capture_output=True, text=True, timeout=240)
    assert rejected.returncode != 0 and 'Unsupported attribute metadata: LabelAttribute' in rejected.stdout + rejected.stderr, rejected.stdout + rejected.stderr
    print('Named attribute argument rejected at import')

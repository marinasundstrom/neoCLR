"""Configure the extracted demonstration for its current location."""
import argparse
import json
from pathlib import Path
import sys

root = Path(__file__).resolve().parent
sys.path.insert(0, str(root / 'tools'))
from configure_tasks import configure
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--sdk', type=Path, help='Optional separately installed experimental Raven SDK')
args = parser.parse_args()
project = root / 'demo/Demo.rvnproj'
configure(project, None, root / 'bin/neoclr', root / 'tools/bridge/Probe.dll', root / 'lib/System.neoil')
settings = {'raven.languageServerPath': str(root / 'tools/server/Raven.LanguageServer.dll')}
if args.sdk:
    settings['raven.sdkPath'] = str(args.sdk.resolve())
(project.parent / '.vscode/settings.json').write_text(json.dumps(settings, indent=2) + '\n')
print('Open this folder in VS Code:', project.parent)

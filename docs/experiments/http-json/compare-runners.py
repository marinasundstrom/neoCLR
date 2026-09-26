"""Compare two runtime runners on identical compiled mapped HTTP/JSON inputs."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import selectors
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
parser.add_argument('--baseline', type=Path, required=True)
parser.add_argument('--candidate', type=Path, required=True)
parser.add_argument('--repeat', type=int, default=3)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
if args.repeat < 1:
    parser.error('--repeat must be at least 1')
bundle = args.toolchain_root.resolve()
here = Path(__file__).resolve().parent
runners = {name: getattr(args, name).resolve() for name in ('baseline', 'candidate')}
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def stats(result):
    values = {key: int(value) for key, value in re.findall(r'(\w+)=(\d+)', result)}
    assert values['live'] == 0 and values['collections'] > 0, result
    return values


report = {'runner_sha256': {name: sha(path) for name, path in runners.items()},
          'heap_objects': 1024, 'instructions': 100000000, 'runs': []}
with tempfile.TemporaryDirectory(prefix='neoclr-http-runtime-comparison-') as folder:
    root = Path(folder)
    library = root / 'System.neoil'
    shutil.copyfile(bundle / 'lib/System.neoil', library)
    apps = {}
    for name in ('Server', 'Client'):
        target = root / name.lower()
        target.mkdir()
        mapping = here.parent / 'json-object-mapping'
        shutil.copyfile(here / (name + '.rvnproj'), target / (name + '.rvnproj'))
        shutil.copyfile(mapping / ('Http' + name + '.rvn'), target / (name + '.rvn'))
        shutil.copyfile(mapping / 'HttpApplication.rvn', target / 'Application.rvn')
        result = subprocess.run(['dotnet', 'msbuild', str(target / (name + '.rvnproj')),
                                 '-nologo', '-v:minimal', '-t:NeoCLRImport'],
                                env=env, capture_output=True, text=True, timeout=240)
        assert result.returncode == 0, result.stdout + result.stderr
        apps[name] = next((target / 'obj').glob('**/imported/App.neoil'))
    report['input_sha256'] = {name: sha(path) for name, path in {**apps, 'System': library}.items()}

    def command(runner, app, *arguments, live=False):
        return [str(runner), str(app), str(library), '1024', '100000000'] + (
            ['--live-output'] if live else []) + ['--', *arguments]

    for iteration in range(1, args.repeat + 1):
        # Alternate order; neither runner should always benefit from going second.
        order = ('baseline', 'candidate') if iteration % 2 else ('candidate', 'baseline')
        for name in order:
            record = {'iteration': iteration, 'runner': name}
            print(f'{name} iteration {iteration}/{args.repeat}', flush=True)
            runner = runners[name]
            server = subprocess.Popen(command(runner, apps['Server'], '2', live=True),
                                      stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            try:
                with selectors.DefaultSelector() as selector:
                    selector.register(server.stdout, selectors.EVENT_READ)
                    assert selector.select(180), 'Server did not report a port'
                port = server.stdout.readline().strip()
                assert port.isdecimal(), f'Invalid server port: {port!r}'
                client = subprocess.run(command(runner, apps['Client'], f'http://localhost:{port}/'),
                                        capture_output=True, text=True, timeout=180)
                record['client_exit_code'] = client.returncode
                record['client_stderr'] = client.stderr
                assert client.returncode == 0, client.stdout + client.stderr
                assert json.loads(client.stdout) == {'accepted': True}, client.stdout
                record['client'] = stats(client.stderr)
                output, errors = server.communicate(timeout=180)
                record['server_stderr'] = errors
                assert server.returncode == 0 and output == 'Reports served\n', output + errors
                record['server'] = stats(errors)
                record['outcome'] = 'passed'
                # Structured measurements suffice for successful runs.
                del record['client_stderr'], record['server_stderr']
            except (AssertionError, subprocess.TimeoutExpired, ValueError, KeyError) as error:
                record['outcome'] = 'failed'
                record['error'] = str(error)
            finally:
                if server.poll() is None:
                    server.kill()
                    output, errors = server.communicate()
                    record['terminated_server_output'] = output
                    record['terminated_server_stderr'] = errors
                report['runs'].append(record)
                args.output.write_text(json.dumps(report, indent=2) + '\n')
            print(f"{name}: {record['outcome']}", flush=True)
# Diagnostic comparisons retain both failures and later observations; no retry
# turns a failed check into success, and any failure makes the command fail.
raise SystemExit(0 if all(run['outcome'] == 'passed' for run in report['runs']) else 1)

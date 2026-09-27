"""Prepare attributed routes at startup and exercise the cached mapper and HTTP case."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import selectors
import json
from urllib.request import urlopen, Request
from urllib.error import HTTPError


def run(command, env=None, success=True, timeout=240):
    result = subprocess.run(command, env=env, capture_output=True, text=True, timeout=timeout)
    if success:
        assert result.returncode == 0, result.stdout + result.stderr
    else:
        assert result.returncode != 0, result.stdout + result.stderr
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--toolchain-root', type=Path, required=True)
    parser.add_argument('--runner', type=Path, required=True)
    parser.add_argument('--sdk', type=Path, help='Separate extracted SDK; defaults to TOOLCHAIN_ROOT/raven-sdk')
    parser.add_argument('--consumer-only', action='store_true')
    parser.add_argument('--schemas-only', action='store_true')
    parser.add_argument('--schema', action='append', help='Only check this named invalid schema')
    args = parser.parse_args()
    bundle = args.toolchain_root.resolve()
    runner = args.runner.resolve()
    here = Path(__file__).resolve().parent
    env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(args.sdk.resolve() if args.sdk else bundle / 'raven-sdk'))
    with tempfile.TemporaryDirectory(prefix='neoclr-route-mapper-') as folder:
        root = Path(folder)
        for name in ('Routes.rvn', 'Mapper.rvn', 'Main.rvn'):
            shutil.copyfile(here / name, root / name)
        for name in ('Mapper', 'Server', 'Client'):
            shutil.copyfile(here / (name + '.rvnproj'), root / (name + '.rvnproj'))
        app = root / 'Mapper.rvnproj'
        original = (root / 'Routes.rvn').read_text()
        if not args.schemas_only:
            run(['dotnet', 'msbuild', str(app), '-nologo', '-v:minimal', '-t:NeoCLRImport'], env)
            imported = max((root / 'obj').glob('**/imported/App.neoil'), key=lambda path: path.stat().st_mtime_ns)
            result = run([str(runner), str(imported), str(bundle / 'lib/System.neoil'), '512', '100000000'])
            assert result.stdout.splitlines() == ['Runtime route mapper checks passed'], result.stdout
            assert 'live=0' in result.stderr, result.stderr
            print(result.stdout + result.stderr)
            if args.consumer_only:
                return
            apps = {}
            for name in ('Server', 'Client'):
                shutil.copyfile(here / (name + '.rvn'), root / (name + '.rvn'))
                target = root / (name + '.rvnproj')
                run(['dotnet', 'msbuild', str(target), '-nologo', '-v:minimal', '-t:NeoCLRImport'], env)
                imported = max((root / 'obj').glob('**/imported/App.neoil'), key=lambda path: path.stat().st_mtime_ns)
                apps[name] = root / (name + '.neoil')
                shutil.copyfile(imported, apps[name])
            def command(name, *arguments, live=False):
                return [str(runner), str(apps[name]), str(bundle / 'lib/System.neoil'), '1024', '100000000'] + (['--live-output'] if live else []) + ['--', *arguments]
            def serve(consume, count):
                server = subprocess.Popen(command('Server', str(count), live=True), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                try:
                    with selectors.DefaultSelector() as selector:
                        selector.register(server.stdout, selectors.EVENT_READ)
                        assert selector.select(180), 'Server did not report a port'
                    port = server.stdout.readline().strip()
                    assert port.isdecimal(), port
                    consume(int(port))
                    output, errors = server.communicate(timeout=180)
                    assert server.returncode == 0 and output == 'Items served\n', output + errors
                    assert 'live=0' in errors, errors
                    print('HTTP server: ' + errors.strip())
                finally:
                    if server.poll() is None:
                        server.kill()
                        server.communicate()
            item = {'id': 42, 'name': 'Desk lamp'}
            cases = [('GET', '/items', 200, [item]), ('GET', '/items/42', 200, item),
                     ('GET', '/items/%34%32?details=true', 200, item),
                     ('GET', '/items/9', 404, {'error': 'Not found'}),
                     ('GET', '/missing', 404, {'error': 'Not found'}),
                     ('POST', '/items', 404, {'error': 'Not found'}),
                     ('GET', '/items/no', 400, {'error': 'Invalid route'}),
                     ('GET', '/items/2147483648', 400, {'error': 'Invalid route'}),
                     ('GET', '/items/%2F', 400, {'error': 'Invalid route'})]
            def independent(port):
                for method, target, status, body in cases:
                    try:
                        response = urlopen(Request(f'http://127.0.0.1:{port}' + target, method=method), timeout=30)
                    except HTTPError as error:
                        response = error
                    with response:
                        assert response.status == status, (target, response.status)
                        assert json.load(response) == body, target
            serve(independent, len(cases))
            def client(port):
                result = run(command('Client', f'http://127.0.0.1:{port}/'))
                assert [json.loads(line) for line in result.stdout.splitlines()] == [[item], item], result.stdout
                assert 'live=0' in result.stderr, result.stderr
                print('HTTP client: ' + result.stderr.strip())
            serve(client, 2)
            print('HTTP checks: 9 independent requests and a 2-request neoCLR client/server pair passed.')
            # Exercise mixed payloads and binding by name rather than pattern order.
            (root / 'Routes.rvn').write_text(original.replace('case GetItem(id: int)', 'case GetItem(id: int)\n    [RoutePattern("/owners/{name}/items/{id}")]\n    case OwnerItem(id: int, name: string)'))
            (root / 'Main.rvn').write_text('''import System.*
    import System.Result.*
    func Main() {
        let Ok(parser) = AppRoutesParser.Create() else {
            System.Fault("Creation failed")
            return
        }
        let Ok(AppRoutes.OwnerItem(id, name)) = parser.Parse("/owners/Caf%C3%A9/items/42") else {
            System.Fault("Mixed route missing")
            return
        }
        if id != 42 || name != "Café" {
            System.Fault("Name binding failed")
            return
        }
        Console.WriteLine("Mixed route checks passed")
    }
    ''')
            run(['dotnet', 'msbuild', str(app), '-nologo', '-v:minimal', '-t:NeoCLRImport'], env)
            imported = max((root / 'obj').glob('**/imported/App.neoil'), key=lambda path: path.stat().st_mtime_ns)
            result = run([str(runner), str(imported), str(bundle / 'lib/System.neoil'), '512', '100000000'])
            assert result.stdout == 'Mixed route checks passed\n' and 'live=0' in result.stderr, result.stdout + result.stderr
            print(result.stdout + result.stderr)
        rejected = {
            'repeated attribute': original.replace('[RoutePattern("/items")]', '[RoutePattern("/items")]\n    [RoutePattern("/other")]'),
            'missing attribute': original.replace('    [RoutePattern("/items")]\n', ''),
            'missing argument': original.replace('/items/{id}', '/items/{other}'),
            'unsupported type': original.replace('id: int', 'id: long'),
            'ambiguous patterns': original.replace('/items")', '/items/{id}")').replace('case ListItems', 'case ListItems(id: int)'),
            'encoded overlap': original.replace('/items")', '/items/%34%32")'),
            'invalid UTF8 literal': original.replace('/items")', '/%FF")'),
            'bad pattern': original.replace('/items/{id}', '/items/{id?}'),
        }
        if args.schema:
            assert set(args.schema) <= rejected.keys(), "Unknown schema name"
        for reason, source in rejected.items():
            if args.schema and reason not in args.schema:
                continue
            (root / 'Routes.rvn').write_text(source)
            (root / 'Main.rvn').write_text('import System.*\nimport System.Result.*\nfunc Main() {\n    if AppRoutesParser.Create() is Ok(_) {\n        System.Fault("Invalid schema was accepted")\n    }\n    Console.WriteLine("Schema rejected at startup")\n}\n')
            if reason == 'unsupported type':
                result = run(['dotnet', 'msbuild', str(app), '-nologo', '-v:minimal', '-t:NeoCLRImport'], env, success=False)
                assert 'Unsupported application type: AppRoutes' in result.stdout + result.stderr
                print('Rejected schema by importer: ' + reason, flush=True)
                continue
            run(['dotnet', 'msbuild', str(app), '-nologo', '-v:minimal', '-t:NeoCLRImport'], env)
            imported = max((root / 'obj').glob('**/imported/App.neoil'), key=lambda path: path.stat().st_mtime_ns)
            result = run([str(runner), str(imported), str(bundle / 'lib/System.neoil'), '512', '100000000'])
            assert result.stdout == 'Schema rejected at startup\n' and 'live=0' in result.stderr, result.stdout + result.stderr
            print('Rejected schema: ' + reason, flush=True)


if __name__ == '__main__':
    main()

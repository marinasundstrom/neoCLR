"""Compile attributed routes, generate a reusable mapper, and execute its consumer."""
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
    args = parser.parse_args()
    bundle = args.toolchain_root.resolve()
    runner = args.runner.resolve()
    here = Path(__file__).resolve().parent
    env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
    compiler = ['dotnet', str(bundle / 'raven-sdk/tools/rvnc/rvnc.dll')]
    core = bundle / 'demo/NeoCLR.CoreProbe.dll'
    run(['dotnet', 'build', str(here / 'Generator.csproj'), '-v:quiet', '-p:NeoCLRRoot=' + str(bundle)])
    generator = ['dotnet', str(here / 'bin/Debug/net11.0/Generator.dll')]
    with tempfile.TemporaryDirectory(prefix='neoclr-route-mapper-') as folder:
        root = Path(folder)
        for name in ('Routes.rvn', 'Attributes.rvn', 'Main.rvn'):
            shutil.copyfile(here / name, root / name)
        def project(name, files, output):
            path = root / (name + '.rvnproj')
            path.write_text('<Project DefaultTargets="Build">\n'
                f'<Import Project="{bundle}/build/NeoCLR.Raven.props" />\n'
                f'<PropertyGroup><OutputType>{output}</OutputType></PropertyGroup>\n<ItemGroup>\n'
                + ''.join(f'<Compile Include="{f}" />\n' for f in files)
                + f'</ItemGroup>\n<Import Project="{bundle}/build/NeoCLR.Raven.targets" />\n</Project>\n')
            return path
        schema = project('Schema', ['Attributes.rvn', 'Routes.rvn'], 'Library')
        app = project('Mapper', ['Attributes.rvn', 'Routes.rvn', 'Generated.rvn', 'Main.rvn'], 'Exe')
        def generate(success=True):
            run([*compiler, str(schema), '--no-project-restore', '-o', str(root / 'schema')], env)
            return run([*generator, str(root / 'schema/Schema.dll'), str(core), str(root / 'Generated.rvn')], success=success)
        generated = generate()
        print(generated.stdout, end='')
        print(run(['dotnet', str(bundle / 'tools/bridge/Probe.dll'), '--scalar-union-checks',
                   str(root / 'schema/Schema.dll'), str(core)]).stdout, end='')
        run(['dotnet', 'msbuild', str(app), '-nologo', '-v:minimal', '-t:NeoCLRImport'], env)
        imported = max((root / 'obj').glob('**/imported/App.neoil'), key=lambda path: path.stat().st_mtime_ns)
        result = run([str(runner), str(imported), str(bundle / 'lib/System.neoil'), '512', '100000000'])
        assert result.stdout.splitlines() == ['Attributed route mapper checks passed'], result.stdout
        assert 'live=0' in result.stderr, result.stderr
        print(result.stdout + result.stderr)
        apps = {}
        for name in ('Server', 'Client'):
            shutil.copyfile(here / (name + '.rvn'), root / (name + '.rvn'))
            files = ['Attributes.rvn', 'Routes.rvn', 'Generated.rvn', name + '.rvn'] if name == 'Server' else ['Client.rvn']
            target = project(name, files, 'Exe')
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
        original = (root / 'Routes.rvn').read_text()
        # Exercise mixed payloads and binding by name rather than pattern order.
        (root / 'Routes.rvn').write_text(original.replace('case GetItem(id: int)', 'case GetItem(id: int)\n    [RoutePattern("/owners/{name}/items/{id}")]\n    case OwnerItem(id: int, name: string)'))
        generate()
        (root / 'Main.rvn').write_text('''import System.*
import System.Result.*
func Main() {
    let Ok(parser) = AppRoutesParser.Create() else {
        System.Fail("Creation failed")
        return
    }
    let Ok(AppRoutes.OwnerItem(id, name)) = parser.Parse("/owners/Caf%C3%A9/items/42") else {
        System.Fail("Mixed route missing")
        return
    }
    if id != 42 || name != "Café" {
        System.Fail("Name binding failed")
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
            'missing attribute': original.replace('    [RoutePattern("/items")]\n', ''),
            'missing argument': original.replace('/items/{id}', '/items/{other}'),
            'unsupported type': original.replace('id: int', 'id: long'),
            'ambiguous patterns': original.replace('/items")', '/items/{id}")').replace('case ListItems', 'case ListItems(id: int)'),
            'encoded overlap': original.replace('/items")', '/items/%34%32")'),
            'invalid UTF8 literal': original.replace('/items")', '/%FF")'),
            'bad pattern': original.replace('/items/{id}', '/items/{id?}'),
        }
        for reason, source in rejected.items():
            (root / 'Routes.rvn').write_text(source)
            generated = generate(success=False)
            assert not (root / 'Generated.rvn').exists(), 'Stale generated output after failure'
            print('Rejected schema: ' + reason)


if __name__ == '__main__':
    main()

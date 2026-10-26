#!/usr/bin/env python3
"""Compile public SQL consumers; qualify local PostgreSQL and hostile protocol peers."""
import argparse
from contextlib import ExitStack
import hashlib
import json
import os
from pathlib import Path
import socket
import struct
import subprocess
import threading
import shutil
import xml.etree.ElementTree as ET
import time

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
p = argparse.ArgumentParser(description=__doc__)
for name in ('compiler', 'libraries', 'runtime', 'jit-compiler', 'jit-host', 'output'):
    p.add_argument('--' + name, type=Path, required=True)
p.add_argument('--postgres-port', type=int, required=True, help='Disposable trust-authenticated local postgres database/user postgres')
p.add_argument('--skip-interpreter', action='store_true', help='Run only the routine JIT checks')
p.add_argument('--queries-only', action='store_true', help='Focused mapper and live PostgreSQL consumer, skipping unchanged wire peers')
p.add_argument('--aot', action='store_true', help='Also emit and link the SQL consumer against the supplied native host (macOS)')
a = p.parse_args()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
lib = a.libraries.resolve()
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
report = dict(passed=False, commands=[], cases=[], inputs={})
for path in [Path(__file__), *HERE.glob('*.rvn'), a.compiler, a.runtime, a.jit_compiler, a.jit_host,
             ROOT / 'runtime/raven/compiler-services/RuntimeAwait.rvn',
             *(a.compiler.parent / name for name in ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')),
             *lib.glob('*.dll'), lib / 'System.runtime.neox', lib / 'bundle.json',
             *(ROOT / 'runtime/raven/src/System/Data/Sql').glob('*.rvn'),
             *(ROOT / 'runtime/raven/providers/System.Data.PostgreSql').glob('*.rvn')]:
    report['inputs'][str(path.resolve())] = sha(path)


def run(label, command, expected=None):
    command = list(map(str, command))
    started = time.monotonic()
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=300)
    (out / (label + '.stdout')).write_text(result.stdout)
    (out / (label + '.stderr')).write_text(result.stderr)
    report['commands'].append(dict(label=label, command=command, exitCode=result.returncode, elapsedSeconds=round(time.monotonic() - started, 3),
                                   stdout=result.stdout if expected else '', stderr=result.stderr[-4000:]))
    if result.returncode or (expected is not None and result.stdout != expected):
        raise RuntimeError(label + ': ' + result.stdout[-4000:] + result.stderr[-4000:])
    return result


modules = [lib / (name + '.dll') for name in ('System.Runtime', 'System.Data', 'System.Networking', 'System.Data.PostgreSql')]
catalog = json.loads((lib / 'bundle.json').read_text())
base = ['dotnet', a.compiler.resolve(), 'neoclr', '--core-reference', lib / 'Core.dll',
        '--runtime-seed', lib / 'System.runtime.neox', '--bootstrap-intrinsics',
        '--bootstrap-ownership', lib / 'ownership.json', '--object-library', 'System.Runtime',
        '--async-library', 'System.Runtime']
for path in modules:
    base += ['--reference', path]
flags = ['--compile-system', '--bind-user-fault', '--reference-arena', '--native-gc',
         '--bind-console-write-line', '--bind-utf8-text', '--bind-int32-to-string', '--native-stack-budget',
         '--bind-integer-text', '--bind-character-text', '--bind-task-queue', '--bind-scheduler', '--bind-timers',
         '--bind-socket-listener', '--bind-socket-accept', '--bind-socket-transfer', '--bind-socket-client', '--bind-runtime-async']


def command(case, mode, arguments):
    assembly = out / (case + '.dll')
    dependencies = ['--system', lib / 'System.runtime.neox', '--object-root', lib / 'System.Runtime.dll']
    for path in modules:
        dependencies += ['--module', path]
    if mode == 'interpreter':
        return [a.runtime.resolve(), 'run', assembly, '--execution', 'interpreter', *dependencies,
                '--instructions', '200000000', '--', *arguments]
    result = [a.jit_compiler.resolve(), '--jit-run', '--closed-world', assembly, '@entry', a.jit_host.resolve(), *dependencies, *flags]
    if case == 'Main':
        result += ['--reflection-roots', out / 'reflection.json']
    for argument in arguments:
        result += ['--jit-entry-argument', argument]
    return result


def frame(kind, payload=b''):
    return kind + struct.pack('!i', len(payload) + 4) + payload


def exact(connection, count):
    data = b''
    while len(data) < count:
        chunk = connection.recv(count - len(data))
        if not chunk:
            raise RuntimeError('Peer EOF')
        data += chunk
    return data


def peer(listener, mode, errors):
    try:
        with listener.accept()[0] as connection:
            connection.settimeout(120)
            length = struct.unpack('!i', exact(connection, 4))[0]
            assert 4 <= length <= 65540
            startup = exact(connection, length - 4)
            assert startup[:4] == struct.pack('!i', 196608)
            assert b'client_encoding\0UTF8\0' in startup
            if mode == 'authentication':
                connection.sendall(frame(b'R', struct.pack('!i', 3)))
            elif mode == 'startup':
                connection.sendall(b'R' + struct.pack('!i', 3))
            elif mode == 'timeout':
                # Do not send a response; the shared opening deadline must close the connection.
                assert connection.recv(1) == b''
                return
            else:
                connection.sendall(frame(b'R', struct.pack('!i', 0)) + frame(b'Z', b'I'))
                requests = []
                for _ in range(5):
                    tag = exact(connection, 1)
                    size = struct.unpack('!i', exact(connection, 4))[0]
                    assert 4 <= size <= 65540
                    requests.append((tag, exact(connection, size - 4)))
                assert [tag for tag, _ in requests] == [b'P', b'B', b'D', b'E', b'S']
                assert requests[0][1] == b'\0SELECT 1\0\0\0'
                assert requests[1][1] == b'\0\0\0\0\0\0\0\0'
                if mode == 'oversized':
                    connection.sendall(b'T' + struct.pack('!i', 65541))
                elif mode == 'copy':
                    connection.sendall(frame(b'G'))
                else:
                    column = struct.pack('!h', 1) + b'value\0' + struct.pack('!ihihih', 0, 0, 23, 4, -1, 0)
                    response = frame(b'1') + frame(b'2') + frame(b'T', column)
                    if mode == 'length':
                        response += frame(b'D', struct.pack('!hi', 1, -2))
                    elif mode == 'utf8':
                        response += frame(b'D', struct.pack('!hi', 1, 1) + b'\xff')
                    elif mode == 'rows':
                        response += frame(b'D', struct.pack('!hi', 1, 2) + b'42') * 1025
                    elif mode == 'columns':
                        response += frame(b'D', struct.pack('!h', 2))
                    else:
                        response += frame(b'D', struct.pack('!hi', 1, 2) + b'42') + frame(b'C', b'SELECT 1\0') + frame(b'Z', b'I')
                    if mode == 'fragmented':
                        for byte in response:
                            connection.sendall(bytes([byte]))
                            time.sleep(0.001)
                    else:
                        connection.sendall(response)
            # No password, further SQL or resynchronization is allowed on these retired sessions.
            assert connection.recv(1) == b''
    except Exception as error:
        errors.append(repr(error))


try:
    for case in (['Main', 'Contracts'] if a.queries_only else ['Main', 'Failures', 'Contracts']):
        if catalog.get('kind') == 'native-source-core-class-library':
            # Import the current native core, runtime contracts and async provider
            # together, just as an installed SDK consumer does.
            project_dir = out / (case + '-project')
            project_dir.mkdir()
            project = ET.Element('Project', Sdk='Microsoft.NET.Sdk')
            ET.SubElement(project, 'Import', Project=str(lib / 'NeoCLR.ClassLibrary.props'))
            properties = ET.SubElement(project, 'PropertyGroup')
            for key, value in dict(TargetFramework='net10.0', OutputType='Exe',
                                   AssemblyName=case, EnableDefaultCompileItems='false').items():
                ET.SubElement(properties, key).text = value
            ET.SubElement(ET.SubElement(project, 'ItemGroup'), 'Compile', Include=str(HERE / (case + '.rvn')))
            project_path = project_dir / (case + '.rvnproj')
            ET.ElementTree(project).write(project_path, encoding='unicode')
            run(case + '-compile', ['dotnet', a.compiler.resolve(), 'neoclr', '--project', project_path])
            shutil.copyfile(project_dir / 'bin/neoclr' / (case + '.dll'), out / (case + '.dll'))
        else:
            args = base + ['-o', out / (case + '.dll'), HERE / (case + '.rvn')]
            args += ['--runtime-async-poc', 'RuntimeAwaitProvider.Bridge', ROOT / 'runtime/raven/compiler-services/RuntimeAwait.rvn']
            run(case + '-compile', args)
    disassembly = out / 'Main.neoil'
    run('metadata', [a.runtime.resolve(), 'disassemble', out / 'Main.dll', disassembly])
    listing = disassembly.read_text()
    module, _ = json.JSONDecoder().raw_decode(listing[listing.index('{'):])
    people = [index for index, type in enumerate(module['types']) if type['name'].endswith(('.T__506572736F6E', '.T__57696465506172616D6574657273'))]
    assert len(people) == 2, 'Expected Person and WideParameters metadata definitions'
    (out / 'reflection.json').write_text(json.dumps(dict(schemaVersion=2, types=[dict(
        definition=dict(module=module['name'], revision=module['revision'], index=index),
        construct=True, properties=True, getters=True, setters=True) for index in people]), indent=2))
    for mode in (['jit'] if a.skip_interpreter else ['jit', 'interpreter']):
        run('contracts-' + mode, command('Contracts', mode, []), 'PASS SQL contracts\n')
    report['cases'].append('owned named/positional snapshots, strict access, conversion bounds and explicit maps')
    for mode in (['jit'] if a.skip_interpreter else ['jit', 'interpreter']):
        run('postgres-' + mode, command('Main', mode, [str(a.postgres_port)]), 'PASS SQL PostgreSQL\n')
        report['cases'].append('real PostgreSQL: ' + mode)
    if a.aot:
        native = command('Main', 'jit', [])
        native.remove('--jit-run')
        native[native.index(a.jit_host.resolve())] = out / 'Main.o'
        run('postgres-aot-compile', native)
        (out / 'main.c').write_text(
            '#include "native-gc.h"\n'
            'extern int32_t neoclr_entry_args_v1(int32_t,int32_t*,neoclr_aot_context*,const char**);\n'
            'extern int32_t neoclr_jit_run_v1(const void*,int32_t,int32_t,const char**);\n'
            'int main(int argc,char**argv) { return neoclr_jit_run_v1((const void*)'
            'neoclr_entry_args_v1,1,argc,(const char**)argv); }\n')
        run('postgres-aot-link', [os.environ.get('CC', 'cc'), '-O2', '-I', ROOT / 'docs/experiments/aot-console',
             out / 'main.c', out / 'Main.o', a.jit_host.resolve(),
             '-Wl,-rpath,' + str(a.jit_host.resolve().parent), '-o', out / 'app'])
        run('postgres-aot', [out / 'app', str(a.postgres_port)], 'PASS SQL PostgreSQL\n')
        report['cases'].append('real PostgreSQL: aot (shared native host)')
    cases = () if a.queries_only else ('authentication', 'startup', 'timeout', 'oversized', 'copy', 'length', 'utf8', 'columns', 'rows', 'fragmented')
    # One compiled native consumer drives all peers, avoiding repeated JIT compilation.
    with ExitStack() as stack:
        peers = []
        arguments = []
        for mode in cases:
            errors = []
            listener = stack.enter_context(socket.socket())
            listener.bind(('127.0.0.1', 0))
            listener.listen(1)
            listener.settimeout(240)
            thread = threading.Thread(target=peer, args=(listener, mode, errors), daemon=True)
            thread.start()
            peers.append((mode, thread, errors))
            arguments += [str(listener.getsockname()[1]), mode]
        if cases:
            run('protocol-jit', command('Failures', 'jit', arguments), ''.join('PASS SQL ' + mode + '\n' for mode in cases))
        for mode, thread, errors in peers:
            thread.join(5)
            if thread.is_alive() or errors:
                raise RuntimeError(mode + ' peer: ' + str(errors))
            report['cases'].append(mode)
    for path, expected in report['inputs'].items():
        if sha(path) != expected:
            raise RuntimeError('Input changed during validation: ' + path)
    report['passed'] = True
finally:
    (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
print('PASS ' + ', '.join(report['cases']))

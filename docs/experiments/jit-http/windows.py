"""Build an explicitly exported JIT helper DLL from a qualified Windows AOT recipe."""
from pathlib import Path
import re


def recipe(prior, root, output):
    if not prior.get('passed') or prior.get('profile') != 'windows-x64-http-v1':
        raise ValueError('Requires a successful Windows HTTP build report')
    native = next(item['command'] for item in prior['commands']
                  if item['phase'] == 'native-compilation')
    link = next(item['command'] for item in prior['commands']
                if item['phase'] == 'host-compile-link')
    host_source = str(root / 'docs/experiments/jit-managed/host.c')
    link = [host_source if part.endswith('benchmarks/native-web/http-host.c') else part
            for part in link]
    # pathlib uses Windows separators when producing the original build command.
    link = [host_source if Path(part).name == 'http-host.c' else part for part in link]
    return native, link


def build_host(link, output, build):
    sources = [part for part in link if part.endswith('.c')]
    libraries = [part for part in link if part.lower().endswith('.lib')]
    flags = [part for part in link[1:link.index(sources[0])]
             if not part.startswith(('/Fo', '/Fe:'))]
    objects = []
    exports = set()
    for index, source in enumerate(sources):
        obj = output / f'helper-{index}.obj'
        build(['cl', *flags, '/c', source, '/Fo' + str(obj)])
        symbols = build(['dumpbin', '/symbols', obj]).stdout
        for line in symbols.splitlines():
            if 'External' in line and 'UNDEF' not in line:
                match = re.search(r'\|\s+(neoclr_[A-Za-z0-9_]+)\s*$', line)
                if match:
                    exports.add(match[1])
        objects.append(obj)
    if 'neoclr_jit_run_v1' not in exports:
        raise ValueError('JIT host entry was not emitted')
    definition = output / 'host.def'
    definition.write_text('EXPORTS\n' + '\n'.join(sorted(exports)) + '\n')
    host = output / 'host.dll'
    build(['link', '/NOLOGO', '/DLL', '/OUT:' + str(host), '/DEF:' + str(definition),
           *objects, *libraries])
    return host


def link_guest(link, old_object, obj, wrapper, exe, folder):
    command = []
    for part in link:
        if part == old_object:
            command.extend([str(obj), str(wrapper)])
        elif part.startswith('/Fe:'):
            command.append('/Fe:' + str(exe))
        elif part.startswith('/Fo'):
            command.append('/Fo' + str(folder) + '/')
        else:
            command.append(part)
    return command

"""Check native assembly-level constant emission boundaries and cross-assembly visibility."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'bundle', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    compiler, bundle, output = (getattr(args, n).resolve() for n in ('compiler', 'bundle', 'output'))
    output.mkdir(parents=True, exist_ok=False)
    libraries = [bundle / (n + '.dll') for n in ('System.Runtime', 'System.Data', 'System.Networking', 'System.Web')]
    common = ['dotnet', compiler, 'neoclr', '--core-reference', bundle / 'Core.dll',
              '--runtime-seed', bundle / 'System.runtime.neox', '--bootstrap-intrinsics',
              '--bootstrap-ownership', bundle / 'ownership.json', '--object-library', 'System.Runtime',
              *[x for lib in libraries for x in ('--reference', lib)]]
    inputs = [Path(__file__), compiler, *compiler.parent.glob('*.dll'), *libraries,
              *[bundle / n for n in ('Core.dll', 'System.runtime.neox', 'ownership.json')]]
    evidence = dict(inputs={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, commands=[])

    def run(name, source, flags, rejected=False, diagnostic=None):
        path = output / (name + '.rvn')
        path.write_text(source)
        command = list(map(str, [*common, *flags, '-o', output / (name + '.dll'), path]))
        result = subprocess.run(command, capture_output=True, text=True, timeout=60)
        evidence['commands'].append(dict(command=command, source=source, status=result.returncode,
                                         stdout=result.stdout, stderr=result.stderr))
        (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
        assert (result.returncode != 0) == rejected, evidence['commands'][-1]
        if diagnostic:
            assert diagnostic in result.stderr, evidence['commands'][-1]

    run('integer', 'namespace Constants\npublic const Answer: int = 42\n', ['--library'], True, 'NEOMETA001')
    run('nonfinite', 'namespace Constants\npublic const Infinity: double = 1.0 / 0.0\n', ['--library'], True, 'RAV0171')
    run('definitions', 'namespace Constants\ninternal const Hidden: double = 42.0\npublic const Visible: double = 42.0\n', ['--library'])
    for name, expression, rejected in [('visible', 'Constants.Visible', False),
                                       ('hidden-qualified', 'Constants.Hidden', True),
                                       ('hidden-wildcard', 'Hidden', True)]:
        run(name, 'import Constants.*\nfunc Main() -> int { if ' + expression + ' == 42.0 { return 0 }; return 1 }\n',
            ['--reference', output / 'definitions.dll'], rejected)
    for path in inputs:
        assert hashlib.sha256(path.read_bytes()).hexdigest() == evidence['inputs'][str(path)], path
    print('PASS native assembly-level constant contracts and visibility')


if __name__ == '__main__':
    main()

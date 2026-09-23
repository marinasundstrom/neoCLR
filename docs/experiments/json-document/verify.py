"""Validate the document consumer against .NET 10 and an independent JSON reader."""
import argparse
from dataclasses import dataclass
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


def run(command, **kwargs):
    result = subprocess.run(command, capture_output=True, text=True, timeout=120, **kwargs)
    if result.returncode:
        raise RuntimeError(result.stdout + result.stderr)
    return result


@dataclass
class Number:
    text: str


def value(text):
    # Keep number lexemes, including exponent spelling and negative zero.
    return json.loads(text, parse_int=Number, parse_float=Number)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--toolchain-root', type=Path, required=True)
    toolchain = parser.parse_args().toolchain_root.resolve()
    here = Path(__file__).resolve().parent
    env = dict(os.environ, NeoCLRRoot=str(toolchain), RavenSdkRoot=str(toolchain / 'raven-sdk'))
    numbers = ['0', '-0', '2147483647', '-2147483648', '2147483648', '-2147483649',
               '1.0', '1e0', '-2.50E+03', '9007199254740993', '1e400', '1e-400']
    valid = ['{}', '[]', 'null', 'true', 'false', '"café 😀é"',
             r'{"a":"\ud83d\ude00","b":[true,false,null,-1.25e+2]}',
             '{"a":1,"A":2}', r'{"a\"b":"x\\y"}', ' \r\n {"x": []}\t ',
             '[[[[0]]]]', '[' + ','.join('0' for _ in range(31)) + ']',
             '"' + 'a' * 126 + '"'] + numbers
    invalid = ['', '01', '+1', '-', '.1', '1.', '1e', '1e+', 'NaN', 'Infinity',
               '[1,]', '[,1]', '[1 2]', '{"x" 1}', '{x:1}', '{"x":}',
               '{"x":1,}', '{"x":1 "y":2}', 'true false', '/*x*/0',
               r'{"a":1,"\u0061":2}', '{"x":{"a":1,"a":2}}',
               r'{"\ud800":0}', r'["\udfff"]', '[[[[[0]]]]]',
               '[' + ','.join('0' for _ in range(32)) + ']', '"' + 'a' * 127 + '"',
               '\ufeff{}', '{"a":1]']
    corpus = valid + invalid
    with tempfile.TemporaryDirectory(prefix='neoclr-json-document-') as directory:
        root = Path(directory)
        reference = root / 'reference'
        reference.mkdir()
        (reference / 'global.json').write_text('{"sdk":{"version":"10.0.100","rollForward":"disable"}}')
        (reference / 'Reference.csproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>Exe</OutputType><TargetFramework>net10.0</TargetFramework><Nullable>enable</Nullable></PropertyGroup></Project>')
        shutil.copyfile(here / 'Reference.cs', reference / 'Program.cs')
        (reference / 'cases.json').write_text(json.dumps(corpus))
        run(['dotnet', 'build', '-nologo', '-v:q'], cwd=reference)
        compared = run(['dotnet', str(reference / 'bin/Debug/net10.0/Reference.dll'), str(reference / 'cases.json')])
        outcomes = json.loads(compared.stdout)
        assert [entry['accepted'] for entry in outcomes] == [True] * len(valid) + [False] * len(invalid), outcomes
        print(f'.NET reference: {len(corpus)} cases and duplicate-name baseline passed', flush=True)
        (root / 'json-message').mkdir()
        shutil.copyfile(here.parent / 'json-message/JsonMessage.rvn', root / 'json-message/JsonMessage.rvn')
        project = root / 'json-document'
        project.mkdir()
        for name in ['JsonDocument.rvnproj', 'JsonDocument.rvn', 'JsonValue.rvn']:
            shutil.copyfile(here / name, project / name)

        def build(name):
            shutil.copyfile(here / name, project / 'Main.rvn')
            result = run(['dotnet', 'msbuild', str(project / 'JsonDocument.rvnproj'), '-nologo', '-v:minimal'], env=env)
            assert 'warning ' not in result.stdout, result.stdout

        def execute(*arguments):
            command = [str(toolchain / 'bin/neoclr'), 'run', str(project / 'bin/neoclr/Debug/App.neoil'), '--system', str(toolchain / 'lib/System.neoil')]
            return run(command + (['--'] + list(arguments) if arguments else [])).stdout

        build('Main.rvn')
        assert execute() == (here / 'expected.txt').read_text()
        print('Sensor acknowledgement sample: passed', flush=True)
        build('Checks.rvn')
        for index, (text, outcome) in enumerate(zip(corpus, outcomes)):
            output = execute('roundtrip', text).rstrip('\n')
            if outcome['accepted']:
                assert output != 'ERROR', text
                assert value(output) == value(text), (text, output)
            else:
                assert output == 'ERROR', (text, output)
            if (index + 1) % 6 == 0 or index + 1 == len(corpus):
                print(f'Document cases {index + 1}/{len(corpus)}: passed', flush=True)
        for number in numbers:
            expected = outcomes[corpus.index(number)]['integer'] or 'ERROR'
            assert execute('integer', number) == expected + '\n', number
        print('Checked Int32 conversions: passed', flush=True)
        assert execute('contracts') == 'contracts passed\n'
        assert execute('writer') == 'writer limits passed\n'
        print('Field access, construction, cycles and output limits: passed', flush=True)


if __name__ == '__main__':
    main()

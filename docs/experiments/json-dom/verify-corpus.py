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
    parser.add_argument('--runner', type=Path, required=True)
    args = parser.parse_args()
    toolchain = args.toolchain_root.resolve()
    runner = args.runner.resolve()
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
        shutil.copyfile(here.parent / 'json-document/Reference.cs', reference / 'Program.cs')
        (reference / 'cases.json').write_text(json.dumps(corpus))
        run(['dotnet', 'build', '-nologo', '-v:q'], cwd=reference)
        compared = run(['dotnet', str(reference / 'bin/Debug/net10.0/Reference.dll'), str(reference / 'cases.json')])
        outcomes = json.loads(compared.stdout)
        assert [entry['accepted'] for entry in outcomes] == [True] * len(valid) + [False] * len(invalid), outcomes
        print(f'.NET reference: {len(corpus)} cases and duplicate-name baseline passed', flush=True)
        project = root / 'json-dom'
        project.mkdir()
        shutil.copyfile(here / 'JsonDom.rvnproj', project / 'JsonDom.rvnproj')
        shutil.copyfile(here / 'Corpus.rvn', project / 'Main.rvn')
        shutil.copyfile(here / 'Sample.rvn', project / 'Sample.rvn')
        result = run(['dotnet', 'msbuild', str(project / 'JsonDom.rvnproj'), '-nologo', '-v:minimal'], env=env)
        assert 'warning ' not in result.stdout, result.stdout

        def execute(*arguments):
            command = [str(runner), str(project / 'bin/neoclr/Debug/App.neoil'), str(toolchain / 'lib/System.neoil'), '512', '100000000']
            result = run(command + (['--'] + list(arguments) if arguments else []))
            assert 'live=0' in result.stderr, result.stderr
            return result.stdout

        # Batch inputs to avoid reloading the complete runtime for every document.
        for start in range(0, len(corpus), 8):
            batch = corpus[start:start + 8]
            outputs = execute('roundtrip', *batch).splitlines()
            assert len(outputs) == len(batch), outputs
            for text, output, outcome in zip(batch, outputs, outcomes[start:start + 8]):
                if outcome['accepted']:
                    assert output != 'ERROR', text
                    assert value(output) == value(text), (text, output)
                else:
                    assert output == 'ERROR', (text, output)
            print(f'Document cases {min(start + 8, len(corpus))}/{len(corpus)}: passed', flush=True)
        outputs = execute('integer', *numbers).splitlines()
        assert len(outputs) == len(numbers), outputs
        for number, output in zip(numbers, outputs):
            expected = outcomes[corpus.index(number)]['integer'] or 'ERROR'
            assert output == expected, (number, output, expected)
        print('Checked Int32 conversions: passed', flush=True)



if __name__ == '__main__':
    main()

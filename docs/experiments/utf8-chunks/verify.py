"""Execute bounded Raven UTF-8 cases and compare them with strict .NET 10 decoding."""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile


def run(command, **kwargs):
    result = subprocess.run(command, capture_output=True, text=True, timeout=120, **kwargs)
    if result.returncode:
        raise RuntimeError(result.stdout + result.stderr)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--toolchain-root', type=Path, required=True)
    args = parser.parse_args()
    toolchain = args.toolchain_root.resolve()
    here = Path(__file__).resolve().parent
    env = dict(os.environ, NeoCLRRoot=str(toolchain), RavenSdkRoot=str(toolchain / 'raven-sdk'))
    data = 'Aé€😀é'.encode()
    corpus = [{'hex': data.hex(), 'stops': [split]} for split in range(len(data) + 1)]
    corpus.append({'hex': data.hex(), 'stops': list(range(len(data) + 1))})
    # UTF-8 boundaries, surrogate encodings, overlong forms, stray continuations,
    # out-of-range scalars, truncated sequences and an invalid suffix after text.
    for value in ['', '00', '7f', 'c280', 'dfbf', 'e0a080', 'ed9fbf', 'ee8080', 'efbfbf',
                  'f0908080', 'f48fbfbf', 'efbbbf', '80', 'c080', 'c1bf', 'e08080',
                  'eda080', 'f0808080', 'f4908080', 'f5808080', 'ff', 'c2', 'e282',
                  'f09f98', 'e228a1', '41ff']:
        raw = bytes.fromhex(value)
        corpus.append({'hex': value, 'stops': list(range(len(raw) + 1))})
    expected = []
    for case in corpus:
        try:
            expected.append({'accepted': True, 'text': bytes.fromhex(case['hex']).decode('utf-8')})
        except UnicodeDecodeError:
            expected.append({'accepted': False, 'text': ''})
    with tempfile.TemporaryDirectory(prefix='neoclr-utf8-chunks-') as directory:
        root = Path(directory)
        reference = root / 'reference'
        reference.mkdir()
        (reference / 'global.json').write_text('{"sdk":{"version":"10.0.100","rollForward":"disable"}}')
        (reference / 'Reference.csproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>Exe</OutputType><TargetFramework>net10.0</TargetFramework></PropertyGroup></Project>')
        shutil.copyfile(here / 'Reference.cs', reference / 'Program.cs')
        (reference / 'cases.json').write_text(json.dumps(corpus))
        run(['dotnet', 'build', 'Reference.csproj', '-nologo', '-v:q'], cwd=reference)
        actual = run(['dotnet', str(reference / 'bin/Debug/net10.0/Reference.dll'), str(reference / 'cases.json')])
        assert json.loads(actual.stdout) == expected, actual.stdout
        print(f'Strict .NET 10 reference: {len(corpus)} cases passed', flush=True)
        (root / 'byte-copy').mkdir()
        shutil.copyfile(here.parent / 'byte-copy/ByteCopy.rvn', root / 'byte-copy/ByteCopy.rvn')
        project = root / 'utf8-chunks'
        project.mkdir()
        for name in ['Utf8Chunks.rvnproj', 'Utf8Chunks.rvn']:
            shutil.copyfile(here / name, project / name)

        def execute(source, output, gc=False):
            (project / 'Main.rvn').write_text(source)
            build = run(['dotnet', 'msbuild', str(project / 'Utf8Chunks.rvnproj'), '-nologo', '-v:minimal'], env=env)
            assert 'warning ' not in build.stdout, build.stdout
            result = run([str(toolchain / 'bin/neoclr'), 'run', str(project / 'bin/neoclr/Debug/App.neoil'), '--system', str(toolchain / 'lib/System.neoil'), '--gc-stats'])
            assert result.stdout == output, result.stdout + result.stderr
            if gc:
                match = re.search(r'collections=(\d+)', result.stderr)
                assert match and int(match[1]) > 0, result.stderr

        execute((here / 'Main.rvn').read_text(), (here / 'expected.txt').read_text())
        print('Teaching sample: passed', flush=True)
        execute((here / 'Checks.rvn').read_text(), 'Owned carry, unchanged state after invalid ranges and finalization: passed\nMalformed and truncated inputs stop the decoder: passed\nPending scalar retained through guest GC: passed\n', gc=True)
        print('State and GC checks: passed', flush=True)
        helpers = (here / 'Checks.rvn').read_text().split('func Main()')[0]
        for batch in range(0, len(corpus), 3):
            statements = []
            for index in range(batch, min(batch + 3, len(corpus))):
                case, outcome = corpus[index], expected[index]
                data = bytes.fromhex(case['hex'])
                values = ', '.join(f'(byte){byte}' for byte in data)
                statements += [f'let bytes{index}: byte[] = [{values}]', f'let decoder{index} = ChunkDecoder()', f'var text{index} = ""', f'var rejected{index} = false']
                start = 0
                for end, final in [(stop, False) for stop in case['stops']] + [(len(data), True)]:
                    statements.append(f'''if !rejected{index} {{
                        match decoder{index}.Decode(bytes{index}, {start}, {end - start}, {str(final).lower()}) {{
                            Ok(let part) => text{index} = String.Concat(text{index}, part)
                            Error(_) => rejected{index} = true
                        }}
                    }}''')
                    start = end
                if outcome['accepted']:
                    # Raven accepts JSON string escaping for these scalar values.
                    text = json.dumps(outcome['text'], ensure_ascii=False)
                    statements.append(f'Check(!rejected{index} && text{index}.Equals({text}))')
                else:
                    statements.append(f'Check(rejected{index})')
            execute(helpers + '\nfunc Main() {\n' + '\n'.join(statements) + '\nWriteLine("passed")\n}\n', 'passed\n')
            print(f'Decoder cases {batch + 1}–{min(batch + 3, len(corpus))}: passed', flush=True)


if __name__ == '__main__':
    main()

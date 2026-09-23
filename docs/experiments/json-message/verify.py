"""Compare the bounded JSON message codec with .NET 10 string materialization."""
import argparse
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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--toolchain-root', type=Path, required=True)
    toolchain = parser.parse_args().toolchain_root.resolve()
    here = Path(__file__).resolve().parent
    env = dict(os.environ, NeoCLRRoot=str(toolchain), RavenSdkRoot=str(toolchain / 'raven-sdk'))
    valid = ['""', '"hello"', ' \t\r\n"café 😀é"\n',
             r'"\"\\\/\b\f\n\r\t"', r'"\u0000\u007f\u0080\u07ff"',
             r'"\u0800\ud7ff\ue000\uffff"', r'"\ud800\udc00"',
             r'"\udbff\udfff"', r'"\u00e9\u00E9"', '"<>&"',
             '"' + 'a' * 126 + '"', '"' + 'é' * 63 + '"']
    invalid = ['', ' ', 'null', 'true', '0', '[]', '{}', '"unterminated',
               '"trailing" false', '"a","b"', '"a" "b"', '\ufeff"x"',
               '\v"x"', '"x"\u00a0', '"' + 'a' * 127 + '"',
               r'"\x00"', r'"\u000"', r'"\u00gg"', r'"\ud800"',
               r'"\udfff"', r'"\ud800x"', r'"\ud800\u0041"',
               r'"\ud800\ud800"', r'"\udc00\ud800"', '"abc\\']
    invalid += ['"' + chr(value) + '"' for value in range(32)]
    corpus = valid + invalid
    expected = [{'accepted': True, 'text': json.loads(value)} for value in valid]
    expected += [{'accepted': False, 'text': ''} for _ in invalid]
    helpers = '''import System.*
import System.Result.*
import System.Console.*
func Check(value: bool) {
    if !value {
        System.Fault("JSON message check failed")
    }
}
func Text(result: Result<string, string>) -> string {
    match result {
        Ok(let value) => return value
        Error(_) => System.Fault("Unexpected codec error")
    }
    return ""
}
func Rejected(result: Result<string, string>) {
    match result {
        Ok(_) => System.Fault("Expected codec rejection")
        Error(_) => { }
    }
}
'''
    with tempfile.TemporaryDirectory(prefix='neoclr-json-message-') as directory:
        root = Path(directory)
        reference = root / 'reference'
        reference.mkdir()
        (reference / 'global.json').write_text('{"sdk":{"version":"10.0.100","rollForward":"disable"}}')
        (reference / 'Reference.csproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>Exe</OutputType><TargetFramework>net10.0</TargetFramework></PropertyGroup></Project>')
        shutil.copyfile(here / 'Reference.cs', reference / 'Program.cs')
        (reference / 'cases.json').write_text(json.dumps(corpus))
        run(['dotnet', 'build', '-nologo', '-v:q'], cwd=reference)
        actual = run(['dotnet', str(reference / 'bin/Debug/net10.0/Reference.dll'), str(reference / 'cases.json')])
        assert json.loads(actual.stdout) == expected, actual.stdout
        print(f'.NET 10 comparison: {len(corpus)} cases passed', flush=True)
        project = root / 'raven'
        project.mkdir()
        for name in ['JsonMessage.rvnproj', 'JsonMessage.rvn']:
            shutil.copyfile(here / name, project / name)

        def execute(source, expected_output):
            (project / 'Main.rvn').write_text(source)
            build = run(['dotnet', 'msbuild', str(project / 'JsonMessage.rvnproj'), '-nologo', '-v:minimal'], env=env)
            assert 'warning ' not in build.stdout, build.stdout
            result = run([str(toolchain / 'bin/neoclr'), 'run', str(project / 'bin/neoclr/Debug/App.neoil'), '--system', str(toolchain / 'lib/System.neoil')])
            assert result.stdout == expected_output, result.stdout + result.stderr

        execute((here / 'Main.rvn').read_text(), (here / 'expected.txt').read_text())
        print('Message reply sample: passed', flush=True)
        # Each invocation stays within the normal guest instruction budget.
        for index, (payload, outcome) in enumerate(zip(valid, expected)):
            literal = json.dumps(payload, ensure_ascii=False)
            if outcome['accepted']:
                value = json.dumps(outcome['text'], ensure_ascii=False)
                body = f'''let message = Text(ReadMessage({literal}))
Check(message.Equals({value}))
let written = Text(WriteMessage(message))
Check(Text(ReadMessage(written)).Equals(message))'''
            else:
                body = f'Rejected(ReadMessage({literal}))'
            execute(helpers + '\nfunc Main() {\n' + body + '\nWriteLine("passed")\n}\n', 'passed\n')
            print(f'JSON case {index + 1}/{len(corpus)}: passed', flush=True)
        # Rejections are short; batch them to avoid rebuilding for every byte.
        for start in range(0, len(invalid), 6):
            body = '\n'.join(f'Rejected(ReadMessage({json.dumps(value, ensure_ascii=False)}))'
                             for value in invalid[start:start + 6])
            execute(helpers + '\nfunc Main() {\n' + body + '\nWriteLine("passed")\n}\n', 'passed\n')
            print(f'Rejection cases {start + 1}–{min(start + 6, len(invalid))}: passed', flush=True)
        # Inspect generated JSON independently, covering every control escape.
        for start in [0, 16]:
            message = ''.join(chr(value) for value in range(start, start + 16))
            encoded = '"' + ''.join('\\u%04x' % ord(value) for value in message) + '"'
            body = f'WriteLine(Text(WriteMessage({json.dumps(message)})))'
            execute(helpers + '\nfunc Main() {\n' + body + '\n}\n', encoded + '\n')
            assert json.loads(encoded) == message
        print('All control-byte writer escapes: passed', flush=True)
        # Output expansion is checked independently of unescaped input size.
        for count, accepted in [(21, True), (22, False), (127, False)]:
            value = json.dumps('\0' * count)
            body = f'Check(Text(WriteMessage({value})).GetUtf8ByteCount() == 128)' if accepted else f'Rejected(WriteMessage({value}))'
            execute(helpers + '\nfunc Main() {\n' + body + '\nWriteLine("passed")\n}\n', 'passed\n')
        print('Writer expansion and input limits: passed', flush=True)


if __name__ == '__main__':
    main()

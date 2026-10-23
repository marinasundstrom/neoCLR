#!/usr/bin/env python3
"""Check allocation search counters and replay compiled consumers with ThinLTO."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--diagnostic', type=Path, required=True)
parser.add_argument('--qualification', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parents[4]
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
report = dict(passed=False, commands=[], inputs={})


def run(command, expected=0):
    result = subprocess.run(command, capture_output=True, text=True, timeout=120)
    report['commands'].append(dict(command=command, exitCode=result.returncode,
                                   stdout=result.stdout, stderr=result.stderr))
    assert result.returncode == expected, (command, result.stderr)
    assert 'runtime error:' not in result.stderr, result.stderr
    return result


try:
    diagnostic = json.loads(args.diagnostic.read_text())
    qualification = json.loads(args.qualification.read_text())
    assert diagnostic['passed'] and qualification['passed']
    generated = args.diagnostic.resolve().with_name('diagnostic-gc.c')
    assert hashlib.sha256(generated.read_bytes()).hexdigest() == diagnostic['inputs'][str(generated)]
    for sanitized in [False, True]:
        binary = out / ('counters-' + str(sanitized))
        command = ['clang', '-O2', '-std=c11', '-Wall', '-Wextra', '-Werror',
                   '-DNEOCLR_NATIVE_GC', '-I', str(generated.parent),
                   '-I', str(root/'runtime/native/gc'),
                   '-I', str(root/'docs/experiments/aot-console'),
                   str(Path(__file__).with_name('search-counter-test.c')),
                   str(root/'docs/experiments/aot-console/root-probe.c'), '-o', str(binary)]
        if sanitized:
            command += ['-fsanitize=undefined,bounds', '-fno-sanitize-recover=all']
        run(command)
        result = run([str(binary)])
        assert result.stderr.splitlines() == ['ALLOCATION_SEARCH 5 3 1 4 240',
                                               'ALLOCATION_SEARCH 0 0 0 0 0']
    commands = qualification['commands']
    for index, entry in enumerate(commands):
        command = entry['command']
        if '-o' not in command or not any('http-host.c' in item for item in command):
            continue
        original = command[command.index('-o') + 1]
        obj = next(Path(item) for item in command if item.endswith('/app.o'))
        assert hashlib.sha256(obj.read_bytes()).hexdigest() == qualification['inputs'][str(obj)]
        binary = out / Path(original).name
        link = [str(binary) if item == original else item for item in command] + ['-flto=thin']
        run(link)
        expected = commands[index + 1]
        assert expected['command'] == [original]
        result = run([str(binary)], expected['exitCode'])
        assert result.stdout == expected['stdout']
        if expected['exitCode']:
            assert result.stderr.count('runtime async sample terminal fault') == 2
            assert 'UNREACHABLE' not in result.stdout and 'Native HTTP host failed' not in result.stderr
        else:
            assert not result.stderr
    assert len(report['commands']) == 44
    inputs = [args.diagnostic.resolve(), args.qualification.resolve(), generated,
              Path(__file__).resolve(), Path(__file__).with_name('search-counter-test.c').resolve(),
              *(root/'runtime/native').rglob('*.[ch]')]
    for command in report['commands']:
        inputs += [Path(item) for item in command['command'] if Path(item).is_absolute() and Path(item).is_file()]
    report['inputs'] = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in inputs}
    report['passed'] = True
finally:
    (out/'validation.json').write_text(json.dumps(report, indent=2)+'\n')
print('Two counter fixtures and 20 ThinLTO consumer runs / 40 invocations passed')

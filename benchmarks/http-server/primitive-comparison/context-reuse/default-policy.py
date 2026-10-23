#!/usr/bin/env python3
"""Verify default and fallback objects against the qualified opt-in implementation."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parents[4]
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
source = root / 'runtime/native/async/native-runtime.c'
reference = '5afe26b02'
old = out / 'qualified.c'
old.write_bytes(subprocess.check_output(
    ['git', 'show', reference + ':runtime/native/async/native-runtime.c'], cwd=root))
report = dict(passed=False, reference=reference, commands=[], hashes={})
common = ['clang', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror',
          '-Wno-deprecated-declarations', '-DNEOCLR_NATIVE_GC',
          '-I', str(root / 'runtime/native/async'),
          '-I', str(root / 'docs/experiments/aot-console'), '-c']
try:
    for name, path, flags in [
        ('old-enabled', old, ['-DNEOCLR_ASYNC_CONTEXT_REUSE']),
        ('old-disabled', old, []),
        ('default', source, []),
        ('enabled', source, ['-DNEOCLR_ASYNC_CONTEXT_REUSE=1']),
        ('disabled', source, ['-DNEOCLR_ASYNC_CONTEXT_REUSE=0']),
        ('invalid', source, ['-DNEOCLR_ASYNC_CONTEXT_REUSE=2']),
    ]:
        obj = out / (name + '.o')
        command = common + flags + [str(path), '-o', str(obj)]
        result = subprocess.run(command, capture_output=True, text=True, timeout=60)
        report['commands'].append(dict(command=command, exitCode=result.returncode,
                                       stderr=result.stderr))
        if name == 'invalid':
            assert result.returncode != 0 and 'must be zero or one' in result.stderr
        else:
            result.check_returncode()
            report['hashes'][name] = hashlib.sha256(obj.read_bytes()).hexdigest()
    hashes = report['hashes']
    assert hashes['default'] == hashes['enabled'] == hashes['old-enabled']
    assert hashes['disabled'] == hashes['old-disabled']
    report['inputs'] = {str(p): hashlib.sha256(p.read_bytes()).hexdigest()
                        for p in [source, old, Path(__file__).resolve()]}
    report['passed'] = True
finally:
    (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
print('Default matches qualified reuse; fallback matches previous default; invalid policy rejected')

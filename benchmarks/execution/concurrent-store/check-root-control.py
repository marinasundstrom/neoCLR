"""Require loss of compiled storage roots to fail the empty-caller-root consumer."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--validation', type=Path, required=True)
args = parser.parse_args()
out = args.validation.resolve()
report = json.loads((out / 'results.json').read_text())
assert report['passed']
source = out / 'native-gc.c'
for path in [source, Path(__file__).with_name('consumer.c')]:
    assert hashlib.sha256(path.read_bytes()).hexdigest() == report['inputs'][str(path)]
text = source.read_text()
needle = 'mark_slots(&context->text, index, f->storage, f->storage_count, pending);'
assert text.count(needle) == 1
broken = out / 'missing-storage-roots.c'
broken.write_text(text.replace(needle, 'mark_slots(&context->text, index, f->storage, 0, pending);'))
command = next(row['command'] for row in report['commands']
               if '-DMANAGED_MAIN' in row['command'] and '-DNEOCLR_GC_PACED' not in row['command'])[:]
command[command.index(str(source))] = str(broken)
exe = out / 'missing-storage-roots'
command[command.index('-o') + 1] = str(exe)
result = {'passed': False, 'compileCommand': command,
          'sourceSha256': hashlib.sha256(broken.read_bytes()).hexdigest()}
try:
    build = subprocess.run(command, capture_output=True, text=True, timeout=90)
    result['build'] = dict(exitCode=build.returncode, stdout=build.stdout, stderr=build.stderr)
    build.check_returncode()
    run = subprocess.run([str(exe)], capture_output=True, text=True, timeout=90)
    result['run'] = dict(exitCode=run.returncode, stdout=run.stdout, stderr=run.stderr)
    # Require the new empty-caller fixture to detect the loss, not a later test
    # or an unrelated sanitizer/linker failure.
    consumer = Path(__file__).with_name('consumer.c').read_text().splitlines()
    first = next(i for i, line in enumerate(consumer, 1) if 'static void compiled_argument_roots' in line)
    last = next(i for i, line in enumerate(consumer, 1) if 'int native_managed_consumer' in line)
    assert run.returncode != 0
    assert any(f'consumer line {i}\n' in run.stderr for i in range(first, last))
    result['passed'] = True
finally:
    (out / 'missing-storage-roots.json').write_text(json.dumps(result, indent=2) + '\n')

"""Execute file fixtures in isolation; check UTF-8 bytes and preflight preservation."""
import argparse
from pathlib import Path
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('probe', type=Path)
parser.add_argument('--runtime', required=True, type=Path)
parser.add_argument('--system', type=Path, help='Matching Raven target library')
args = parser.parse_args()
expected = {
    'Files': 'Written\nCompleted\nHello, värld!\nWrite too large\nHello, värld!\nRead too large\n',
    'Failures': 'Not found\nInvalid UTF-8\nInvalid limit\nWritten\nCompleted\n\n',
}
with tempfile.TemporaryDirectory(prefix='neoclr-file-check-') as temporary:
    root = Path(temporary)
    (root / 'invalid.txt').write_bytes(b'\xff')
    for name, output in expected.items():
        artifact = (args.probe / (name + '.neoil')).resolve()
        for command in ('verify', 'run'):
            run = subprocess.run([str(args.runtime.resolve()), command, str(artifact),
                                  *(['--system', str(args.system.resolve())] if args.system else [])],
                                 cwd=root, capture_output=True, text=True, timeout=60)
            if run.returncode or (command == 'run' and run.stdout != output):
                raise AssertionError((name, command, run.stdout, run.stderr))
        print(name + ': passed')
    assert (root / 'neoclr-file-demo.txt').read_bytes() == 'Hello, värld!'.encode('utf-8')
    assert (root / 'empty.txt').read_bytes() == b''
    assert (root / 'invalid.txt').read_bytes() == b'\xff'
    assert not (root / 'missing.txt').exists()
print('UTF-8 round trip, rejected-write preservation, missing/invalid input and empty files: passed')
for name in ('IgnoredExtraction', 'InvertedExtraction', 'UninitializedError'):
    assert (args.probe / (name + '.rejected.txt')).is_file()
    assert not (args.probe / (name + '.neoil')).exists()
print('Conditional-output and uninitialized-error rejections: passed')

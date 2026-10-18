#!/usr/bin/env python3
"""Relink a retained Serve object with GC timing only, preserving explicit policy."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--link', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=False)
command = json.loads(a.link.read_text())['command'][:]
command[command.index('-o')+1] = str((a.output/'app').resolve())
command += ['-DNEOCLR_GC_PROFILE', str(Path(__file__).with_suffix('.c').resolve())]
root = Path(__file__).resolve().parents[3]
inputs = {str(Path(item).resolve()) for item in command if Path(item).is_file()}
inputs.update(str(path) for path in (root/'runtime/native').rglob('*') if path.suffix in ('.c', '.h'))
result = subprocess.run(command, capture_output=True, text=True)
report = dict(command=command, exitCode=result.returncode, stderr=result.stderr,
              inputs={name: hashlib.sha256(Path(name).read_bytes()).hexdigest() for name in sorted(inputs)})
(a.output/'link.json').write_text(json.dumps(report, indent=2)+'\n')
result.check_returncode()

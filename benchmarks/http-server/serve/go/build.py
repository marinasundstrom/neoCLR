#!/usr/bin/env python3
"""Build the matched Go fixture with the installed toolchain and retain its identity."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--go', default='go')
a = p.parse_args()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
source = Path(__file__).with_name('main.go').resolve()
command = [a.go, 'build', '-trimpath', '-o', str(out/'app'), str(source)]
env = dict(os.environ, GOTOOLCHAIN='local', CGO_ENABLED='0')
result = subprocess.run(command, env=env, capture_output=True, text=True)
report = dict(passed=result.returncode == 0, runtime='go',
              version=subprocess.check_output([a.go, 'version'], text=True).strip(),
              command=command, exitCode=result.returncode, stderr=result.stderr,
              environment={k: env.get(k) for k in ['GOTOOLCHAIN','CGO_ENABLED','GOOS','GOARCH']},
              sourceSha256=hashlib.sha256(source.read_bytes()).hexdigest(), builtAtUnixSeconds=time.time())
if result.returncode == 0:
    report['executableSha256'] = hashlib.sha256((out/'app').read_bytes()).hexdigest()
(out/'build.json').write_text(json.dumps(report, indent=2)+'\n')
result.check_returncode()

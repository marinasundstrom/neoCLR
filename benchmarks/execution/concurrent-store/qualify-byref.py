"""Qualify immutable-byref roots with matched compiler controls and lost-root faults."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--runner', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
base = Path(__file__).resolve().parent
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
runner = args.runner.resolve()
report = {'passed': False, 'commands': [], 'runnerSha256': hashlib.sha256(runner.read_bytes()).hexdigest()}

def run(command, env):
    command = list(map(str, command))
    result = subprocess.run(command, env=env, capture_output=True, text=True, timeout=180)
    report['commands'].append(dict(command=command,
        environment={k:v for k,v in env.items() if k.startswith('NEOCLR_')},
        exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
    result.check_returncode()
    return result

try:
    for enabled in [False, True]:
        # Ignore ambient experiment knobs; isolate this option on the existing
        # qualified stack-extent proof, at production optimization level.
        env = {k:v for k,v in os.environ.items() if not k.startswith('NEOCLR_')}
        env.update(NEOCLR_TEST_OPT_LEVEL='speed', NEOCLR_LIVE_ELIDE_STACK_ARGUMENTS='1',
                   NEOCLR_LIVE_ELIDE_FORWARDED_STACK_ARGUMENTS='1', NEOCLR_LIVE_FORWARD_STACK_ARGUMENTS='1')
        if enabled:
            env['NEOCLR_LIVE_SSA_BYREF_ARGUMENTS'] = '1'
        name = 'on' if enabled else 'off'
        focused = run([runner, 'byref_', '--nocapture'], env)
        assert '3 passed' in focused.stdout
        validation = out / name
        run(['python3', base / 'validate.py', '--runner', runner, '--output', validation], env)
        assert json.loads((validation / 'results.json').read_text())['passed']
        run(['python3', base / 'check-root-control.py', '--validation', validation], env)
        assert json.loads((validation / 'missing-storage-roots.json').read_text())['passed']
        print(name, 'PASS', flush=True)
    report['passed'] = True
finally:
    (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')

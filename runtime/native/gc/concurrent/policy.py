"""Validate the selected owned candidate; research variants remain separate builds."""
import json
from pathlib import Path

POLICY = json.loads(Path(__file__).with_name('policy.json').read_text())


def validate_flags(command):
    expected = {value.split('=', 1)[0]: value for value in POLICY['defines']}
    found = {}
    for argument in command:
        if argument in ("-D", "-U"):
            raise ValueError("candidate settings require combined -Dname/-Uname arguments")
        if not argument.startswith(('-D', '-U')):
            continue
        value = argument[2:]
        name = value.split('=', 1)[0]
        controlled = name in expected or name.startswith((
            'NEOCLR_GC_', 'NEOCLR_LIVE_', 'NEOCLR_SHADOW_', 'NEOCLR_GATE_',
            'NEOCLR_WORKER_', 'GATE_'))
        if not controlled:
            continue
        if argument.startswith('-U') or name not in expected or value != expected[name]:
            raise ValueError(f'incompatible {POLICY["name"]} setting: {argument}')
        if name in found:
            raise ValueError(f'duplicate candidate setting: {name}')
        found[name] = value
    missing = expected.keys() - found.keys()
    if missing:
        raise ValueError(f'missing candidate settings: {sorted(missing)}')

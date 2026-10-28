#!/usr/bin/env python3
"""Development schema check; requires jsonschema, not needed by the profiler."""
import json
from pathlib import Path
from jsonschema import Draft202012Validator
from diagnostic_session import gc_session
from test_diagnostic_session import snapshot

schema = json.loads(Path(__file__).with_name('session-v1.schema.json').read_text())
Draft202012Validator.check_schema(schema)
validator = Draft202012Validator(schema)
fresh = snapshot()
absent = dict(fresh, available=False, fresh=False, sequence=0, timestampNanoseconds=0)
session = gc_session([absent, fresh, dict(fresh, fresh=False)])
validator.validate(session)
session['observations'][0]['metrics'] = {}
assert list(validator.iter_errors(session)), 'unavailable metrics must be null'
print('PASS Draft 2020-12 schema, large integer strings and availability shapes')

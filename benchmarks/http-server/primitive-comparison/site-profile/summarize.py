#!/usr/bin/env python3
"""Verify object identity by replaying compilation, then attribute publication counts."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--compilation', type=Path, required=True)
p.add_argument('--object', type=Path, required=True)
p.add_argument('--profile', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=False)
recipe = json.loads(a.compilation.read_text())
command = next(c['command'] for c in recipe['commands'] if Path(c['command'][0]).name == 'neoclr-codegen')[:]
command[4] = str((a.output / 'verified.o').resolve())
result = subprocess.run(command, capture_output=True, check=True, timeout=120)
assert Path(command[4]).read_bytes() == a.object.read_bytes(), 'Compilation does not reproduce profiled object'
metadata = json.loads(result.stdout)
profile = json.loads(a.profile.read_text())
assert profile['passed']
digest = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
assert profile['inputs'][str(a.object.resolve())] == digest(a.object)
functions = {f['compiledIndex']: f for f in metadata['functions']}
for dispatch in metadata['classVirtualDispatch']:
    if 'thunkCompiledIndex' in dispatch:
        index = dispatch['thunkCompiledIndex']
        assert index not in functions
        functions[index] = dict(name='generated class dispatch for ' + functions[dispatch['contractCompiledIndex']]['name'])
counts = Counter()
for site in profile['gc']['sites']:
    assert site['function'] in functions, 'Unmapped function; extend mapping explicitly'
    counts[site['function']] += site['count']
total = sum(counts.values())
assert total == profile['gc']['rootPublications']
responses = profile['load']['completed'] + profile['load']['drained']
# Decode bridge T_namespace_type.M_member tokens without changing raw identities.
def display(name):
    name = re.sub(r'^NeoMetadata_[0-9A-F]+\.', '', name)
    if name.startswith('T_'):
        owner, member = name[2:].split('.', 1)
        if member.startswith('M_'):
            member = bytes.fromhex(member[2:]).decode()
        return '.'.join(bytes.fromhex(s).decode() for s in owner.split('_') if s) + '.' + member
    return re.sub(r'\.M_([0-9A-F]+)$', lambda m: '.' + bytes.fromhex(m[1]).decode(), name)
summary = dict(scope='Main-owner diagnostic publication counts, not CPU attribution or capacity; startup/shutdown included.',
               compilationCommand=command, compilationStderr=result.stderr.decode(),
               compilationSha256=digest(a.compilation), objectSha256=digest(a.object),
               metadataSha256=hashlib.sha256(result.stdout).hexdigest(), profileSha256=digest(a.profile),
               responses=responses, rootPublications=total, sites=len(profile['gc']['sites']),
               functions=[dict(function=i, name=display(functions[i]['name']), rawName=functions[i]['name'],
                               count=n, perResponse=n/responses, percent=100*n/total)
                          for i,n in counts.most_common()])
(a.output/'summary.json').write_text(json.dumps(summary, indent=2)+'\n')
print(json.dumps(summary['functions'][:6], indent=2))

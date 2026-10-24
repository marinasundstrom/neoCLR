"""Resolve diagnostic store return PCs against exact AOT image symbols and selection."""
import argparse
import bisect
import collections
import hashlib
import json
from pathlib import Path
import re
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--measurement', type=Path, required=True)
p.add_argument('--selection', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
r = json.loads(a.measurement.read_text())
assert r['passed'] and r.get('diagnosticOnly')
selection = json.loads(a.selection.read_text())
rows = selection.get('commands', [])
if 'cases' in selection:
    rows = [dict(n, sample=c['sample']) for c in selection['cases'] for n in c['native']]
functions = {}
for row in rows:
    if row.get('mode') != 'live' or row['exitCode']:
        continue
    text = row['stdout']
    data = json.JSONDecoder().raw_decode(text[text.index('{'):])[0]
    selected = {f['compiledIndex']: f for f in data['functions']}
    # Synthetic box/query helpers are reported separately from source functions.
    for category, entries in data.items():
        if not isinstance(entries, list) or category.endswith('Sites'): continue
        for entry in entries:
            if isinstance(entry, dict) and 'helperCompiledIndex' in entry:
                index = entry['helperCompiledIndex']
                selected.setdefault(index, {'compiledIndex':index, 'name':'[generated '+category+']'})
    functions[row['sample']] = selected
def pretty(name):
    def decode(match):
        try: return bytes.fromhex(match[0]).decode('utf8')
        except (ValueError, UnicodeDecodeError): return match[0]
    return re.sub(r'(?<=_)[0-9A-Fa-f]{4,}(?=_|\.|$)', decode, name)
report = {'passed': False, 'scope': __doc__, 'trials': [], 'inputs': {}}
for trial in r['trials']:
    if trial['mode'] == 'sync':
        continue
    sample = trial.get('sample', r.get('sample'))
    executable = a.measurement.parent / ((sample+'-' if 'sample' in trial else '') + trial['mode'])
    nm = subprocess.run(['nm', '-n', str(executable)], capture_output=True, text=True, check=True)
    symbols = []
    image_base = None
    for line in nm.stdout.splitlines():
        parts = line.split()
        if len(parts) != 3: continue
        address = int(parts[0],16)
        if parts[2] == '__mh_execute_header': image_base = address
        if parts[1] in ['t','T']: symbols.append((address,parts[2]))
    assert image_base is not None
    symbols.sort(); addresses = [s[0] for s in symbols]
    stderr = trial['stderr'] if isinstance(trial['stderr'],str) else ''.join(trial['stderr'])
    sites = []; total = None; overflow = None
    for line in stderr.splitlines():
        fields = line.split()
        if fields and fields[0] == 'STORE_SITE_TOTAL': total,overflow=map(int,fields[1:])
        if not fields or fields[0] != 'STORE_SITE': continue
        pc = image_base + int(fields[1],16) - 1
        index = bisect.bisect_right(addresses,pc)-1
        assert index >= 0
        start,symbol = symbols[index]
        function = None
        if match := re.fullmatch(r'_neoclr_value_(\d+)',symbol):
            function = functions[sample][int(match[1])]
        name = function['name'] if function else symbol
        sites.append({'imageOffset':fields[1],'functionOffset':pc+1-start,
                      'calls':int(fields[2]),'symbol':symbol,'name':name,'display':pretty(name)})
    expected = (trial.get('STORES') or trial['storeCounts'])[0]
    assert overflow == 0 and total == expected == sum(s['calls'] for s in sites)
    grouped = collections.Counter()
    for site in sites: grouped[(site['symbol'],site['display'])] += site['calls']
    report['trials'].append({'sample':sample,'mode':trial['mode'],'total':total,
        'sites':sorted(sites,key=lambda s:-s['calls']),
        'functions':[{'symbol':key[0],'display':key[1],'calls':count,'fraction':count/total}
                     for key,count in grouped.most_common()]})
    report['inputs'][str(executable.resolve())] = hashlib.sha256(executable.read_bytes()).hexdigest()
    report['inputs'][str(executable.resolve())+':nm'] = hashlib.sha256(nm.stdout.encode()).hexdigest()
for path in [a.measurement,a.selection,Path(__file__)]:
    report['inputs'][str(path.resolve())] = hashlib.sha256(path.read_bytes()).hexdigest()
report['passed'] = bool(report['trials'])
a.output.write_text(json.dumps(report,indent=2)+'\n')

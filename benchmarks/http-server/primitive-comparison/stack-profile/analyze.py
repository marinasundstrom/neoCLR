#!/usr/bin/env python3
"""Account for every sample-tree self count; do not infer CPU time from wall samples."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import subprocess

def parse(text):
    graph = text.split('Call graph:\n',1)[1].split('\nTotal number in stack',1)[0]
    roots, stack = [], []
    for line in graph.splitlines():
        if not line.strip(): continue
        match = re.fullmatch(r'([ +!|:]*)(\d+) (.+)', line)
        if not match: raise ValueError('Unrecognized sample tree line: '+line)
        depth, count, name = len(match[1]), int(match[2]), match[3]
        node = dict(count=count, name=name, children=[])
        while stack and stack[-1][0]>=depth: stack.pop()
        if stack: stack[-1][1]['children'].append(node)
        else:
            if not name.startswith('Thread_'): raise ValueError('Missing thread root')
            roots.append(node)
        stack.append((depth,node))
    if not roots: raise ValueError('No thread samples')
    return roots

def symbol(name):
    return name.split('  (in ',1)[0]

def leaves(root):
    counts, callers = Counter(), Counter()
    def visit(node, path):
        own = node['count'] - sum(c['count'] for c in node['children'])
        if own<0: raise ValueError('Children exceed parent sample count')
        name = symbol(node['name'])
        if own:
            counts[name] += own
            caller = path[-1] if path else '(no caller)'
            callers[(name,caller)] += own
        for child in node['children']: visit(child,path+[name])
    visit(root,[])
    assert sum(counts.values()) == root['count']
    return counts,callers

def has_application(node, image):
    return node['name'].startswith(f'application  (in {image})') or any(has_application(c, image) for c in node['children'])

def display(name):
    name = re.sub(r'^NeoMetadata_[0-9A-F]+\.', '', name)
    if name.startswith('T_'):
        owner, member = name[2:].split('.',1)
        if member.startswith('M_'): member=bytes.fromhex(member[2:]).decode()
        return '.'.join(bytes.fromhex(s).decode() for s in owner.split('_') if s)+'.'+member
    return re.sub(r'\.M_([0-9A-F]+)$',lambda m:'.'+bytes.fromhex(m[1]).decode(),name)

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--capture',type=Path,required=True)
    p.add_argument('--build-report',type=Path,required=True)
    p.add_argument('--compilation',type=Path,required=True)
    p.add_argument('--object',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True)
    a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
    digest=lambda path:hashlib.sha256(Path(path).read_bytes()).hexdigest()
    capture=json.loads(a.capture.read_text());build=json.loads(a.build_report.read_text())
    assert capture['passed'] and build['passed']
    assert capture['inputs'][str(a.build_report.resolve())]==digest(a.build_report)
    assert build['inputs'][str(a.object.resolve())]==digest(a.object)
    binary = Path(next(c['command'][c['command'].index('-o')+1] for c in build['commands'] if '-o' in c['command']))
    assert capture['inputs'][str(binary)] == build['inputs'][str(binary)]
    sample=a.capture.parent/'sample.txt'
    assert digest(sample)==capture['sampleSha256']
    compilation=json.loads(a.compilation.read_text())
    command=next(c['command'] for c in compilation['commands'] if Path(c['command'][0]).name=='neoclr-codegen')[:]
    command[4]=str((a.output/'verified.o').resolve())
    result=subprocess.run(command,capture_output=True,check=True,timeout=120)
    assert Path(command[4]).read_bytes()==a.object.read_bytes(), 'Object mapping mismatch'
    metadata=json.loads(result.stdout)
    names={str(f['compiledIndex']):display(f['name']) for f in metadata['functions']}
    for dispatch in metadata['classVirtualDispatch']:
        if 'thunkCompiledIndex' in dispatch:
            names[str(dispatch['thunkCompiledIndex'])]='generated class dispatch: '+names[str(dispatch['contractCompiledIndex'])]
    def mapped(name):
        match=re.fullmatch(r'neoclr_value_(\d+)',name)
        if match:
            if match[1] not in names: raise ValueError('Unmapped generated function '+name)
            return names[match[1]]
        return name
    roots=parse(sample.read_text())
    # Only qualify this fixture when thread identities are unambiguous. Do not
    # silently classify arbitrary runtime/helper threads as HTTP workers.
    mains=[r for r in roots if 'com.apple.main-thread' in r['name']]
    workers=[r for r in roots if r not in mains]
    assert len(mains)==1 and len(workers)==capture['workers']
    combined,callers=Counter(),Counter();thread_results=[]
    for root in roots:
        counts, parents=leaves(root)
        role='coordinator' if root in mains else 'HTTP worker'
        if role=='HTTP worker':
            assert has_application(root, binary.name), 'Worker identity is not established by the fixture stack'
            combined.update(counts);callers.update(parents)
        thread_results.append(dict(thread=root['name'],role=role,samples=root['count'],
            selfSamples=[dict(symbol=s,name=mapped(s),count=n) for s,n in counts.most_common()]))
    total=sum(combined.values())
    summary=dict(scope=capture['scope'],objectSha256=digest(a.object),sampleSha256=digest(sample),
        captureSha256=digest(a.capture),compilationSha256=digest(a.compilation),
        mappingMetadataSha256=hashlib.sha256(result.stdout).hexdigest(),mappingCommand=command,
        workerSamples=total,threads=thread_results,
        workerSelfSamples=[dict(symbol=s,name=mapped(s),count=n,percent=100*n/total) for s,n in combined.most_common()],
        workerSelfSamplesByImmediateCaller=[dict(symbol=s,name=mapped(s),caller=c,callerName=mapped(c),count=n) for (s,c),n in callers.most_common()])
    (a.output/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    print(json.dumps(summary['workerSelfSamples'][:12],indent=2))
if __name__=='__main__':main()

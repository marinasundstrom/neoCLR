#!/usr/bin/env python3
"""Resolve only addresses within explicit JIT function ranges from the same process."""
import argparse
import gzip
from bisect import bisect_right
from collections import Counter
import json
from pathlib import Path
import re


class Symbols:
    def __init__(self, text):
        self.ranges=sorted((int(a,16),int(n,16),name) for a,n,name in
                           (line.split(maxsplit=2) for line in text.splitlines() if line.strip()))
        for i,(start,size,name) in enumerate(self.ranges):
            if size <= 0 or (i and self.ranges[i-1][0]+self.ranges[i-1][1]>start):
                raise ValueError('invalid or overlapping function ranges')
        self.starts=[x[0] for x in self.ranges]

    def find(self, address):
        i=bisect_right(self.starts,address)-1
        if i<0:return None
        start,size,name=self.ranges[i]
        return (name,address-start) if address-start<size else None

    def layout(self):
        pages=set()
        for start,size,_ in self.ranges:
            pages.update(range(start//16384,(start+size-1)//16384+1))
        return dict(functions=len(self.ranges),codeBytes=sum(n for _,n,_ in self.ranges),
                    addressSpan=self.ranges[-1][0]+self.ranges[-1][1]-self.ranges[0][0],
                    codePages16KiB=len(pages))


def symbolize(text,symbols,names):
    resolved=0;unknown=0
    pattern=re.compile(r'\?\?\?  \(in <unknown binary>\)  \[0x([0-9a-fA-F]+)\]')
    def replace(match):
        nonlocal resolved,unknown
        found=symbols.find(int(match[1],16))
        if not found:
            unknown+=1;return match[0]
        resolved+=1;name,offset=found
        return f'{names.get(name,name)} [{name}] (in JIT) + {offset} [0x{match[1]}]'
    return pattern.sub(replace,text),dict(resolvedOccurrences=resolved,unresolvedOccurrences=unknown)


def self_samples(text):
    # sample's indentation identifies parent/child frames; self count is the
    # node count minus its immediate children, never its whole descendant tree.
    tree=text.split('Call graph:',1)[1].split('Total number in stack',1)[0]
    stack=[];counts=Counter()
    def finish():
        depth,count,children,name=stack.pop()
        if children>count:
            raise ValueError('child samples exceed parent')
        counts[name]+=count-children
    for line in tree.splitlines():
        m=re.match(r'^([ +!:|]*)([0-9]+) (.+)$',line)
        if not m:continue
        depth=len(m[1]);count=int(m[2]);name=m[3].split('  (in ',1)[0].split(' (in ',1)[0]
        while stack and stack[-1][0]>=depth:finish()
        if stack:stack[-1][2]+=count
        stack.append([depth,count,0,name])
    while stack:finish()
    return dict(counts.most_common())


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('directory',type=Path);p.add_argument('inspection',type=Path)
    a=p.parse_args()
    inspection=json.loads(gzip.decompress(a.inspection.read_bytes()) if a.inspection.suffix=='.gz' else a.inspection.read_bytes())
    names={f"neoclr_value_{f['compiledIndex']}":f['name'] for f in inspection['functions']}
    result={}
    for label in ['system','arena']:
        symbols=Symbols((a.directory/f'{label}.map').read_text())
        text,counts=symbolize((a.directory/f'{label}.sample.txt').read_text(),symbols,names)
        (a.directory/f'{label}.symbolized.txt').write_text(text)
        result[label]=dict(**symbols.layout(),**counts,selfSamples=self_samples(text))
    (a.directory/'symbols.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))

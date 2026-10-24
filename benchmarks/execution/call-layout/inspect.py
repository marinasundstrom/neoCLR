#!/usr/bin/env python3
"""Count final generated-function call forms, excluding native helper code."""
import json
from pathlib import Path
import re
import subprocess
import sys

out = Path(sys.argv[1])
result = {}
for label in ['baseline', 'long-range', 'dynamic']:
    assembly = subprocess.check_output(['xcrun', 'llvm-objdump', '-d', str(out/label)], text=True)
    chunks = re.split(r'(?m)^([0-9a-f]+ <[^>]+>:)\n', assembly)
    functions = [chunks[i]+'\n'+chunks[i+1] for i in range(1, len(chunks)-1, 2)
                 if '<_neoclr_value_' in chunks[i]]
    selected = ''.join(functions)
    result[label] = {op: len(re.findall(r'\s'+op+r'\s', selected))
                     for op in ['bl', 'blr', 'adrp', 'ldr']}
    result[label]['functions'] = len(functions)
    (out/(label+'-excerpt.txt')).write_text('\n'.join(selected.splitlines()[:90])+'\n')
(out/'instructions.json').write_text(json.dumps(result, indent=2)+'\n')

#!/usr/bin/env python3
"""Relink identical qualified HTTP guest objects with GC counters and sample native stacks."""
import argparse
import hashlib
import json
import os
import re
from pathlib import Path
import subprocess
import time
import benchmark

ROOT = Path(__file__).resolve().parents[2]
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()

def describe_sites(sites, build, totals):
    compilation = next(json.loads(c['stdout']) for c in build['commands'] if c.get('phase') == 'native-compilation')
    names = {f['compiledIndex']: f['name'] for f in compilation['functions']}
    for kind, key in [('Int32', 'int32Boxes'), ('Int64', 'int64Boxes'), ('Boolean', 'booleanBoxes'), ('record', 'emptyRecordBoxes')]:
        for helper in compilation.get(key, []):
            names.setdefault(helper['helperCompiledIndex'], '<generated ' + kind + ' boxing helper for type ' + str(helper['typeCompiledIndex']) + '>')
    def decode(name):
        def unhex(value):
            try: return bytes.fromhex(value).decode('utf-8')
            except (ValueError, UnicodeDecodeError): return value
        name = re.sub(r'\bT_([0-9A-F]*)_([0-9A-F]+)(?=[.]|$)',
            lambda m: 'T_' + unhex(m[1]) + '_' + unhex(m[2]), name)
        return re.sub(r'\b([MF])_([0-9A-F]+)(?=[.]|$)',
            lambda m: m[1] + '_' + unhex(m[2]), name)
    for site in sites:
        index = site['function']
        site['mapped'] = index in names
        site['name'] = names.get(index, '<no managed frame>' if index == -1 else '<unmapped compiled function ' + str(index) + '>')
        site['displayName'] = decode(site['name'])
    for field, total in [('blocks', 'allocations'), ('bytes', 'allocatedBytes'), ('textBlocks', 'textBlocks'), ('textBytes', 'textBytes')]:
        if sum(s['exclusive'][field] for s in sites) != totals[total]:
            raise ValueError('Allocation attribution does not reconcile: ' + field)
    return dict(scope='24 requests between close boundaries 8 and 32. Exclusive rows sum to totals; inclusive rows overlap and count recursive functions once per allocation. Native allocator bytes exclude headers, native resources and retained memory. Published logical frames only; no causal async ancestry is inferred.', sites=sites)

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--build',type=Path,required=True)
    p.add_argument('--workload',choices=['plain','json'],required=True)
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--allocation-sites', action='store_true', help='Attribute measured allocations to logical managed frames')
    a=p.parse_args();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
    build=json.loads(a.build.read_text())
    cmd=next(c['command'][:] for c in build['commands'] if c.get('phase')=='host-compile-link')
    cmd[cmd.index('-o')+1]=str(out/'app')
    cmd+=['-g','-DNEOCLR_GC_PROFILE',str(Path(__file__).with_name('profile-gc.c').resolve())]
    if a.allocation_sites:
        cmd += ['-DNEOCLR_ALLOCATION_SITES', str(Path(__file__).with_name('allocation-sites.c').resolve())]
    inputs={str(Path(v).resolve()):sha(v) for v in cmd if Path(v).is_file()}
    inputs[str(a.build.resolve())]=sha(a.build)
    inputs[str(Path(__file__).resolve())]=sha(__file__)
    # Diagnostic-only adapters keep the guest and production runtime unchanged.
    socket_source=ROOT/'docs/experiments/aot-console/socket-listener.c'
    original=socket_source.read_text()
    hook='        neoclr_os_close(scope->slots[i].descriptor);'
    assert original.count(hook) == 1
    socket_copy=out/'profile-socket.c'
    socket_copy.write_text('extern void neoclr_http_profile_request(void);\n'+original.replace(hook,
        '        if (!scope->slots[i].listener) neoclr_http_profile_request();\n'+hook))
    collector=ROOT/'docs/experiments/aot-console/native-gc.c'
    original=(ROOT/'runtime/native/gc/native-gc.c').read_text();hook='    statistics.allocations++;'
    assert original.count(hook) == 1
    collector_copy=out/'profile-collector.c'
    collector_copy.write_text('#include <stdint.h>\nextern void neoclr_http_profile_allocated(uint32_t, uint64_t);\n'+original.replace(hook,
        hook+'\n    neoclr_http_profile_allocated(kind, bytes);'))
    cmd=[str(socket_copy) if v==str(socket_source) else str(collector_copy) if v==str(collector) else v for v in cmd]
    for path in [socket_copy,collector_copy,*sorted((ROOT/'runtime/native').rglob('*.[ch]'))]: inputs[str(path)]=sha(path)
    for directory in ['aot-console','aot-fault-details','runtime-async']:
        for path in (ROOT/'docs/experiments'/directory).glob('*.h'): inputs[str(path)]=sha(path)
    report=dict(passed=False,workload=a.workload,command=cmd,inputs=inputs,
      scope='Diagnostic whole-process counters include startup, all 32 requests and teardown. Timings are instrumented, not performance evidence.')
    try:
        subprocess.run(cmd,check=True,capture_output=True,text=True)
        original=benchmark.workbench.Guest
        class SampledGuest(original):
            def __init__(self, command, *args, **kwargs):
                super().__init__(command,*args,**kwargs)
                self.sample=subprocess.Popen(['sample',str(self.process.pid),'1','1','-file',str(out/'stacks.txt')],stdout=subprocess.PIPE,stderr=subprocess.PIPE)
            def close(self):
                super().close()
                stdout,stderr=self.sample.communicate(timeout=30)
                report['sample']={'exitCode':self.sample.returncode,'diagnostic':(stdout+stderr).decode()}
        benchmark.workbench.Guest=SampledGuest
        started=time.perf_counter()
        report['requests']=benchmark.server_sample(['/usr/bin/env','NEOCLR_HTTP_PROFILE_OUTPUT='+str(out/'gc.json'),str(out/'app')],a.workload)
        report['processAndSamplerSeconds']=time.perf_counter()-started
        counters=[json.loads(line) for line in (out/'gc.json').read_text().splitlines()]
        report['gc'],report['requestAllocations']=counters[:2]
        if a.allocation_sites:
            report['allocationSites'] = describe_sites(counters[2]['sites'], build, report['requestAllocations'])
        assert report['requestAllocations']['requests']==32
        report['requestAllocations']['scope']='Close boundaries 8 to 32: 24 requests; textBlocks means native GC TEXT storage, not all String objects.'
        assert not report['gc']['failures']
        for path,digest in inputs.items(): assert sha(path)==digest,path
        report['outputs']={name:sha(out/name) for name in ['app','gc.json','stacks.txt']}
        report['passed']=True
    finally: (out/'profile.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report['gc'],indent=2))
if __name__=='__main__': main()

"""Refresh the current website tables/chart from a validated benchmark run."""
import argparse,json
from pathlib import Path
from html import escape
p=argparse.ArgumentParser(description=__doc__);p.add_argument('run',type=Path);p.add_argument('--date',required=True);a=p.parse_args()
r=json.loads(a.run.read_text());assert r['passed'] and r['kind']=='benchmark' and r['suite']=='all'
root=Path(__file__).resolve().parents[2]
rows={(x['sample'],x['collector']):x for x in r['rows']}
labels={'compute':'Compute only','reuse-buffer':'Reusable buffer','steady':'Steady allocation','bursts':'Allocation bursts','retained-window':'Retained window','mixed-phases':'Mixed phases'}
text=f'''## Current collector comparison — {a.date}

**Development candidate, {r['hardware']['cpu']}, {r['hardware']['logicalCpus']} logical CPUs,
{r['hardware']['memoryBytes']//(1024**3)} GiB RAM, macOS ARM64.** Our intention is to make concurrent GC the
general-purpose default, with better overall application performance. Sync remains
the current default and a useful option for small, resource-constrained devices.
These measurements do not yet establish a general concurrent advantage.

Each collector keeps its own result history. Cross-collector runs use the same
application inputs, while allowing different policies. Here sync uses allocation /
publication budgets 256/4096; concurrent uses 1024/16384 and a 50% mutation-log
watermark. Both use a 1 MiB arena; the concurrent log has 4096 words.

### Plain HTTP and HTTP JSON

{r['rounds']} alternating rounds, four clients, {r['requests']:,} measured requests plus 16 warmups
per run, HTTP loopback with connection close and checked response bodies/EOF.
CPU includes the server process and collector worker through shutdown. These are
AOT experiments, separate from published-package and cross-framework results.

| Sample | Collector | Median requests/s | Process CPU µs/response | Peak arena bytes |
| --- | --- | ---: | ---: | ---: |
'''
for sample,label in [('http-plain','Plain text'),('http-json','JSON')]:
 for collector in ['sync','concurrent']:
  x=rows[sample,collector]
  text+=f"| {label} | {collector.title()} | {x['unitsPerSecond']:,.0f} | {x['cpuSeconds']/(r['requests']+16)*1e6:.1f} | {x['peakArenaBytes']:,.0f} |\n"
text+=f'''
Concurrent-specific tuning previously improved plain/JSON throughput by about
13%/17% with lower CPU, at the cost of more arena space. Log-pressure completion
removes observed store cancellations. These are gains against earlier concurrent
settings; sync remains ahead in the paired table above.

### Applications with different allocation patterns

The same non-HTTP sources run under both collectors. Each prints a checked checksum.
Different rows have different work counts: compare collectors **within a row**.
Invocation CPU excludes process startup and compilation, includes cleanup and
collector threads. {r['rounds']} alternating rounds; allocation counts match across modes.

| Sample | Managed allocations | Sync CPU ms | Concurrent CPU ms | Concurrent / sync CPU |
| --- | ---: | ---: | ---: | ---: |
'''
ratios=[]
for name,label in labels.items():
 s=rows[name,'sync'];c=rows[name,'concurrent'];assert s['allocations']==c['allocations'];ratio=c['cpuSeconds']/s['cpuSeconds'];ratios.append((label,ratio))
 text+=f"| {label} | {s['allocations']:,.0f} | {s['cpuSeconds']*1000:.2f} | {c['cpuSeconds']*1000:.2f} | {ratio:.2f}× |\n"
text+='''
![Concurrent CPU relative to sync for six non-HTTP samples; numerical values are in the table.](/gc-allocation-cpu.svg)

The reusable-buffer case allocates only 14 managed objects and completes no
concurrent collection cycle, yet its CPU cost is much higher. That points toward
runtime store/barrier work rather than repeated collection. Compute-only work is
approximately equal. Allocation-heavy and retained-object cases still cost more
with concurrent GC. These cases guide the next improvements; the default will not
change merely because HTTP tuning improved.

Benchmark and profile runs remain separate. Diagnostic runs capture owner pauses,
begin/capture, marker, finish and collection-phase costs. No instrumented timing is
used as a throughput improvement. The suite records build inputs and policies;
self-comparison rejects mismatched workloads, hardware or diagnostic runs.

[Standard samples and procedure](/benchmarks/gc/procedure/) ·
[Raw reports and interpretation](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/gc-suite/results-20261025) ·
[Concurrent tuning and worker evidence](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/execution/concurrent-tuning/combined.md)

'''
p=root/'website/content/benchmarks/gc/index.md';s=p.read_text();start='<!-- GC_SUITE_RESULTS_START -->';end='<!-- GC_SUITE_RESULTS_END -->'
if start not in s:
 anchor='## Direct leaf marking';s=s.replace(anchor,start+'\n'+end+'\n\n'+anchor,1)
a0=s.index(start)+len(start);a1=s.index(end,a0);s=s[:a0]+'\n\n'+text+s[a1:];p.write_text(s)
max_x=max(6.5,max(v for _,v in ratios)*1.1);left=175;scale=470/max_x
svg=['<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 720 300" role="img" aria-labelledby="title desc">','<title id="title">Concurrent CPU relative to synchronous GC</title>','<desc id="desc">Lower is better. Synchronous CPU is one for every sample. '+escape('; '.join(f'{n}: {v:.2f} times' for n,v in ratios))+'.</desc>','<rect width="720" height="300" fill="white"/>','<g font-family="system-ui,sans-serif" font-size="13" fill="#172554">','<text x="16" y="24" font-size="17">Concurrent CPU relative to sync — lower is better</text>']
for i,(label,value) in enumerate(ratios):
 y=52+i*33;svg += [f'<text x="16" y="{y+14}">{escape(label)}</text>',f'<rect x="{left}" y="{y}" width="{value*scale:.1f}" height="21" rx="3" fill="#2563eb"/>',f'<text x="{left+value*scale+8:.1f}" y="{y+15}">{value:.2f}×</text>']
svg += [f'<path d="M {left+scale:.1f} 43 V 252" stroke="#64748b" stroke-dasharray="4 3"/>',f'<text x="{left+scale-25:.1f}" y="273">sync 1×</text>','</g></svg>']
(root/'website/gc-allocation-cpu.svg').write_text('\n'.join(svg)+'\n')

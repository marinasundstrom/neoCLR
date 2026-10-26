"""Render measured load curves; ranges are observed rounds, not confidence intervals."""
import argparse
import json
from pathlib import Path

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--results', type=Path, required=True, help='Directory containing complex-load and service-load summaries')
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=True)
modes = [('arena-eager-sync-jit', 'Eager / sync', '#2467aa', '--'),
         ('arena-control-sync-jit', 'Lazy / sync', '#2467aa', '-'),
         ('arena-eager-live-jit', 'Eager / concurrent', '#c85d20', '--'),
         ('arena-control-live-jit', 'Lazy / concurrent', '#c85d20', '-')]
plt.rcParams.update({'font.size': 10, 'axes.spines.top': False, 'axes.spines.right': False})
fig, axes = plt.subplots(2, 2, figsize=(11, 8), constrained_layout=True)
for row, (case, title) in enumerate([('complex-load', 'Complex handler'), ('service-load', 'Complex handler + 20 ms dependency')]):
    data = json.loads((a.results/case/'summary.json').read_text())
    assert data['completed'] and data['backendOptimization'] == 'cranelift-default'
    for col, apps in enumerate([1, 2]):
        ax = axes[row, col]
        selected = [r for r in data['rows'] if r['apps'] == apps]
        rates = sorted({r['rate'] for r in selected})
        ax.plot(rates, rates, color='#999999', linestyle=':', linewidth=1, label='Offered rate')
        for mode, label, color, style in modes:
            rows = sorted((r for r in selected if r['mode'] == mode), key=lambda r: r['rate'])
            x = [r['rate'] for r in rows]
            d = [r['metrics']['completionRate'] for r in rows]
            y = [v['median'] for v in d]
            ax.errorbar(x, y, yerr=[[v['median']-v['minimum'] for v in d], [v['maximum']-v['median'] for v in d]],
                        color=color, linestyle=style, marker='o', capsize=4, label=label)
        ax.set_title(f'{title}\n{apps} server process' + ('es' if apps > 1 else ''))
        ax.set_xlabel('Aggregate offered requests/s')
        ax.set_ylabel('Successful completions/s inside window')
        ax.set_ylim(bottom=0)
        ax.grid(alpha=.15)
axes[0, 0].legend(fontsize=8)
fig.suptitle('Production-default codegen · matched arena placement · two observed rounds\nApple M1 shared host; bounded loopback clients, not a capacity estimate', fontsize=12)
fig.savefig(a.output/'load-curves.png', dpi=160)
fig.savefig(a.output/'load-curves.svg')
plt.close(fig)

cold = json.loads((a.results/'cold-load/summary.json').read_text())
fig, axes = plt.subplots(1, 2, figsize=(11, 4), constrained_layout=True)
for mode, label, color, style in modes:
    rows = [r for r in cold['trials'] if r['mode'] == mode]
    length = len(rows[0]['perApp'][0]['timeline'])
    for ax, metric, title in zip(axes, ['completed', 'outstandingAtEnd'], ['Successful completions per second', 'Outstanding client requests at second end']):
        samples = [[sum(app['timeline'][i][metric] for app in r['perApp']) for i in range(length)] for r in rows]
        low = [min(v[i] for v in samples) for i in range(length)]
        high = [max(v[i] for v in samples) for i in range(length)]
        middle = [(lo+hi)/2 for lo,hi in zip(low,high)]
        if metric == 'completed':
            x = list(range(length+1))
            ax.step(x, middle+[middle[-1]], where='post', color=color, linestyle=style, label=label)
            ax.fill_between(x, low+[low[-1]], high+[high[-1]], step='post', color=color, alpha=.12)
        else:
            x = list(range(1, length+1))
            ax.plot(x, middle, color=color, linestyle=style, label=label)
            ax.fill_between(x, low, high, color=color, alpha=.12)
        ax.set_title(title)
        ax.set_xlabel('Second after traffic begins (no warmup)')
        ax.grid(alpha=.15)
for ax in axes:
    ax.set_ylim(bottom=0)
axes[0].legend(fontsize=8)
fig.suptitle('Cold complex-handler traffic · 150 offered requests/s · two-round ranges\nOutstanding client requests are not an internal server-queue measurement', fontsize=12)
fig.savefig(a.output/'cold-traffic.png',dpi=160)
fig.savefig(a.output/'cold-traffic.svg')

# Matplotlib emits trailing spaces inside SVG path data; normalize for source control.
for path in a.output.glob("*.svg"):
    path.write_text("\n".join(line.rstrip() for line in path.read_text().splitlines()) + "\n")

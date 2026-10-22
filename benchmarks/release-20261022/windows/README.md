# Windows development measurements — 2026-10-22

Windows x64 hosted runner, Windows Server build 26100, AMD64 Family 25 Model 1.
All five aggregate stages pass: 55 CPU/JSON processes and 1,152 HTTP responses.
This series uses the package qualified by run 39841298887 (original package run
39835531991), with benchmark source `69441f2fd`. It is separate from macOS and
published Preview 16 measurements; host differences prevent platform rankings.

Five rounds rotate modes. CPU/JSON uses 16 operations and two warmups; native CPU
uses a separate 10,000-operation, 20-warmup series. HTTP validates 32 responses per
process, discards the first eight from timing and uses one whole-process warmup
per mode. Requests are sequential loopback HTTP/1.1 with connection close. Startup
is separate from throughput and includes JIT compilation. These results do not
measure peak concurrency, allocation/GC rates, HTTPS throughput or production load.

The aggregate reports a dirty checkout. This is retained, not relabeled clean:
an audit of recorded source hashes matches 85 tracked inputs at the recorded
revision (exact bytes or Windows CRLF checkout). Two generated native archives
are hashed separately and have no tracked-source equivalent. The raw report does
not retain the dirty path list, so its cause is not established. Every measurement's
input-integrity and correctness checks pass; no controlled speedup is claimed.

## CPU and JSON

| Workload | Mode | Median operations/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| cpu.sum | interpreter | 5.08 | 4.48–5.34 | 7.527 |
| cpu.sum | jit | 42245.41 | 41560.15–42727.94 | 6.219 |
| cpu.sum | aot | 43193.06 | 43100.68–43290.72 | 0.012 |
| json.serialize | interpreter | 9.60 | 8.22–9.98 | 7.634 |
| json.serialize | jit | 281.55 | 198.75–287.40 | 6.376 |
| json.serialize | aot | 251.35 | 195.55–285.68 | 0.013 |
| json.deserialize | interpreter | 7.47 | 6.74–7.77 | 7.685 |
| json.deserialize | jit | 205.44 | 136.87–213.24 | 6.765 |
| json.deserialize | aot | 199.89 | 184.79–216.20 | 0.014 |

## HTTP

| Workload | Mode | Median requests/s | Range | Median startup s |
| --- | --- | ---: | ---: | ---: |
| http-plain | interpreter | 2.45 | 1.85–2.53 | 7.299 |
| http-plain | jit | 42.76 | 40.95–57.37 | 12.268 |
| http-plain | aot | 40.04 | 37.79–58.89 | 0.010 |
| http-json | interpreter | 1.27 | 1.25–1.29 | 7.143 |
| http-json | jit | 28.87 | 26.17–29.75 | 19.274 |
| http-json | aot | 28.18 | 26.19–28.43 | 0.009 |

## Provenance

[Raw-report hashes and source audit](summary.json) retain archive identity,
source revisions and the dirty flag. [Run 39841353425](https://github.com/marinasundstrom/neoCLR/actions/runs/39841353425)
uses the qualified package manifest `2da664e5cc96900b6f1d7b66e7b2ab0e62a7d1d09472b613e83c7c31b8052f0d`.
The [macOS series](../macos-complete/README.md) and [published Windows series](../../release-20261018/windows/README.md) remain separate.

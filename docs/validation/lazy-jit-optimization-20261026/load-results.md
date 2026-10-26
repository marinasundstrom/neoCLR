# Production-default offered-load results — 2026-10-26

All configurations use matched arena placement; eager arena is an explicit control,
while lazy arena is the selected macOS ARM64 policy. Production codegen defaults,
Apple M1, 16 GiB RAM, same-host client/dependency and ordinary desktop activity.
Two observed rounds per point; ranges are not confidence intervals. All p99 values
describe successful requests, including dispatch lag. Missed arrivals are never
successes. CPU brackets traffic plus drain; RSS is the aggregate after-traffic
snapshot, not a peak or managed-heap measurement.

The provisional target requires every offered arrival to succeed, every app p99
at most 100 ms and generator p99 lag at most 5 ms in **both rounds**. No passing point
is a measured maximum capacity. High generator lag limits attribution. See the
[main report](README.md) for methodology, configuration and host-activity evidence.

## Complex handlers

10-second windows; 20 ms header gap; 0 ms dependency delay; 16 warmup requests per app; 32 client slots per app. Reused processes across rates: True.

| Mode / apps | Offered/s | Completion/s median [range] | Success % | Worst-app p99 ms | Generator p99 ms | CPU s | RSS MiB | Target |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| lazy / concurrent / 1 | 150 | 149.7 [149.7–149.7] | 100.00 | 54.8 | 6.2 | 7.08 | 811.9 | fail |
| lazy / sync / 1 | 150 | 149.7 [149.7–149.7] | 100.00 | 55.9 | 9.3 | 4.89 | 732.8 | fail |
| eager / concurrent / 1 | 150 | 149.7 [149.7–149.7] | 100.00 | 58.6 | 5.8 | 7.14 | 533.4 | fail |
| eager / sync / 1 | 150 | 149.6 [149.6–149.7] | 100.00 | 56.3 | 6.8 | 4.88 | 823.6 | fail |
| lazy / concurrent / 1 | 300 | 299.3 [299.2–299.4] | 99.97 | 40.5 | 0.5 | 8.92 | 755.0 | fail |
| lazy / sync / 1 | 300 | 298.5 [298.3–298.6] | 99.70 | 86.5 | 5.1 | 7.45 | 557.3 | fail |
| eager / concurrent / 1 | 300 | 297.9 [296.9–299.0] | 99.52 | 108.3 | 24.3 | 8.76 | 660.6 | fail |
| eager / sync / 1 | 300 | 298.2 [297.5–299.0] | 99.62 | 86.4 | 18.3 | 7.08 | 812.4 | fail |
| lazy / concurrent / 1 | 600 | 579.2 [576.2–582.1] | 96.79 | 100.6 | 14.5 | 10.44 | 703.4 | fail |
| lazy / sync / 1 | 600 | 526.3 [505.4–547.2] | 88.23 | 111.1 | 20.1 | 9.75 | 568.5 | fail |
| eager / concurrent / 1 | 600 | 574.2 [573.4–575.1] | 95.92 | 125.5 | 50.0 | 10.58 | 782.8 | fail |
| eager / sync / 1 | 600 | 574.8 [567.7–581.8] | 96.11 | 116.8 | 23.5 | 9.66 | 805.9 | fail |
| lazy / concurrent / 2 | 150 | 149.8 [149.8–149.8] | 100.00 | 39.0 | 4.2 | 12.54 | 1650.4 | fail |
| lazy / sync / 2 | 150 | 149.8 [149.8–149.8] | 100.00 | 141.3 | 26.6 | 7.69 | 1112.5 | fail |
| eager / concurrent / 2 | 150 | 149.8 [149.8–149.8] | 100.00 | 26.4 | 0.7 | 12.24 | 1531.3 | pass |
| eager / sync / 2 | 150 | 149.8 [149.8–149.8] | 100.00 | 105.4 | 38.0 | 7.94 | 1405.0 | fail |
| lazy / concurrent / 2 | 300 | 299.4 [299.4–299.4] | 100.00 | 84.7 | 19.0 | 14.46 | 1645.5 | fail |
| lazy / sync / 2 | 300 | 299.4 [299.4–299.4] | 100.00 | 24.5 | 0.6 | 9.78 | 1162.6 | pass |
| eager / concurrent / 2 | 300 | 299.4 [299.4–299.4] | 100.00 | 24.5 | 0.7 | 14.13 | 1677.9 | pass |
| eager / sync / 2 | 300 | 299.4 [299.4–299.4] | 100.00 | 34.8 | 1.0 | 9.68 | 1401.2 | pass |
| lazy / concurrent / 2 | 600 | 593.7 [592.3–595.1] | 99.19 | 150.6 | 19.5 | 18.51 | 1714.8 | fail |
| lazy / sync / 2 | 600 | 593.4 [592.5–594.3] | 99.10 | 164.6 | 20.2 | 14.29 | 1201.5 | fail |
| eager / concurrent / 2 | 600 | 596.3 [593.8–598.8] | 99.58 | 81.3 | 21.6 | 17.66 | 1732.8 | fail |
| eager / sync / 2 | 600 | 593.4 [590.6–596.1] | 99.09 | 116.6 | 43.5 | 13.99 | 1424.9 | fail |

Full round/per-app data and per-second timelines are retained in the corresponding compressed results and summaries.

## Awaited dependency

10-second windows; 20 ms header gap; 20 ms dependency delay; 16 warmup requests per app; 32 client slots per app. Reused processes across rates: True.

| Mode / apps | Offered/s | Completion/s median [range] | Success % | Worst-app p99 ms | Generator p99 ms | CPU s | RSS MiB | Target |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| lazy / concurrent / 1 | 100 | 99.5 [99.5–99.6] | 100.00 | 47.8 | 0.9 | 7.39 | 826.7 | pass |
| lazy / sync / 1 | 100 | 99.2 [98.7–99.6] | 100.00 | 95.9 | 16.3 | 4.76 | 678.8 | fail |
| eager / concurrent / 1 | 100 | 99.6 [99.6–99.6] | 100.00 | 84.8 | 19.5 | 7.26 | 828.9 | fail |
| eager / sync / 1 | 100 | 99.6 [99.6–99.6] | 100.00 | 115.4 | 22.6 | 4.53 | 752.3 | fail |
| lazy / concurrent / 1 | 200 | 197.3 [197.0–197.6] | 99.05 | 186.9 | 27.2 | 9.29 | 871.4 | fail |
| lazy / sync / 1 | 200 | 192.1 [184.9–199.2] | 96.45 | 248.7 | 32.3 | 6.70 | 678.8 | fail |
| eager / concurrent / 1 | 200 | 198.5 [197.8–199.2] | 99.65 | 108.7 | 24.0 | 8.91 | 846.1 | fail |
| eager / sync / 1 | 200 | 197.7 [197.1–198.3] | 99.25 | 165.0 | 21.0 | 6.68 | 817.6 | fail |
| lazy / concurrent / 1 | 400 | 336.0 [331.7–340.4] | 84.64 | 116.4 | 5.7 | 9.97 | 999.6 | fail |
| lazy / sync / 1 | 400 | 263.8 [259.5–268.1] | 66.66 | 200.2 | 21.5 | 7.98 | 690.8 | fail |
| eager / concurrent / 1 | 400 | 386.5 [380.6–392.3] | 97.04 | 108.3 | 29.7 | 10.56 | 934.5 | fail |
| eager / sync / 1 | 400 | 291.4 [276.3–306.5] | 73.62 | 135.5 | 0.4 | 8.08 | 958.8 | fail |
| lazy / concurrent / 2 | 100 | 99.6 [99.6–99.6] | 100.00 | 77.0 | 5.8 | 12.02 | 1422.4 | fail |
| lazy / sync / 2 | 100 | 99.6 [99.6–99.6] | 100.00 | 84.1 | 7.0 | 7.44 | 1560.1 | fail |
| eager / concurrent / 2 | 100 | 99.6 [99.6–99.6] | 100.00 | 47.7 | 1.3 | 11.80 | 1786.0 | pass |
| eager / sync / 2 | 100 | 99.6 [99.6–99.6] | 100.00 | 51.1 | 1.3 | 7.13 | 1337.1 | pass |
| lazy / concurrent / 2 | 200 | 199.2 [199.2–199.2] | 100.00 | 145.4 | 17.2 | 14.20 | 1409.2 | fail |
| lazy / sync / 2 | 200 | 199.2 [199.2–199.2] | 100.00 | 48.0 | 0.9 | 8.67 | 1420.9 | pass |
| eager / concurrent / 2 | 200 | 199.2 [199.2–199.2] | 100.00 | 80.1 | 5.6 | 13.77 | 1794.1 | fail |
| eager / sync / 2 | 200 | 199.2 [199.2–199.2] | 100.00 | 108.7 | 10.3 | 8.36 | 1400.2 | fail |
| lazy / concurrent / 2 | 400 | 398.3 [398.3–398.3] | 100.00 | 51.1 | 0.7 | 18.62 | 1449.7 | pass |
| lazy / sync / 2 | 400 | 397.4 [396.4–398.4] | 99.75 | 107.3 | 4.9 | 13.13 | 1364.9 | fail |
| eager / concurrent / 2 | 400 | 397.7 [397.1–398.3] | 99.84 | 97.4 | 4.4 | 18.29 | 1725.7 | fail |
| eager / sync / 2 | 400 | 398.4 [398.4–398.4] | 100.00 | 48.2 | 0.6 | 12.47 | 1489.8 | pass |

Full round/per-app data and per-second timelines are retained in the corresponding compressed results and summaries.

## Cold traffic

15-second windows; 20 ms header gap; 0 ms dependency delay; 0 warmup requests per app; 32 client slots per app. Reused processes across rates: False.

| Mode / apps | Offered/s | Completion/s median [range] | Success % | Worst-app p99 ms | Generator p99 ms | CPU s | RSS MiB | Target |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| lazy / concurrent / 1 | 150 | 117.4 [117.0–117.9] | 78.42 | 336.9 | 7.1 | 11.27 | 523.7 | fail |
| lazy / sync / 1 | 150 | 132.4 [132.3–132.5] | 88.40 | 1849.1 | 14.7 | 7.97 | 925.7 | fail |
| eager / concurrent / 1 | 150 | 149.8 [149.8–149.8] | 100.00 | 35.1 | 1.9 | 9.98 | 837.3 | pass |
| eager / sync / 1 | 150 | 149.8 [149.8–149.8] | 100.00 | 52.7 | 4.6 | 6.84 | 791.3 | fail |

Full round/per-app data and per-second timelines are retained in the corresponding compressed results and summaries.

## Uneven traffic (90/10)

10-second windows; 20 ms header gap; 0 ms dependency delay; 16 warmup requests per app; 32 client slots per app. Reused processes across rates: True.

| Mode / apps | Offered/s | Completion/s median [range] | Success % | Worst-app p99 ms | Generator p99 ms | CPU s | RSS MiB | Target |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| lazy / concurrent / 2 | 300 | 298.7 [298.0–299.4] | 99.77 | 90.3 | 23.0 | 14.16 | 1168.5 | fail |
| lazy / sync / 2 | 300 | 299.4 [299.4–299.4] | 100.00 | 26.5 | 0.5 | 8.89 | 1350.1 | pass |
| eager / concurrent / 2 | 300 | 298.8 [298.1–299.4] | 99.77 | 63.4 | 5.3 | 13.49 | 1432.7 | fail |
| eager / sync / 2 | 300 | 298.5 [297.7–299.4] | 99.72 | 83.2 | 23.6 | 8.46 | 1401.2 | fail |
| lazy / concurrent / 2 | 600 | 585.0 [581.1–589.0] | 97.74 | 130.4 | 46.9 | 15.66 | 1276.2 | fail |
| lazy / sync / 2 | 600 | 580.5 [570.6–590.5] | 97.28 | 81.7 | 11.4 | 12.26 | 1270.9 | fail |
| eager / concurrent / 2 | 600 | 595.8 [592.8–598.8] | 99.50 | 45.1 | 13.7 | 16.04 | 1396.0 | fail |
| eager / sync / 2 | 600 | 594.3 [590.4–598.2] | 99.30 | 65.3 | 14.0 | 12.11 | 1461.2 | fail |

Full round/per-app data and per-second timelines are retained in the corresponding compressed results and summaries.

## Unfragmented complex requests

15-second windows; 0 ms header gap; 0 ms dependency delay; 16 warmup requests per app; 32 client slots per app. Reused processes across rates: True.

| Mode / apps | Offered/s | Completion/s median [range] | Success % | Worst-app p99 ms | Generator p99 ms | CPU s | RSS MiB | Target |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| lazy / concurrent / 1 | 150 | 150.0 [150.0–150.0] | 100.00 | 4.5 | 0.7 | 10.04 | 914.0 | pass |
| lazy / sync / 1 | 150 | 150.0 [150.0–150.0] | 100.00 | 31.1 | 4.9 | 6.84 | 754.6 | fail |
| lazy / concurrent / 1 | 600 | 587.7 [586.5–588.9] | 98.18 | 67.2 | 7.8 | 15.86 | 914.1 | fail |
| lazy / sync / 1 | 600 | 567.0 [542.1–592.0] | 94.68 | 98.4 | 15.5 | 14.47 | 841.1 | fail |

Full round/per-app data and per-second timelines are retained in the corresponding compressed results and summaries.

## Unfragmented requests with dependency

15-second windows; 0 ms header gap; 20 ms dependency delay; 16 warmup requests per app; 32 client slots per app. Reused processes across rates: True.

| Mode / apps | Offered/s | Completion/s median [range] | Success % | Worst-app p99 ms | Generator p99 ms | CPU s | RSS MiB | Target |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| lazy / concurrent / 1 | 100 | 99.9 [99.9–99.9] | 100.00 | 29.7 | 0.9 | 11.31 | 618.7 | pass |
| lazy / sync / 1 | 100 | 99.9 [99.9–99.9] | 100.00 | 98.5 | 17.7 | 6.75 | 705.1 | fail |
| lazy / concurrent / 1 | 400 | 360.4 [354.6–366.3] | 90.45 | 94.7 | 4.1 | 14.68 | 519.2 | fail |
| lazy / sync / 1 | 400 | 292.5 [277.1–307.9] | 73.58 | 152.3 | 10.7 | 12.16 | 788.4 | fail |

Full round/per-app data and per-second timelines are retained in the corresponding compressed results and summaries.

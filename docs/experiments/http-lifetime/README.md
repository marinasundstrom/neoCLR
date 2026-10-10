# HTTP lifetime and handler pipeline validation

The runtime `http-lifetime` suite discovers ten tests for client ownership, use/disposal,
caller/client cancellation, completed responses and async delegating middleware.
`Faults.rvn` supplies eight terminal guard cases which need separate processes.

```sh
python3 scripts/test-runtime-library.py --bundle BUNDLE --output OUTPUT --suite http-lifetime
python3 scripts/validate-http-lifetime.py --bundle BUNDLE --output FAULT_OUTPUT
```

Use fresh output directories and a matching development bundle. The scripts select
macOS or Windows HTTP hosting and compare interpreted/native results. Windows CI
runs both, but local macOS results do not establish Windows execution.

Qualified locally: `target/http-delegating-validation-v8/report.json` and
`target/http-delegating-faults-v3/validation.json`. See the
[lifetime design](../../design/http-client-lifetime.md) for supported behavior and gaps.
Configurable timeout is not implemented by this slice.

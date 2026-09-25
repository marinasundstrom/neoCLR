# HTTP final status and error-conversion checkpoint

Development checkpoint, 2026-09-25. Run from the repository root with a matching
SDK/reference/library bundle and release measure_async runner:

```sh
python3 docs/experiments/http-status/verify.py \
  --toolchain-root /tmp/neoclr-http-status \
  --runner target/release/examples/measure_async
```

The verifier builds each Raven consumer, then compares .NET 10 and neoCLR against an
independent raw TCP peer. Get returns response data for 201, 204, 205, 304, 400, 404,
500 and the unnamed extension code 599. IsSuccessStatusCode and GetString use 200–299;
non-success text requests retain the actual code in HttpError.UnsuccessfulStatus.
The peer checks completion before EOF, empty reason phrases and bodyless statuses,
including a 304 representation length larger than the body buffer bound. Malformed
codes, out-of-range codes, informational responses, forbidden 204/205 framing and
missing required length receive the expected error categories.

A separate neoCLR server returns these statuses to a raw client, which inspects wire
framing/body bytes and checks rejection before output for an invalid status and a
nonempty 204. Server fixtures still serve one GET at a time.

Main.rvn also proves `?` propagation through an application-owned implicit extension
conversion from HttpError to AppError, preserving the nested status cause, and through
boxing into Object. Raven emits informational diagnostic RAV1506 for the extension.
Neither conversion requires a new runtime intrinsic or global HTTP conversion policy.

Observed local macOS results: .NET baseline passed; neoCLR client allocated 3,631
objects across 86 collections, server 1,501 across 39 collections; both finished with
zero live objects under a 256-object heap limit. Signature checks cover the Boolean
success getter and exact Int32 error payload, including rejection of a forged string
payload. Library/API snapshot checks passed. Website builds and full platform matrices
were skipped by author direction. These counts are fixture evidence, not benchmarks.

Fixture authoring exposed existing target/compiler limitations: a captured integer's
address is not admitted for ToString; callback field compound assignment produced an
invalid receiver; a string constant match called an unavailable Object.Equals overload.
The sample uses local copies, owner methods and string equality respectively. These
observations remain candidates for independent Raven reduction, not fixes or evidence
that general language support should be restricted. Importer diagnostics now identify
the offending method/instruction without broadening admission.

See [status design and limitations](../../http-client-design.md#final-response-statuses--implemented-2026-09-25).
Request bodies, additional verbs, informational responses, chunked/close-delimited
framing, redirects and TLS remain separate work. HttpStatusCode enum and request/response
Deconstruct contracts are proposed follow-ups, not implemented by this checkpoint.

Supplementary regression check: the existing http-cancellation `--case headers`
fixture was also attempted twice. Its custom-handler/text checks reached completion,
but the response exchange settled before cancellation; a diagnostic rerun identified
HttpError.TimedOut. This fixture waits for three requests before signaling cancellation,
so native/exchange deadlines can win. The assertion remains strict and the result is
not counted as a pass. Investigate timing/performance and reproduce the previous
checkpoint before release; this observation alone does not establish whether the
status slice caused a regression. Status-fixture successes above remain separate.

## Named-status follow-up

HttpResponse.StatusCode and UnsuccessfulStatus now carry HttpStatusCode. The fixture
casts to int for wire-code checks, verifies NotFound's name/value and unnamed 599
formatting, constructs a typed response and matches its readable properties through
`Ok(HttpResponse { StatusCode: HttpStatusCode.NotFound, Headers: headers })`.
This is optional property inspection, not positional Deconstruct or record equality.
The existing integer response constructor remains covered by the server fixture.

The enum/property follow-up passed: client 3,639 allocations, 86 collections; server
1,501 allocations, 39 collections; both zero live objects. The previous Int32 payload
signature is intentionally incompatible; bridge checks reject it and unrelated enums.
These results supersede the earlier numeric-contract totals, not the historical tests.

A focused follow-up executes CheckNamedStatus with all three property-binding forms:
`if let`, a refutable `let` binding with an else branch, and `is` with an inline
`let` capture. It passes with eight allocations and zero live objects. This keeps
syntax/inspection validation independent of another full networking run.

Cancellation follow-up: the previous checkpoint's original headers fixture passes.
The revised peer removes the independent third request from the cancellation signal's
prerequisites. Current headers and isolated-body runs pass with strict cancellation,
connection-close and independent-request assertions (885/957 allocations respectively,
zero live objects). An overlapping body run still timed out; serial validation avoids
our own competing workload but is not proof of a runtime timing/performance fix.

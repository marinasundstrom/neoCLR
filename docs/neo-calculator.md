# Neo console calculator

The [calculator source](../examples/source/calculator.neo) is a small end-to-end
integer-division program. It combines the companion compiler's control flow, managed
reference parameters, union matching and ordinary System calls. It deliberately
keeps input parsing bounded and adds no special calculator runtime service.

## Run

From the repository root:

```sh
cargo run -- run examples/source/calculator.neo
```

Enter a signed decimal dividend and divisor on separate lines. `84`, then `2`,
prints `= 42`. Division truncates toward zero, so `-7`, then `3`, prints `= -2`.
The program prompts for another pair after each calculation. Signal EOF to finish
(Ctrl-D on Unix at an empty input line). For a reproducible redirected run:

```sh
printf '84\n2\n-7\n3\n' | cargo run -- run examples/source/calculator.neo
```

The calculated result lines are `= 42` and `= -2`; the CLI also prints the final guest
return value `=> Int32(0)`. Prompts and result lines go to stdout.

```sh
printf '84\n2\n' | cargo run -- run examples/source/calculator.neo --gc-stats --gc-events
```

GC diagnostics go to stderr. This program uses ordinary values and scoped references,
so it makes no explicit managed heap allocations. The collector's object counters do
not measure host string storage. Existing diagnostics suffice for this workload.

## Input, recovery and limits

- Accepts ASCII decimal digits with an optional initial `+` or `-`. Spaces and other
  characters are rejected. LF terminates a line; CR bytes are ignored for CRLF input.
- Limits each line to 32 non-CR bytes. Invalid or overlong lines are consumed through
  LF before retrying, subject to the overall input limit. Empty lines and sign-only
  lines reach Int32.Parse and produce an InvalidFormat result.
- Reports Int32 parse overflow and permits retrying the same operand. An invalid
  divisor does not discard the successfully parsed dividend.
- Uses Int32.Divide's Result contract for division by zero and MinValue / -1 overflow.
  After a completed division attempt, including either error, it starts a fresh pair.
- EOF processes a final unterminated line. A dividend without a divisor reports that
  input ended before the divisor; ordinary EOF returns guest Int32(0).
- Reads at most 256 raw bytes per invocation, including CR/LF. Reaching that cap stops
  with a message and guest Int32(2), without reading another byte or processing an
  unfinished line. An exactly full input therefore ends at the cap rather than doing
  an extra read to discover EOF.
- Console unavailability/read failure is a recoverable union case reported with guest
  Int32(1). Runtime instruction, memory and other execution faults remain faults.

Guest return values are reported by the CLI; they are not mapped to operating-system
exit codes. Console calls are synchronous: a byte limit cannot interrupt a host read
that is waiting for input. The embedding API can supply a Console implementation for
scripted input, captured output and injected failures.

## Regression coverage

```sh
cargo test --test neo_calculator --test neo_match --test neo_control_flow --test neo
```

Tests cover repeated signed calculations, CRLF and partial-line EOF, parse/division
errors and retries, invalid bytes, line/session bounds, unavailable/failing input,
instance calls and explicit byte conversion, and CLI stdin/stdout/GC reporting.
A separate loop/match workload retains managed heap objects under a two-object budget
and verifies collection and complete reclamation. It exercises memory management
without adding unnecessary heap allocations to the calculator.

# Time-zone comparisons

Pinned .NET SDK 10.0.100, executed on macOS arm64, 2026-09-27:

```sh
dotnet run --project time-zones.csproj
```

Checks the number of local-time mappings and prints sorted Unix 100 ns timestamps
for Stockholm's gap/overlap, New York's overlap, Lord Howe's half-hour overlap and
Jerusalem's ordinary winter offset. These outputs agree with the native and Raven
fixtures. The probe separately confirms .NET's standard-time choice for Stockholm's
overlap; neoCLR instead returns both candidates. This uses the host .NET zone data,
not a claim that its complete historical/future database matches the pinned IANA data.

# Watching appsettings.json

Development sample: a standalone `System.IO.FileSystemWatcher` reloads a
`JsonConfigurationProvider`. It runs for 30 seconds, printing the `Message` value
on startup and after changes. Edit or atomically replace `appsettings.json` in
the supplied directory while it runs. Comments and trailing commas are accepted.

The monitor subscribes before its first read, watches creation/change/deletion/
rename and rescans after watcher errors. A 200 ms timer coalesces bursts. Failed
reads/parses preserve the provider's previous values. A delay cannot guarantee a
writer has finished; this sample does not implement retry or transactional writes.
It disposes both timers and the watcher explicitly on normal completion.

Build with the current compiler/class-library bundle and the `http` native profile
(`windows-http` on Windows), which supplies timers and file input:

```sh
python3 scripts/build-native-project.py --profile http \
  --bundle /path/to/bundle --aot target/debug/neoclr-codegen \
  --project samples/file-watcher/src/FileWatcherDemo/FileWatcherDemo.rvnproj \
  --output target/filewatch-demo
```

Run `target/filewatch-demo/app /absolute/settings/directory` (`app.exe` on Windows).
The matching native build statically links the OS provider; no extra shared-library
search path is needed. Windows sample execution also depends on the separate native
configuration file-input support; watcher tests themselves do not require that API.
See [deployment and API contracts](../../api-docs/file-system-watcher.md).

This demonstrates a possible configuration consumer. It does not add automatic
reload to `AddJsonFile`, change DI registration or alter `System.Storage`.

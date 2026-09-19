# Try the completed source port in VS Code

The 2026-09-19 local development workspace is at
`~/.neoclr/experiments/raven-port-20260919/demo`. It contains an editable Main.rvn,
matching target metadata, and one task that compiles, imports, verifies and runs the
saved program on neoCLR. The sibling tools directory contains the copied compiler,
language server and importer. This snapshot is not a published release.

Choose **Terminal → Run Task → neoCLR: Run saved project**, or the default build task.
The initial ArrayList/Option/Result example prints 42, Completed, Price overflow,
Skipped, Product not found, Skipped. The examples folder contains normal BindingFlags,
typeof/reflection and array programs; copy one into Main.rvn to try it.
Raven's ordinary Run/Debug commands target .NET, so use the neoCLR task here.

An isolated VS Code profile is at `~/.neoclr/vscode/raven-port-20260919`.
It uses the existing local Raven extension frontend with explicit paths to the fresh
language server and compiler; no global SDK or extension installation is changed.

```sh
code --new-window \
  --user-data-dir "$HOME/.neoclr/vscode/raven-port-20260919" \
  --extensions-dir "$HOME/.neoclr/vscode/raven-port-20260919/extensions" \
  "$HOME/.neoclr/experiments/raven-port-20260919/demo"
```

## Recreate a local snapshot

Build the Raven compiler and language server on branch `neoclr` for net11.0, and
build the neoCLR runtime and importer as described in [port validation](raven-library-port-validation.md).
Then, from the neoCLR checkout, choose a new output directory:

```sh
python3 docs/experiments/raven-target/prepare_port_demo.py /path/to/new-snapshot \
  --raven /path/to/Raven
```

The script refuses to overwrite an existing directory. It copies built binaries,
generates the matching consumer core and System library, retains license notices,
and records source revisions and binary hashes. Python 3 and .NET 11 are required;
the runtime executable is specific to the machine on which it was built. This is a
development convenience, not the release packaging/signing workflow.

The initial exact task command and headless stdio editor checks have passed, covering
completion and hover for collections, arrays, reflection, files, unions, errors,
primitives, parsing, strings, process and calendar APIs. The editor protocol transcript
and server logs are retained in the local demo directory. The actual VS Code client
also records Starting → Running and didOpen for Main.rvn, followed by completed
semantic-token, inlay and code-action requests. Its Raven output log is under the
isolated profile's logs directory; the live server log is under that profile's
User/globalStorage/raven.raven-vscode/language-server directory. This does not claim an
interactive debugger or a new release API. Object.GetType remains a candidate for
subsequent API alignment; typeof replaces the removed TypeOf<T>.Of helper.

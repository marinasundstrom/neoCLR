# Native IDE API documentation — 2026-10-05

Raven `416d185bb` extends the native editor POC with documentation from generated
Raven Markdown/XML sidecars. The real VS Code 1.140.0 extension host passed **18 checks**,
including imported hover and completion prose, documentation-only refresh and XML
fallback after Markdown member deletion. The unchanged collections and Tasks/await
applications still execute. See [machine evidence](native-ide-docs-2026-10-05.json).

Reproduce using the commands in [the editor acceptance](native-vscode-acceptance-2026-10-05.md),
with a fresh output/profile directory. The updated preparation script compiles the
documented fixture library from source outside the application workspace and copies
its generated documentation beside the native DLL. The consumer has no library sources.
The updated extension-host test verifies descriptions, not only signatures.

C# native catalog checks cover type/method XML documentation and malformed optional
XML. Fifteen completion mapping tests pass, including ordinary source documentation.
Native compiler/server build and extension TypeScript compile pass.

This verifies documentation transport and presentation. It does not claim all runtime
APIs have descriptions or that the release bundle contains matching sidecars yet.
Website RavenDoc remains a separate publisher using its documented CLI reference
assembly. Runtime reflection and native assembly encoding have not changed.

Website validation: the pinned RavenDoc publisher rendered and checked **1,803 pages**;
all **18 website tests** passed. The build validates API inventory, authored member
links, legacy routes and local links for root/project-prefix hosting. Browser inspection
confirmed the Option type page presents a union signature, API description and remarks.
Missing API prose warnings remain tracked; they are not missing page failures.

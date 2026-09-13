# Intrinsic String defaults in managed storage

2026-09-13. Existing String storage now has a typed null default. `initobj String`,
string fields initialized during construction, and `newarr String` no longer fail
merely because the slot's type is String. Writing a real string replaces that null;
null and the empty string remain distinct values.

This follows [.NET's reference default](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/null)
and [string reference semantics](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/strings/).
It is required by ordinary CLI string arrays, including the Environment argument
snapshot. Previously the experiment admitted only arrays whose elements had a
runtime default, excluding String despite its reference metadata in Raven.

The implementation reuses typed null reference storage. Non-null strings retain
the runtime's intrinsic string representation and existing method receiver ABI.
This is not a claim of complete CLR String object identity, interning, interface
casting or null-aware overload parity. Those require their own projection and
validation. Native text operations still require actual string data and fault
when that contract is violated; Option remains the environment API's absence model.

This changes the earlier rejection of managed String defaults. It does not add
nullable metadata, change native pointer initialization, invent an empty-string
default, or make an uninitialized union carrier a valid case. Native `initobj String` through a pointer is rejected by typed verification,
which distinguishes pointer destinations from managed addresses; it is no longer
rejected solely from the opcode operand during assembly. The bounded Raven
importer can remain more conservative about explicit defaults than the runtime.

`tests/string_defaults.rs` covers array defaults, replacement, locals and constructed
class fields. Existing array/reference tests cover overlap with other storage.

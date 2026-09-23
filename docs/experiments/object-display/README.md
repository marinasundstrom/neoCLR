# Object display POC

Development after Preview 9. Run `python3 docs/experiments/object-display/verify.py
--toolchain-root /path/to/bundle` (on one line) with matching development artifacts.

The sample calls ToString through Object and a base class, then calls base.ToString
explicitly. Output is checked against expected.txt. Object supplies the concrete
runtime type name; Named overrides it with domain display text. Class assignment
continues to share reference identity. This is formatting, not serialization.

The Rust object_display tests also cover rootless classes, managed arrays, null and
explicit rejection of boxed-value/intrinsic-string virtual dispatch. Those latter
paths need separate receiver/override work; GetType still supports them. Equality,
hashing, cloning and Value retirement are separate slices. See the
[review](../../object-model-review.md) and [on-site guide](../../../api-docs/objects.md).

Object is abstract. Abstract.rvn is a negative fixture: direct construction must
fail compilation. The runtime test also rejects raw-IL construction, while the
positive sample confirms derived classes complete their Object base constructor.

"""Reference-profile fixture resolver. The host supplies authoritative definitions."""
from dataclasses import dataclass
import struct
from uuid import UUID

from codec import FormatError
from signatures import Context, MAX_ARITY, decode_signature, encode_signature

REFERENCE_SECTION = 2
SIGNATURE_SECTION = 3
SCHEMA = 1
MAX_REFERENCES = 256
TYPE_DEF = 0x02
METHOD_DEF = 0x06


@dataclass(frozen=True)
class Reference:
    assembly: UUID
    module: UUID
    token: int


@dataclass(frozen=True)
class Bindings:
    references: tuple
    type_owner: int = 0
    method_owner: int = 0
    self_owner: int = 0


@dataclass(frozen=True)
class Definition:
    kind: str  # type, interface or method
    arity: int = 0
    owner: object = None  # declaring type Reference for a method


def validate_reference(reference):
    if not isinstance(reference, Reference):
        raise FormatError("invalid reference")
    if any(not isinstance(value, UUID) or value.int == 0
           for value in (reference.assembly, reference.module)):
        raise FormatError("invalid assembly/module identity")
    token = reference.token
    if type(token) is not int or not 0 < token <= 0xffffffff:
        raise FormatError("invalid definition token")
    if token >> 24 not in (TYPE_DEF, METHOD_DEF) or token & 0xffffff == 0:
        raise FormatError("reference must identify a TypeDef or MethodDef")


def encode_bindings(bindings):
    refs = bindings.references
    if len(refs) > MAX_REFERENCES:
        raise FormatError("too many references")
    for ref in refs:
        validate_reference(ref)
    if len(set(refs)) != len(refs):
        raise FormatError("duplicate reference")
    for owner in (bindings.type_owner, bindings.method_owner, bindings.self_owner):
        if type(owner) is not int or not 0 <= owner <= len(refs):
            raise FormatError("owner reference outside table")
    for index, expected in ((bindings.type_owner, TYPE_DEF),
                            (bindings.method_owner, METHOD_DEF), (bindings.self_owner, TYPE_DEF)):
        if index and refs[index - 1].token >> 24 != expected:
            raise FormatError("wrong owner reference kind")
    data = bytearray(struct.pack("<HHHH", len(refs), bindings.type_owner,
                                 bindings.method_owner, bindings.self_owner))
    for ref in refs:
        data.extend(ref.assembly.bytes)
        data.extend(ref.module.bytes)
        data.extend(struct.pack("<I", ref.token))
    return bytes(data)


def decode_bindings(data):
    if len(data) < 8:
        raise FormatError("truncated reference table")
    count, type_owner, method_owner, self_owner = struct.unpack_from("<HHHH", data)
    if count > MAX_REFERENCES or len(data) != 8 + 36 * count:
        raise FormatError("invalid reference table length/count")
    refs = []
    for offset in range(8, len(data), 36):
        refs.append(Reference(UUID(bytes=bytes(data[offset:offset + 16])),
                              UUID(bytes=bytes(data[offset + 16:offset + 32])),
                              struct.unpack_from("<I", data, offset + 32)[0]))
    bindings = Bindings(tuple(refs), type_owner, method_owner, self_owner)
    encode_bindings(bindings)
    return bindings


def validate_local(root, context, bindings):
    """Check use-site indices before resolution; no catalog or identity claim yet."""
    encode_signature(root, context, references=True)
    encode_bindings(bindings)
    if (context.type_parameters and not bindings.type_owner
            or context.method_parameters and not bindings.method_owner
            or context.self_allowed != bool(bindings.self_owner)):
        raise FormatError("signature context lacks matching owner")

    def check(node):
        if node.kind == "nominal":
            if node.index > len(bindings.references):
                raise FormatError("nominal reference outside table")
            if bindings.references[node.index - 1].token >> 24 != TYPE_DEF:
                raise FormatError("nominal reference is not a type")
        for child in node.children:
            check(child)
    check(root)


def resolve(root, context, bindings, catalog):
    """Return immutable equality keys scoped to a host catalog, not persistent IDs."""
    validate_local(root, context, bindings)

    def definition(ref):
        value = catalog.get(ref)
        if not isinstance(value, Definition):
            raise FormatError("unresolved definition")
        expected = "method" if ref.token >> 24 == METHOD_DEF else "type"
        if ((expected == "method" and value.kind != "method")
                or (expected == "type" and value.kind not in ("type", "interface"))):
            raise FormatError("catalog definition kind mismatch")
        if type(value.arity) is not int or not 0 <= value.arity <= MAX_ARITY:
            raise FormatError("invalid catalog arity")
        if value.kind == "method":
            validate_reference(value.owner)
            if value.owner.token >> 24 != TYPE_DEF:
                raise FormatError("method requires declaring type")
        elif value.owner is not None:
            raise FormatError("unexpected catalog owner")
        return value

    # Resolve all declared dependencies, including otherwise-unused references.
    for ref in bindings.references:
        definition(ref)

    def owner(index):
        return bindings.references[index - 1] if index else None

    type_owner, method_owner, self_owner = map(owner, (
        bindings.type_owner, bindings.method_owner, bindings.self_owner))
    for ref, count in ((type_owner, context.type_parameters),
                       (method_owner, context.method_parameters)):
        if ref is not None and definition(ref).arity != count:
            raise FormatError("binder arity disagrees with definition")
    if method_owner is not None:
        method = definition(method_owner)
        definition(method.owner)
        if method.owner != type_owner:
            raise FormatError("method/type owner mismatch")
    if self_owner is not None:
        contract = definition(self_owner)
        if contract.kind != "interface" or contract.arity != 0:
            raise FormatError("Self owner must be a nongeneric interface contract")

    def key(node):
        children = tuple(key(child) for child in node.children)
        if node.kind == "nominal":
            ref = bindings.references[node.index - 1]
            if definition(ref).arity != len(children):
                raise FormatError("nominal generic argument count mismatch")
            return ("nominal", ref, children)
        if node.kind in ("type_parameter", "method_parameter"):
            ref = type_owner if node.kind == "type_parameter" else method_owner
            return (node.kind, ref, node.index)
        if node.kind == "self":
            return ("self", self_owner)
        if node.kind in ("union", "intersection"):
            # Preserve syntax in the codec; flatten like operators and ignore duplicate/order here.
            flattened = set()
            for child in children:
                if child[0] == node.kind:
                    flattened.update(child[1])
                else:
                    flattened.add(child)
            return (node.kind, frozenset(flattened))
        return (node.kind, children, node.modes, node.no_result)

    return key(root)


def read_profile(sections):
    """Recognize a complete reference profile, independent of directory order."""
    selected = {s.kind: s for s in sections if s.kind in (REFERENCE_SECTION, SIGNATURE_SECTION) and s.version == SCHEMA}
    if not selected:
        return None
    if set(selected) != {REFERENCE_SECTION, SIGNATURE_SECTION}:
        raise FormatError("incomplete reference profile")
    if any(s.version != SCHEMA or not s.required for s in selected.values()):
        raise FormatError("reference profile requires supported mandatory sections")
    if any(s.kind == 1 for s in sections):
        raise FormatError("cannot mix local and reference signature profiles")
    bindings = decode_bindings(selected[REFERENCE_SECTION].payload)
    root, context = decode_signature(selected[SIGNATURE_SECTION].payload, references=True)
    validate_local(root, context, bindings)
    return root, context, bindings

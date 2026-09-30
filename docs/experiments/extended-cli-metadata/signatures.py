"""Experimental structural signature trees, with no CLI element-code allocations."""
from dataclasses import dataclass
import struct

from codec import FormatError

SECTION_KIND = 1
SCHEMA = 1
MAX_DEPTH = 32
MAX_NODES = 4096
MAX_ARITY = 256
TAGS = {"int32": 1, "string": 2, "bool": 3, "unit": 4,
        "type_parameter": 5, "method_parameter": 6, "self": 7,
        "array": 8, "tuple": 9, "function": 10, "union": 11,
        "intersection": 12, "nullable": 13, "array_ref": 14}
KINDS = {value: key for key, value in TAGS.items()}
MODES = {"value": 0, "ref": 1, "readonly_ref": 2, "out": 3, "out_when_true": 4}
MODE_NAMES = {value: key for key, value in MODES.items()}


@dataclass(frozen=True)
class Context:
    type_parameters: int = 0
    method_parameters: int = 0
    self_allowed: bool = False


@dataclass(frozen=True)
class Node:
    kind: str
    children: tuple = ()
    index: int = 0
    modes: tuple = ()
    no_result: bool = False


def validate_context(context):
    if any(type(n) is not int or not 0 <= n <= MAX_ARITY
           for n in (context.type_parameters, context.method_parameters)):
        raise FormatError("invalid binder arity")
    if type(context.self_allowed) is not bool:
        raise FormatError("invalid Self context flag")


def encode_signature(root, context=Context()):
    validate_context(context)
    remaining = MAX_NODES

    def node(value, depth):
        nonlocal remaining
        remaining -= 1
        if depth > MAX_DEPTH or remaining < 0:
            raise FormatError("signature resource limit")
        if not isinstance(value, Node) or value.kind not in TAGS:
            raise FormatError("unknown signature kind")
        kind = value.kind
        if type(value.no_result) is not bool or (kind != "function" and value.no_result):
            raise FormatError("invalid no-result flag")
        children = value.children
        if len(children) > MAX_ARITY + 1:
            raise FormatError("signature arity limit")
        if kind != "function" and value.modes:
            raise FormatError("unexpected parameter modes")
        if kind not in ("type_parameter", "method_parameter") and value.index != 0:
            raise FormatError("unexpected index")
        payload = bytearray()
        if kind in ("type_parameter", "method_parameter"):
            bound = context.type_parameters if kind == "type_parameter" else context.method_parameters
            if type(value.index) is not int or not 0 <= value.index < bound:
                raise FormatError("generic parameter outside binder")
            payload.extend(struct.pack("<H", value.index))
        if kind == "self" and not context.self_allowed:
            raise FormatError("Self outside contract context")
        if kind in ("array", "array_ref", "nullable"):
            if len(children) != 1:
                raise FormatError("unary type requires one child")
            payload.extend(node(children[0], depth + 1))
        elif kind in ("tuple", "union", "intersection"):
            minimum = 1 if kind == "tuple" else 2
            if not minimum <= len(children) <= MAX_ARITY:
                raise FormatError("invalid compound arity")
            payload.extend(struct.pack("<H", len(children)))
            for child in children:
                payload.extend(node(child, depth + 1))
        elif kind == "function":
            if not children or len(value.modes) != len(children) - 1:
                raise FormatError("function requires result and matching parameter modes")
            if value.no_result and children[-1] != Node("unit"):
                raise FormatError("no-result Function requires unit result")
            if "out_when_true" in value.modes and (children[-1] != Node("bool") or value.no_result):
                raise FormatError("conditional output requires bool result")
            payload.extend(struct.pack("<BBH", 0, int(value.no_result), len(value.modes)))  # managed convention only
            for mode, child in zip(value.modes, children[:-1]):
                if mode not in MODES:
                    raise FormatError("unknown parameter mode")
                payload.append(MODES[mode])
                payload.extend(node(child, depth + 1))
            payload.extend(node(children[-1], depth + 1))
        elif children:
            raise FormatError("leaf type has children")
        return struct.pack("<BI", TAGS[kind], len(payload)) + payload

    return struct.pack("<HHB", context.type_parameters, context.method_parameters,
                       int(context.self_allowed)) + node(root, 0)


def decode_signature(data):
    if len(data) < 5 or len(data) > 1024 * 1024:
        raise FormatError("invalid signature size")
    types, methods, self_flag = struct.unpack_from("<HHB", data)
    if self_flag not in (0, 1):
        raise FormatError("invalid Self context flag")
    context = Context(types, methods, bool(self_flag))
    validate_context(context)
    remaining = MAX_NODES

    def node(position, limit, depth):
        nonlocal remaining
        remaining -= 1
        if depth > MAX_DEPTH or remaining < 0:
            raise FormatError("signature resource limit")
        if limit - position < 5:
            raise FormatError("truncated type node")
        tag, size = struct.unpack_from("<BI", data, position)
        position += 5
        if size > limit - position:
            raise FormatError("invalid type node length")
        end = position + size
        kind = KINDS.get(tag)
        if kind is None:
            raise FormatError("unknown required type node")
        children, modes, index, no_result = [], [], 0, False

        def take(code):
            nonlocal position
            width = struct.calcsize("<" + code)
            if width > end - position:
                raise FormatError("truncated type payload")
            values = struct.unpack_from("<" + code, data, position)
            position += width
            return values

        def child():
            nonlocal position
            value, position = node(position, end, depth + 1)
            children.append(value)

        if kind in ("type_parameter", "method_parameter"):
            index, = take("H")
        elif kind in ("array", "array_ref", "nullable"):
            child()
        elif kind in ("tuple", "union", "intersection"):
            count, = take("H")
            if count > MAX_ARITY:
                raise FormatError("signature arity limit")
            for _ in range(count):
                child()
        elif kind == "function":
            convention, flags, count = take("BBH")
            if flags not in (0, 1):
                raise FormatError("invalid no-result flag")
            no_result = bool(flags)
            if convention != 0 or count > MAX_ARITY:
                raise FormatError("unsupported calling convention or arity")
            for _ in range(count):
                mode, = take("B")
                if mode not in MODE_NAMES:
                    raise FormatError("unknown parameter mode")
                modes.append(MODE_NAMES[mode])
                child()
            child()
        if position != end:
            raise FormatError("trailing type payload")
        return Node(kind, tuple(children), index, tuple(modes), no_result), end

    root, end = node(5, len(data), 0)
    if end != len(data):
        raise FormatError("trailing signature bytes")
    # Apply the same shape/context rules on both paths, including leaf payloads.
    if encode_signature(root, context) != data:
        raise FormatError("noncanonical signature encoding")
    return root, context

//! Bounded inline value records. All managed addresses borrow active frame storage.
use super::{Error, flow};
use neoclr::metadata::{FunctionRef, Instruction as Op, Representation, Type};
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Ty {
    Int,
    Byte,
    SByte,
    Short,
    UShort,
    Bool,
    Unit,
    Erased,
    Character, // One validated extended grapheme, stored as immutable UTF-8 text.
    Literal,   // Immutable image/explicit invocation-arena UTF-8; not a general managed String.
    Record(usize),
    ByteMemory(usize), // owner pointer + offset + length; kind remains in the type.
    Tuple(Vec<Ty>),
    Reference(usize),
    Interface(usize),
    Intersection(Vec<Ty>),
    ReferenceUnion(Vec<Ty>),
    ScalarUnion(Vec<Type>),
    ValueUnion(Vec<Ty>), // Discriminator plus disjoint, zeroed inactive payload regions.
    MixedUnion(Vec<Ty>), // Tag, numeric bits, and a separate nullable reference slot.
    PendingUnionScalar(Type),
    ScalarUnionTest, // Ephemeral non-null discriminator, consumed only by ref.isnull.
    ByteArray,
    CharacterArray,  // Same pointer-slot layout, distinct element identity.
    StringArray,     // Invocation-owned pointer slots; verified nominal backing enables views.
    CharacterValues, // Immutable grapheme snapshot; distinct from mutable character arrays.
    ByteValues,      // Immutable native-produced value-array snapshot; no element addresses.
    Size,
    TypeToken, // Image-local opaque type identity, never a native address.
    Wide,
    Double,
    Single,
    ReferenceArray(usize), // Default-null or reserved slots retaining nominal object identity.
    StructuralArray(Type), // Checked snapshots of signature-native value/view storage.
    RecordArray(usize),    // Checked reserved snapshots of a closed value record.
    ScalarArray(Type),     // Atomic one-lane slots; exact element identity retained.
    CallableArray(Type),   // Initialized/reserved pointer slots of one exact Function shape.
    Callable(Type),        // Managed descriptor: target identity and strong heap receiver.
    Address(Box<Ty>),
}
impl Ty {
    pub fn address(self) -> Self {
        Self::Address(Box::new(self))
    }
}
pub(super) struct Profile<'a> {
    pub input: &'a neoclr::Module,
    references: bool,
    object_base: Option<usize>,
    pub array_backing: Option<usize>,
    pub reference_array_backings: Vec<usize>,
    layouts: Vec<Vec<usize>>,
    widths: Vec<usize>,
    names: HashMap<&'a str, Vec<usize>>,
    pub args: Vec<Vec<Ty>>,
    pub locals: Vec<Vec<Ty>>,
    pub results: Vec<Option<Ty>>,
    pub dispatch: HashMap<usize, Vec<(usize, usize)>>,
}
pub(super) type Stacks = Vec<Option<Vec<Ty>>>;

// Private compilation tags, not runtime Type ordinals or a public/native ABI.
pub(super) fn erased_tag(ty: &Type) -> Result<i64, Error> {
    match ty {
        Type::Void => Ok(0),
        Type::Int32 => Ok(1),
        Type::Byte => Ok(2),
        Type::Boolean => Ok(3),
        Type::String => Ok(4),
        Type::Int64 => Ok(5),
        Type::UInt64 => Ok(6),
        Type::SByte => Ok(7),
        Type::Int16 => Ok(8),
        Type::UInt16 => Ok(9),
        Type::UInt32 => Ok(10),
        Type::Single => Ok(11),
        Type::Double => Ok(12),
        _ => Err("erased payload requires a supported primitive or String".into()),
    }
}

fn object_base_shape(t: &neoclr::metadata::TypeDef) -> bool {
    t.is_reference_type
        && t.is_abstract
        && t.representation == Representation::Record
        && t.fields.is_empty()
        && t.base.is_none()
        && t.generic_parameters.is_empty()
        && t.implements.is_empty()
}

impl<'a> Profile<'a> {
    pub fn new(
        input: &'a neoclr::Module,
        references: bool,
        object_base: Option<usize>,
        array_backing: Option<usize>,
        reference_backings: &[usize],
        object_display: Option<&HashMap<usize, Vec<(usize, usize)>>>,
        string_dispatch: Option<&HashMap<usize, usize>>,
        interface_dispatch: Option<&HashMap<usize, Vec<(usize, usize)>>>,
        primitive_receivers: Option<&[usize]>,
        native_stack_budget: bool,
    ) -> Result<Self, Error> {
        if input.name == "System"
            || input.types.len() > crate::limits::TYPES
            || input.functions.len() > crate::limits::FUNCTIONS
        {
            return Err(format!(
                "value profile requires an application with at most {} types and {} functions",
                crate::limits::TYPES,
                crate::limits::FUNCTIONS
            )
            .into());
        }
        if input
            .functions
            .iter()
            .any(|f| f.body.iter().any(|op| matches!(op, Op::ReserveArray(_))))
            && input
                .functions
                .iter()
                .any(|f| f.body.iter().any(|op| matches!(op, Op::ArrayAddress(_))))
        {
            return Err("reserved byte arrays require indexed access; element borrows need initialization-aware addresses".into());
        }
        if object_base.is_some_and(|i| {
            !references || input.types.get(i).is_none_or(|t| !object_base_shape(t))
        }) {
            return Err("invalid private Object base projection".into());
        }
        if array_backing.is_some_and(|i| {
            !references
                || input.types.get(i).is_none_or(|t| {
                    !t.is_reference_type
                        || t.representation != Representation::Record
                        || t.fields.len() != 1
                        || t.fields[0].ty != Type::ArrayRef(Box::new(Type::Byte))
                        || !t.generic_parameters.is_empty()
                })
        }) {
            return Err("invalid private byte-array backing projection".into());
        }
        for &i in reference_backings {
            let valid = input.types.get(i).is_some_and(|t| t.is_reference_type
                && t.representation == Representation::Record && t.fields.len() == 1
                && t.generic_parameters.is_empty()
                && matches!(&t.fields[0].ty, Type::ArrayRef(element) if matches!(**element, Type::String | Type::Char | Type::ArrayRef(_)) || crate::selection::scalar_array_element(element) || input.type_definition(element).is_some_and(|e| (e.is_reference_type && e.representation == Representation::Record) || e.representation == Representation::Interface)));
            if !references || !valid {
                return Err("invalid private reference-array backing projection".into());
            }
        }
        for t in &input.types {
            if t.fields
                .iter()
                .map(|f| &f.ty)
                .chain(t.static_fields.iter().map(|f| &f.ty))
                .any(|ty| {
                    input
                        .type_definition(ty)
                        .is_some_and(|t| t.memory_view.is_some_and(|k| k.is_borrowed()))
                })
            {
                return Err("borrowed byte views cannot be stored in fields".into());
            }
            // Selection also gates these; direct profile consumers must not
            // interpret opaque fieldless declarations as empty, untraced records.
            if (t.memory_view.is_some()
                || neoclr::native_metadata::memory_view_element(t).is_some())
                && !(references && crate::memory_views::byte_memory(t))
            {
                return Err(
                    "memory views require native ABI and GC lowering before value profiling".into(),
                );
            }
            if references && t.representation == Representation::Interface {
                if !t.fields.is_empty()
                    || t.base.is_some()
                    || t.implements.iter().any(|parent| {
                        input
                            .type_definition(parent)
                            .is_none_or(|p| p.representation != Representation::Interface)
                    })
                    || !t.generic_parameters.is_empty()
                    || !t.generic_constraints.is_empty()
                    || t.enum_info.is_some()
                    || t.packing.is_some()
                    || t.minimum_size.is_some()
                {
                    return Err(
                        "interface views require fieldless closed interface contracts".into(),
                    );
                }
                continue;
            }
            if !t.implements.is_empty()
                && (!references
                    || t.implements.iter().any(|interface| {
                        !matches!(interface, Type::Named(_))
                            || input.type_definition(interface).is_none_or(|t| {
                                t.representation != Representation::Interface
                                    || !t.generic_parameters.is_empty()
                            })
                    }))
            {
                return Err(format!(
                    "{}: only reference-arena nongeneric interface views are supported",
                    t.name
                )
                .into());
            }
            let static_owner = crate::selection::static_owner(t);
            let reference_base = references
                && t.is_reference_type
                && t.base.as_ref().is_some_and(|base| {
                    input.type_definition(base).is_some_and(|b| {
                        b.is_reference_type
                            && b.representation == Representation::Record
                            && b.generic_parameters.is_empty()
                    })
                });
            if (t.is_reference_type && !static_owner && !references)
                || t.representation != Representation::Record
                || t.enum_info.as_ref().is_some_and(|info| info.underlying != Type::Int32
                    || t.is_reference_type || t.fields.len() != 1
                    || t.fields[0].ty != Type::Int32
                    || t.fields[0].visibility != neoclr::metadata::Visibility::Private)
                || (t.base.is_some() && !reference_base
                    && !neoclr::native_metadata::has_value_type_base(input, &Type::Named(t.name.clone())))
                || (t.is_abstract && !static_owner && !(references && t.is_reference_type))
                || !t.generic_parameters.is_empty()
                || !t.generic_constraints.is_empty()
                || t.packing.is_some()
                || t.minimum_size.is_some()
                || (!t.is_reference_type && t.fields.len() > 16)
                || t.fields.iter().any(|f| {
                    f.deferred
                        || !(matches!(f.ty, Type::Single | Type::Double | Type::Int32 | Type::Byte | Type::SByte | Type::Int16 | Type::UInt16 | Type::UInt32 | Type::Int64 | Type::UInt64 | Type::IntPtr | Type::UIntPtr | Type::RuntimeTypeHandle | Type::Boolean | Type::Void | Type::String | Type::Char | Type::Named(_) | Type::Tuple(_))
                            || (references && matches!(&f.ty, Type::Function(_) | Type::Union(_) | Type::Intersection(_)))
                            || (references && matches!(&f.ty, Type::ArrayRef(t) if matches!(**t, Type::Byte | Type::String | Type::Char | Type::Function(_) | Type::Named(_) | Type::Tuple(_) | Type::Union(_) | Type::Intersection(_) | Type::ArrayRef(_)) || crate::selection::scalar_array_element(t))))
                })
            {
                return Err(format!("{}: value profile requires supported closed records (inline values have at most sixteen fields)", t.name).into());
            }
        }
        let mut p = Self {
            input,
            references,
            object_base,
            array_backing,
            reference_array_backings: reference_backings.to_vec(),
            layouts: vec![vec![]; input.types.len()],
            widths: vec![0; input.types.len()],
            names: HashMap::new(),
            args: vec![],
            locals: vec![],
            results: vec![],
            dispatch: object_display.cloned().unwrap_or_default(),
        };
        // Compute bounded inline layouts before using any storage or call signatures.
        // A visiting node is an illegal inline cycle, including otherwise unused types.
        fn layout(p: &mut Profile<'_>, i: usize, states: &mut [u8]) -> Result<(), Error> {
            if states[i] == 1 {
                return Err("recursive inline value layout is unsupported".into());
            }
            if states[i] == 2 {
                return Ok(());
            }
            states[i] = 1;
            if crate::memory_views::byte_memory(&p.input.types[i]) {
                p.widths[i] = 3;
                states[i] = 2;
                return Ok(());
            }
            let fields = p.input.types[i].fields.clone();
            let mut width = 0;
            // ValueType supplies nominal ancestry, never inline storage.
            if let Some(base) = p.input.types[i]
                .base
                .as_ref()
                .filter(|_| p.input.types[i].is_reference_type)
            {
                let Ty::Reference(parent) = p.ty(base)? else {
                    return Err("reference base must be a class".into());
                };
                layout(p, parent, states)?;
                width = p.widths[parent];
                p.layouts[i] = p.layouts[parent].clone();
            }
            for field in fields {
                let ty = p.stored(&field.ty)?;
                fn dependencies(
                    p: &mut Profile<'_>,
                    ty: &Ty,
                    states: &mut [u8],
                ) -> Result<(), Error> {
                    match ty {
                        Ty::Record(child) => layout(p, *child, states)?,
                        Ty::Tuple(parts) | Ty::ValueUnion(parts) => {
                            for part in parts {
                                dependencies(p, part, states)?;
                            }
                        }
                        _ => (),
                    }
                    Ok(())
                }
                dependencies(p, &ty, states)?;
                if matches!(ty, Ty::ValueUnion(_)) && p.lanes(&ty) > crate::limits::INLINE_LANES {
                    return Err("value union exceeds flattened inline lane limit".into());
                }
                p.layouts[i].push(width);
                width += p.lanes(&ty);
                if !p.input.types[i].is_reference_type && width > crate::limits::INLINE_LANES {
                    return Err(format!(
                        "{}: value layout exceeds {} flattened lanes ({width})",
                        p.input.types[i].name,
                        crate::limits::INLINE_LANES
                    )
                    .into());
                }
            }
            if p.input.types[i].is_reference_type && width > crate::limits::OBJECT_LANES {
                return Err(format!(
                    "{}: reference hierarchy exceeds the {}-byte native object bound ({width} flattened lanes)",
                    p.input.types[i].name, 8 + crate::limits::OBJECT_LANES * 8
                ).into());
            }
            p.widths[i] = width.max(1);
            states[i] = 2;
            Ok(())
        }
        let mut states = vec![0; input.types.len()];
        for i in 0..input.types.len() {
            layout(&mut p, i, &mut states)?;
        }
        for (i, f) in input.functions.iter().enumerate() {
            p.names.entry(&f.name).or_default().push(i);
            let interface = references && crate::selection::interface_contract(input, f);
            if interface && interface_dispatch.is_some_and(|targets| targets.contains_key(&i)) {
                p.dispatch
                    .insert(i, interface_dispatch.unwrap()[&i].clone());
            } else if interface {
                let reached = (0..input.functions.len()).collect();
                p.dispatch.insert(
                    i,
                    crate::selection::dispatch_targets_with_array(
                        input,
                        i,
                        &reached,
                        array_backing,
                        reference_backings,
                        None,
                    )?,
                );
            }
            let class = references && crate::selection::class_contract(input, f);
            // Original metadata verification owns mapping/access validity. Selection
            // has already resolved explicit interface targets; these remain private
            // ordinary reference receivers, not class virtual slots.
            let explicit_reference = references
                && f.instance
                && !f.receiver_byref
                && f.visibility == neoclr::metadata::Visibility::Private
                && f.owner
                    .as_ref()
                    .and_then(Type::definition_name)
                    .is_some_and(|name| {
                        input.types.iter().any(|ty| {
                            ty.name == name
                                && ty.is_reference_type
                                && ty.representation == Representation::Record
                        })
                    });
            if ((f.is_virtual || f.is_abstract) && !interface && !class)
                || f.is_override && !class
                || f.impl_flags != 0
                || f.pinvoke.is_some()
                || !f.generic_parameters.is_empty()
                || !f.generic_arguments.is_empty()
                || !f.generic_constraints.is_empty()
                || !f.out_when_true.is_empty()
                || !f.readonly_parameters.is_empty()
                || f.receiver_readonly
                || (!f.interface_implementations.is_empty() && !explicit_reference)
                || f.body.len() > 8192
                || f.locals.len() > 1024
                || f.parameters.len() > 32
            {
                return Err(format!(
                    "{}: unsupported value member contract (owner {:?}, virtual {}, abstract {}, override {}, impl {}, generics {}/{}, readonly {:?}/{})",
                    f.name, f.owner, f.is_virtual, f.is_abstract, f.is_override,
                    f.impl_flags, f.generic_parameters.len(), f.generic_arguments.len(),
                    f.readonly_parameters, f.receiver_readonly
                ).into());
            }
            if f.out_parameters
                .iter()
                .any(|n| !matches!(f.parameters.get(*n), Some(Type::ByRef(_))))
                || f.out_parameters.windows(2).any(|pair| pair[0] >= pair[1])
            {
                return Err("output indices must be sorted, unique borrowed parameters".into());
            }
            let mut args = vec![];
            if let Some(owner) = &f.owner {
                let metadata_only = matches!(owner, Type::Named(name) if input.types.iter()
                    .any(|t| t.name == *name && crate::selection::static_owner(t)));
                if metadata_only && !f.instance && !f.receiver_byref {
                    // Static owner contributes identity/access only, with no receiver.
                } else {
                    let ty = p.ty(owner)?;
                    if !matches!(
                        ty,
                        Ty::Record(_) | Ty::ByteMemory(_) | Ty::Reference(_) | Ty::Interface(_)
                    ) {
                        return Err(format!(
                            "{}: value member requires a local record owner ({owner:?})",
                            f.name
                        )
                        .into());
                    }
                    if f.instance {
                        match ty {
                            Ty::Reference(_) | Ty::Interface(_) if !f.receiver_byref => {
                                args.push(ty)
                            }
                            Ty::Record(_) | Ty::ByteMemory(_) if f.receiver_byref => {
                                args.push(ty.address())
                            }
                            _ => {
                                return Err(
                                    "receiver representation does not match value/reference owner"
                                        .into(),
                                );
                            }
                        }
                    } else if f.receiver_byref {
                        return Err("static member cannot have a by-reference receiver".into());
                    }
                }
            } else if f.instance || f.receiver_byref {
                return Err("instance member requires a record owner".into());
            }
            for (index, t) in f.parameters.iter().enumerate() {
                if let Type::ByRef(target) = t {
                    let projected_receiver = index == 0
                        && matches!(
                            **target,
                            Type::Int32
                                | Type::Boolean
                                | Type::Int64
                                | Type::UInt64
                                | Type::IntPtr
                                | Type::UIntPtr
                                | Type::Char
                        )
                        && primitive_receivers.is_some_and(|indices| indices.contains(&i));
                    if !f.out_parameters.contains(&index) && !projected_receiver {
                        return Err(
                            "explicit borrowed parameters require an output contract".into()
                        );
                    }
                    args.push(p.stored(target)?.address());
                } else {
                    args.push(p.stored(t)?);
                }
            }
            p.args.push(args);
            p.locals.push(
                f.locals
                    .iter()
                    .map(|t| p.stored(t))
                    .collect::<Result<_, _>>()?,
            );
            p.results.push(if f.no_result {
                if f.returns != Type::Void {
                    return Err("no-result member must return Void".into());
                }
                None
            } else {
                Some(p.stored(&f.returns)?)
            });
        }
        // Resolve all calls (even dead ones) and reject recursive native call graphs.
        let mut edges = vec![vec![]; input.functions.len()];
        for (i, f) in input.functions.iter().enumerate() {
            let owner = f.owner.as_ref().and_then(|t| input.type_definition(t));
            let reference_constructor = f.instance
                && f.name.ends_with("..ctor")
                && owner.is_some_and(|t| t.is_reference_type);
            // Keep the uninitialized receiver below independently evaluated arguments.
            // The ordinary typed-stack verifier still checks operand and call types.
            let base_initializer = if reference_constructor && owner.unwrap().base.is_some() {
                Some(leading_base_initializer(&f.body, owner.unwrap().base.as_ref().unwrap(), &|target| {
                        p.callee(target).ok().map(|index| input.functions[index].no_result)
                    })
                    .ok_or("reference constructor requires an unconditional leading base initializer with receiver-independent arguments")?)
            } else {
                None
            };
            for (pc, op) in f.body.iter().enumerate() {
                if let Op::BindFunction {
                    function_type,
                    target,
                } = op
                {
                    if !f.out_parameters.is_empty() {
                        return Err("native callback binding in output-parameter methods requires a later profile".into());
                    }
                    p.ty(function_type)?;
                    let callee = p.callee(target)?;
                    let bound = &input.functions[callee];
                    // Reuse the existing closed-world interface thunk. The receiver's
                    // concrete class is immutable during this image's lifetime; the
                    // descriptor retains that same object, not an interface snapshot.
                    let interface_callback = crate::selection::interface_contract(input, bound)
                        && !string_dispatch.is_some_and(|targets| targets.contains_key(&callee))
                        && p.dispatch.get(&callee).is_some_and(|targets| {
                            !targets.is_empty()
                                && targets.iter().all(|(owner, implementation)| {
                                    !p.is_array_backing(*owner)
                                        && matches!(
                                            p.args[*implementation].first(),
                                            Some(Ty::Reference(_))
                                        )
                                })
                        });
                    if ((!interface_callback && (bound.is_virtual || bound.is_abstract))
                        || bound.is_override)
                        || bound.receiver_byref
                        || bound.impl_flags != 0
                        || (bound.instance
                            && !matches!(p.args[callee].first(), Some(Ty::Reference(_)))
                            && !interface_callback)
                    {
                        return Err(format!("native callback binding requires a static function, nonvirtual heap class receiver or closed class-interface dispatch: target {}, receiver {:?}, byref={}, virtual={}, override={}, abstract={}",
                            bound.name, p.args[callee].first(), bound.receiver_byref,
                            bound.is_virtual, bound.is_override, bound.is_abstract).into());
                    }
                }
                if let Op::Call(target) | Op::CallVirtual(target) | Op::Construct(target) = op {
                    if let Some(shape) = crate::selection::callable_invoke(target) {
                        if !f.out_parameters.is_empty() {
                            return Err("native callback invocation in output-parameter methods requires a later profile".into());
                        }
                        p.ty(target.owner.as_ref().unwrap())?;
                        edges[i]
                            .extend(p.callable_targets(&Type::Function(Box::new(shape.clone())))?);
                        continue;
                    }
                    let callee = p.callee(target)?;
                    let c = &input.functions[callee];
                    if !matches!(op, Op::Construct(_))
                        && c.instance
                        && c.name.ends_with("..ctor")
                        && c.owner
                            .as_ref()
                            .and_then(|t| input.type_definition(t))
                            .is_some_and(|t| t.is_reference_type)
                        && !(matches!(op, Op::Call(_))
                            && reference_constructor
                            && Some(pc) == base_initializer
                            && matches!(f.body.first(), Some(Op::Arg(0)))
                            && owner.unwrap().base == c.owner)
                    {
                        return Err(format!("reference constructor calls require newobj.ctor or the single leading base initializer: {} at {pc} calls {}", f.name, c.name).into());
                    }
                    if p.dispatch.contains_key(&callee) && !matches!(op, Op::CallVirtual(_)) {
                        return Err("interface contracts require callvirt".into());
                    }
                    if matches!(op, Op::CallVirtual(_))
                        && (!references
                            || !input.functions[callee].instance
                            || !matches!(
                                p.args[callee].first(),
                                Some(Ty::Reference(_) | Ty::Interface(_))
                            ))
                    {
                        return Err(
                            "callvirt requires an admitted nonvirtual reference member".into()
                        );
                    }
                    if matches!(op, Op::Construct(_)) {
                        let c = &input.functions[callee];
                        if c.owner.as_ref().is_some_and(|owner| {
                            input.types.iter().enumerate().any(|(i, t)| {
                                p.is_array_backing(i) && *owner == Type::Named(t.name.clone())
                            })
                        }) {
                            return Err("nominal arrays require newarr/array.reserve, not class construction".into());
                        }
                        if !c.instance || !c.name.ends_with("..ctor") || !c.no_result {
                            return Err("newobj.ctor requires a no-result value constructor".into());
                        }
                    }
                    edges[i].push(callee);
                }
            }
        }
        for (&contract, targets) in &p.dispatch {
            edges[contract].extend(targets.iter().map(|(_, target)| *target));
        }
        for (&contract, &target) in string_dispatch.into_iter().flatten() {
            let f = &input.functions[target];
            let expected = std::iter::once(Ty::Literal)
                .chain(p.args[contract].iter().skip(1).cloned())
                .collect::<Vec<_>>();
            if !p.dispatch.contains_key(&contract)
                || f.instance
                || f.owner.is_some()
                || p.args[target] != expected
                || p.results[target] != p.results[contract]
            {
                return Err("String interface target must preserve the projected receiver and contract signature".into());
            }
            edges[contract].push(target);
        }
        fn visit(
            i: usize,
            edges: &[Vec<usize>],
            states: &mut [u8],
            path: &mut Vec<usize>,
            input: &neoclr::Module,
        ) -> Result<(), Error> {
            if states[i] == 1 {
                let start = path.iter().position(|n| *n == i).unwrap_or(0);
                let cycle = path[start..]
                    .iter()
                    .copied()
                    .chain([i])
                    .map(|n| {
                        let f = &input.functions[n];
                        let name = f
                            .origin
                            .as_ref()
                            .map_or(f.name.as_str(), |o| o.name.as_str());
                        let owner = f
                            .owner
                            .as_ref()
                            .and_then(|ty| input.type_definition(ty))
                            .map(|ty| {
                                ty.origin
                                    .as_ref()
                                    .map_or(ty.name.as_str(), |o| o.name.as_str())
                            });
                        match owner {
                            Some(owner) => format!("{owner}::{name} [#{n}]"),
                            None => format!("{name} [#{n}]"),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" -> ");
                return Err(format!(
                    "recursive calls require a native stack-budget contract: {cycle}"
                )
                .into());
            }
            if states[i] == 2 {
                return Ok(());
            }
            states[i] = 1;
            path.push(i);
            for &next in &edges[i] {
                visit(next, edges, states, path, input)?;
            }
            path.pop();
            states[i] = 2;
            Ok(())
        }
        let mut states = vec![0; edges.len()];
        if !native_stack_budget {
            for i in 0..edges.len() {
                visit(i, &edges, &mut states, &mut vec![], input)?;
            }
        }
        Ok(p)
    }
    pub fn callable_targets(&self, shape: &Type) -> Result<Vec<usize>, Error> {
        let mut targets = vec![];
        for f in &self.input.functions {
            for op in &f.body {
                if let Op::BindFunction {
                    function_type,
                    target,
                } = op
                {
                    if function_type == shape {
                        let c = self.callee(target)?;
                        if !targets.contains(&c) {
                            targets.push(c);
                        }
                    }
                }
            }
        }
        // Dispatch emits branches, not a 32-entry table. The selected-function
        // budget already bounds this set; unrelated callbacks must not reject
        // otherwise valid async/TLS programs merely by sharing a signature.
        Ok(targets)
    }
    pub fn union_parts(&self, t: &Ty) -> Option<Vec<Ty>> {
        match t {
            Ty::ScalarUnion(parts) => Some(
                parts
                    .iter()
                    .map(|t| self.ty(t).expect("scalar alternative"))
                    .collect(),
            ),
            Ty::MixedUnion(parts) | Ty::ValueUnion(parts) | Ty::ReferenceUnion(parts) => {
                Some(parts.clone())
            }
            _ => None,
        }
    }

    pub fn widens_union(&self, source: &Ty, target: &Ty) -> bool {
        match (self.union_parts(source), self.union_parts(target)) {
            (Some(from), Some(to)) => from.iter().all(|part| to.contains(part)),
            _ => false,
        }
    }

    pub fn ty(&self, t: &Type) -> Result<Ty, Error> {
        Ok(match t {
            Type::Tuple(parts) => {
                Ty::Tuple(parts.iter().map(|t| self.ty(t)).collect::<Result<_, _>>()?)
            }
            Type::Intersection(parts) if self.references => {
                Ty::Intersection(parts.iter().map(|t| self.ty(t)).collect::<Result<_, _>>()?)
            }
            Type::Union(parts)
                if !parts.is_empty()
                    && parts.iter().all(|t| crate::scalar_unions::tag(t).is_some()) =>
            {
                Ty::ScalarUnion(parts.clone())
            }
            Type::Union(parts) if crate::scalar_unions::payloads(self.input, t) => {
                let parts = parts
                    .iter()
                    .map(|t| self.ty(t))
                    .collect::<Result<Vec<_>, _>>()?;
                let ty = Ty::ValueUnion(parts);
                if self.lanes(&ty) > crate::limits::INLINE_LANES {
                    return Err("value union exceeds flattened inline lane limit".into());
                }
                ty
            }
            Type::Union(parts) if self.references => {
                let parts = parts
                    .iter()
                    .map(|t| self.ty(t))
                    .collect::<Result<Vec<_>, _>>()?;
                if parts
                    .iter()
                    .any(|t| matches!(t, Ty::Int | Ty::Wide | Ty::Bool))
                    && parts
                        .iter()
                        .any(|t| matches!(t, Ty::Reference(_) | Ty::Interface(_)))
                    && parts.iter().all(|t| {
                        matches!(
                            t,
                            Ty::Int | Ty::Wide | Ty::Bool | Ty::Reference(_) | Ty::Interface(_)
                        )
                    })
                    && crate::scalar_unions::inline(self.input, t)
                {
                    return Ok(Ty::MixedUnion(parts));
                }
                if !parts
                    .iter()
                    .all(|t| matches!(t, Ty::Reference(_) | Ty::Interface(_)))
                {
                    return Err(
                        "native union payload requires tagged inline storage or intrinsic-reference lowering".into(),
                    );
                }
                Ty::ReferenceUnion(parts)
            }
            Type::Union(_) | Type::Intersection(_) => {
                return Err("structural reference payloads require --reference-arena".into());
            }
            Type::Int32 | Type::UInt32 => Ty::Int,
            Type::Byte => Ty::Byte,
            Type::SByte => Ty::SByte,
            Type::Int16 => Ty::Short,
            Type::UInt16 => Ty::UShort,
            Type::Boolean => Ty::Bool,
            Type::Void => Ty::Unit,
            Type::Value => Ty::Erased,
            Type::Function(shape) if self.references => {
                if !shape.out_parameters.is_empty()
                    || !shape.out_when_true.is_empty()
                    || shape
                        .parameters
                        .iter()
                        .chain([&shape.returns])
                        .any(|t| matches!(t, Type::ByRef(_) | Type::ReadOnlyByRef(_)))
                {
                    return Err(
                        "native callbacks do not yet admit borrowed/output signature slots".into(),
                    );
                }
                for t in shape.parameters.iter().chain([&shape.returns]) {
                    self.ty(t)?;
                }
                Ty::Callable(t.clone())
            }
            Type::ArrayRef(t) if self.references && crate::selection::scalar_array_element(t) => {
                Ty::ScalarArray(t.as_ref().clone())
            }
            Type::ArrayRef(t) if self.references && matches!(**t, Type::Function(_)) => {
                self.ty(t)?;
                Ty::CallableArray(t.as_ref().clone())
            }
            Type::ArrayRef(t)
                if self.references
                    && matches!(
                        **t,
                        Type::Tuple(_) | Type::Union(_) | Type::Intersection(_) | Type::ArrayRef(_)
                    ) =>
            {
                let element = self.ty(t)?;
                if self.lanes(&element) > crate::limits::INLINE_LANES {
                    return Err(
                        "structural array element exceeds flattened inline lane limit".into(),
                    );
                }
                Ty::StructuralArray(t.as_ref().clone())
            }
            Type::ArrayRef(t) if self.references && matches!(**t, Type::Named(_)) => {
                match self.ty(t)? {
                    Ty::Record(index) => Ty::RecordArray(index),
                    Ty::Reference(index) | Ty::Interface(index) => Ty::ReferenceArray(index),
                    _ => return Err("nominal native arrays require value records, reference classes or interfaces".into()),
                }
            }
            Type::String => Ty::Literal,
            Type::Char => Ty::Character,
            Type::IntPtr | Type::UIntPtr => Ty::Size,
            Type::RuntimeTypeHandle => Ty::TypeToken,
            Type::Int64 | Type::UInt64 => Ty::Wide,
            Type::Double => Ty::Double,
            Type::Single => Ty::Single,
            Type::Array(t) if **t == Type::Byte && self.references => Ty::ByteValues,
            Type::Array(t) if **t == Type::Char && self.references => Ty::CharacterValues,
            Type::ArrayRef(t) if **t == Type::Byte && self.references => Ty::ByteArray,
            Type::ArrayRef(t) if **t == Type::String && self.references => Ty::StringArray,
            Type::ArrayRef(t) if **t == Type::Char && self.references => Ty::CharacterArray,
            Type::Named(name) => {
                let i = self
                    .input
                    .types
                    .iter()
                    .position(|t| &t.name == name)
                    .ok_or("value profile requires a local named record")?;
                if crate::selection::static_owner(&self.input.types[i]) {
                    return Err(
                        "static owners cannot be used as values or instance receivers".into(),
                    );
                }
                if crate::memory_views::byte_memory(&self.input.types[i]) {
                    Ty::ByteMemory(i)
                } else if self.input.types[i].representation == Representation::Interface
                    && self.references
                {
                    Ty::Interface(i)
                } else if self.input.types[i].is_reference_type {
                    Ty::Reference(i)
                } else {
                    Ty::Record(i)
                }
            }
            _ => {
                return Err(
                    "unsupported value type; references and generics require later profiles".into(),
                );
            }
        })
    }
    pub fn is_object_base(&self, t: &Ty) -> bool {
        matches!(t, Ty::Reference(i) if self.object_base == Some(*i))
    }
    fn stored(&self, t: &Type) -> Result<Ty, Error> {
        let ty = self.ty(t)?;
        if matches!(ty, Ty::Tuple(_)) && self.lanes(&ty) > crate::limits::INLINE_LANES {
            return Err("structural tuple exceeds flattened inline lane limit".into());
        }
        Ok(ty)
    }
    pub fn stack_type(t: &Ty) -> Ty {
        if matches!(t, Ty::Byte | Ty::SByte | Ty::Short | Ty::UShort) {
            Ty::Int
        } else {
            t.clone()
        }
    }
    pub fn narrow_lanes(&self, t: &Ty) -> Vec<Option<(u8, bool)>> {
        match t {
            Ty::Byte => vec![Some((8, false))],
            Ty::SByte => vec![Some((8, true))],
            Ty::Short => vec![Some((16, true))],
            Ty::UShort => vec![Some((16, false))],
            Ty::Tuple(parts) => parts.iter().flat_map(|t| self.narrow_lanes(t)).collect(),
            Ty::ValueUnion(parts) => std::iter::once(None)
                .chain(parts.iter().flat_map(|t| self.narrow_lanes(t)))
                .collect(),
            Ty::Record(i) if !self.input.types[*i].fields.is_empty() => self.input.types[*i]
                .fields
                .iter()
                .flat_map(|f| self.narrow_lanes(&self.ty(&f.ty).expect("admitted field")))
                .collect(),
            _ => vec![None; self.lanes(t)],
        }
    }
    pub fn pointer_lanes(&self, t: &Ty) -> Vec<bool> {
        match t {
            Ty::ByteMemory(_) => vec![true, false, false],
            Ty::Erased | Ty::ScalarUnion(_) => vec![false, true],
            Ty::MixedUnion(_) => vec![false, true, true],
            Ty::ValueUnion(parts) => std::iter::once(false)
                .chain(parts.iter().flat_map(|t| self.pointer_lanes(t)))
                .collect(),
            Ty::PendingUnionScalar(t) => {
                self.pointer_lanes(&self.ty(t).expect("admitted union scalar"))
            }
            Ty::ScalarArray(_)
            | Ty::ReferenceArray(_)
            | Ty::RecordArray(_)
            | Ty::StructuralArray(_)
            | Ty::CallableArray(_)
            | Ty::Callable(_)
            | Ty::Literal
            | Ty::Character
            | Ty::Address(_)
            | Ty::Reference(_)
            | Ty::Interface(_)
            | Ty::Intersection(_)
            | Ty::ReferenceUnion(_)
            | Ty::ByteArray
            | Ty::CharacterArray
            | Ty::StringArray
            | Ty::CharacterValues
            | Ty::ByteValues
            | Ty::Size
            | Ty::Wide
            | Ty::TypeToken => vec![true],
            Ty::Tuple(parts) => parts.iter().flat_map(|t| self.pointer_lanes(t)).collect(),
            Ty::Record(i) if !self.input.types[*i].fields.is_empty() => self.input.types[*i]
                .fields
                .iter()
                .flat_map(|f| self.pointer_lanes(&self.ty(&f.ty).expect("admitted field")))
                .collect(),
            _ => vec![false; self.lanes(t)],
        }
    }
    pub fn lanes(&self, t: &Ty) -> usize {
        match t {
            Ty::ByteMemory(_) => 3,
            Ty::Record(i) => self.widths[*i],
            Ty::PendingUnionScalar(t) => self.lanes(&self.ty(t).expect("admitted union payload")),
            Ty::Tuple(parts) => parts.iter().map(|t| self.lanes(t)).sum(),
            Ty::Erased | Ty::ScalarUnion(_) => 2,
            Ty::MixedUnion(_) => 3,
            Ty::ValueUnion(parts) => 1 + parts.iter().map(|t| self.lanes(t)).sum::<usize>(),
            _ => 1,
        }
    }
    pub fn call_result_bytes(&self) -> u32 {
        (self
            .results
            .iter()
            .flatten()
            .map(|t| self.lanes(t))
            .chain(self.widths.iter().copied())
            .max()
            .unwrap_or(1)
            * 8)
        .max(256) as u32
    }

    pub fn bytes(&self, t: &Ty) -> u32 {
        // Private slots use eight bytes per scalar lane so mixed pointer/Int32
        // records remain aligned. This is not an external aggregate ABI.
        self.lanes(t) as u32 * 8
    }
    pub fn is_array_backing(&self, index: usize) -> bool {
        self.array_backing == Some(index) || self.reference_array_backings.contains(&index)
    }
    pub fn backing_for_array(&self, ty: &Ty) -> Option<usize> {
        if *ty == Ty::ByteArray {
            return self.array_backing;
        }
        self.reference_array_backings
            .iter()
            .copied()
            .find(|i| self.ty(&self.input.types[*i].fields[0].ty).ok().as_ref() == Some(ty))
    }
    fn derives_from(&self, actual: usize, expected: usize) -> bool {
        let mut index = actual;
        for _ in 0..self.input.types.len() {
            if index == expected {
                return true;
            }
            let Some(base) = &self.input.types[index].base else {
                return false;
            };
            let Ok(Ty::Reference(parent)) = self.ty(base) else {
                return false;
            };
            index = parent;
        }
        false
    }
    fn reference_assignable(&self, actual: &Ty, expected: &Ty) -> bool {
        if actual == expected {
            return true;
        }
        match (actual, expected) {
            (Ty::ReferenceUnion(parts), _) => parts
                .iter()
                .all(|part| self.reference_assignable(part, expected)),
            (_, Ty::ReferenceUnion(parts)) => parts
                .iter()
                .any(|part| self.reference_assignable(actual, part)),
            (_, Ty::Intersection(parts)) => parts
                .iter()
                .all(|part| self.reference_assignable(actual, part)),
            (Ty::Intersection(parts), _) => parts
                .iter()
                .any(|part| self.reference_assignable(part, expected)),
            // Interface views retain the original object reference. Every managed
            // interface reference is assignable to the verified Object base, but
            // not to arbitrary concrete classes without an explicit cast.
            (Ty::Interface(_), Ty::Reference(expected)) if self.object_base == Some(*expected) => {
                true
            }
            (
                Ty::ByteArray
                | Ty::ReferenceArray(_)
                | Ty::StructuralArray(_)
                | Ty::ScalarArray(_)
                | Ty::StringArray
                | Ty::CharacterArray,
                Ty::Reference(i),
            ) if self.backing_for_array(actual) == Some(*i) => true,
            (
                Ty::ByteArray
                | Ty::ReferenceArray(_)
                | Ty::StructuralArray(_)
                | Ty::ScalarArray(_)
                | Ty::StringArray
                | Ty::CharacterArray,
                _,
            ) if self.backing_for_array(actual).is_some() => self.reference_assignable(
                &Ty::Reference(self.backing_for_array(actual).unwrap()),
                expected,
            ),
            (Ty::Reference(actual) | Ty::Interface(actual), Ty::Interface(expected)) => {
                crate::selection::implements_interface(
                    self.input,
                    &Type::Named(self.input.types[*actual].name.clone()),
                    &Type::Named(self.input.types[*expected].name.clone()),
                )
            }
            (Ty::Reference(actual), Ty::Reference(expected)) => {
                self.derives_from(*actual, *expected)
            }
            _ => false,
        }
    }
    pub fn cast_targets(&self, target: &Ty) -> Vec<usize> {
        match target {
            Ty::Callable(_) => vec![], // Descriptor identity is checked separately from nominal tags.
            Ty::MixedUnion(parts) | Ty::ValueUnion(parts) => {
                self.cast_targets(&Ty::ReferenceUnion(parts.clone()))
            }
            Ty::ReferenceUnion(_) => self
                .input
                .types
                .iter()
                .enumerate()
                .filter(|(index, t)| {
                    t.is_reference_type
                        && t.representation == Representation::Record
                        && !t.is_abstract
                        && self.reference_assignable(&Ty::Reference(*index), target)
                })
                .map(|(index, _)| index)
                .collect(),
            Ty::Intersection(parts) => self
                .input
                .types
                .iter()
                .enumerate()
                .filter(|(index, t)| {
                    t.is_reference_type
                        && t.representation == Representation::Record
                        && !t.is_abstract
                        && parts
                            .iter()
                            .all(|part| self.reference_assignable(&Ty::Reference(*index), part))
                })
                .map(|(index, _)| index)
                .collect(),
            Ty::ByteArray
            | Ty::ReferenceArray(_)
            | Ty::StructuralArray(_)
            | Ty::ScalarArray(_)
            | Ty::StringArray
            | Ty::CharacterArray => self.backing_for_array(target).into_iter().collect(),
            Ty::Reference(i) if self.object_base == Some(*i) => self
                .input
                .types
                .iter()
                .enumerate()
                .filter(|(_, t)| t.representation == Representation::Record && !t.is_abstract)
                .map(|(i, _)| i)
                .collect(),
            Ty::Reference(i) => self
                .input
                .types
                .iter()
                .enumerate()
                .filter(|(n, t)| {
                    t.representation == Representation::Record
                        && !t.is_abstract
                        && self.derives_from(*n, *i)
                })
                .map(|(n, _)| n)
                .collect(),
            Ty::Interface(i) => {
                let target = Type::Named(self.input.types[*i].name.clone());
                self.input
                    .types
                    .iter()
                    .enumerate()
                    .filter(|(_, t)| {
                        t.representation == Representation::Record
                            && !t.is_abstract
                            && crate::selection::implements_interface(
                                self.input,
                                &Type::Named(t.name.clone()),
                                &target,
                            )
                    })
                    .map(|(i, _)| i)
                    .collect()
            }
            _ => unreachable!("admitted reference target"),
        }
    }
    pub fn object_bytes(&self, index: usize) -> u32 {
        8 + self.widths[index] as u32 * 8
    }
    pub fn static_field(
        &self,
        owner: &Type,
        index: usize,
    ) -> Result<(usize, Ty, usize, usize), Error> {
        let i = self
            .input
            .types
            .iter()
            .position(|t| Some(t.name.as_str()) == owner.definition_name())
            .ok_or("missing closed static owner")?;
        let fields = &self.input.types[i].static_fields;
        let mut offset = 0;
        let mut selected = None;
        for (n, field) in fields.iter().enumerate() {
            let ty = self.ty(&field.ty)?;
            fn zero_initializable(p: &Profile<'_>, ty: &Ty) -> bool {
                match ty {
                    Ty::Tuple(parts) => parts.iter().all(|t| zero_initializable(p, t)),
                    Ty::Record(i) => p.input.types[*i]
                        .fields
                        .iter()
                        .all(|f| p.ty(&f.ty).is_ok_and(|t| zero_initializable(p, &t))),
                    Ty::Erased
                    | Ty::ByteValues
                    | Ty::CharacterValues
                    | Ty::Character
                    | Ty::Unit
                    | Ty::TypeToken
                    | Ty::Address(_) => false,
                    _ => true,
                }
            }
            if !zero_initializable(self, &ty) {
                return Err("native static storage requires a zero-initializable field".into());
            }
            if n == index {
                selected = Some((ty.clone(), offset));
            }
            offset += self.lanes(&ty);
        }
        if offset > 256 {
            return Err("native static storage exceeds object lane limit".into());
        }
        let (ty, slot) = selected.ok_or("static field index out of range")?;
        Ok((i, ty, 8 + slot * 8, 8 + offset * 8))
    }
    pub fn field_offset(&self, t: &Ty, index: usize) -> usize {
        let t = if let Ty::Address(t) = t {
            t.as_ref()
        } else {
            t
        };
        if let Ty::Tuple(parts) = t {
            return parts[..index].iter().map(|t| self.lanes(t)).sum();
        }
        let (Ty::Record(owner) | Ty::Reference(owner)) = t else {
            unreachable!("checked field owner")
        };
        self.layouts[*owner][index]
    }
    pub fn field(&self, t: &Ty, index: usize) -> Result<Ty, Error> {
        if matches!(t, Ty::Address(inner) if matches!(**inner, Ty::Reference(_))) {
            return Err("load a borrowed reference slot before accessing object fields".into());
        }
        let t = if let Ty::Address(t) = t {
            t.as_ref()
        } else {
            t
        };
        if let Ty::Tuple(parts) = t {
            return parts
                .get(index)
                .cloned()
                .ok_or_else(|| "tuple element index out of range".into());
        }
        let (Ty::Record(owner) | Ty::Reference(owner)) = t else {
            return Err("field access requires a value record".into());
        };
        let mut owner = *owner;
        let mut index = index;
        for _ in 0..self.input.types.len() {
            if let Some(base) = &self.input.types[owner].base {
                let Ty::Reference(parent) = self.ty(base)? else {
                    return Err("invalid reference base".into());
                };
                let inherited = self.layouts[parent].len();
                if index < inherited {
                    owner = parent;
                    continue;
                }
                index -= inherited;
            }
            return self.ty(&self.input.types[owner]
                .fields
                .get(index)
                .ok_or("field index out of range")?
                .ty);
        }
        Err("cyclic reference field hierarchy".into())
    }
    pub fn callee(&self, target: &FunctionRef) -> Result<usize, Error> {
        if !target.generic_arguments.is_empty() {
            return Err("generic value calls remain unsupported".into());
        }
        let candidates = self
            .names
            .get(target.name.as_str())
            .ok_or("value profile does not support external calls")?;
        let mut matches = candidates.iter().copied().filter(|&i| {
            let f = &self.input.functions[i];
            f.owner == target.owner
                && f.instance == target.instance
                && f.parameters == target.parameters
                && target
                    .definition
                    .as_ref()
                    .is_none_or(|id| Some(id) == f.definition.as_ref())
        });
        let i = matches
            .next()
            .ok_or("value member call identity/signature mismatch")?;
        if matches.next().is_some() {
            return Err("ambiguous value call requires a matching definition identity".into());
        }
        Ok(i)
    }
    pub fn root(&self, root: &str, bootstrap: bool) -> Result<usize, Error> {
        let candidates = self.names.get(root).ok_or("root function not found")?;
        let [i] = candidates.as_slice() else {
            return Err("ambiguous value root name; use a uniquely named wrapper".into());
        };
        let i = *i;
        let f = &self.input.functions[i];
        if bootstrap {
            let valid = matches!(&f.returns, Type::Function(shape) if shape.parameters.is_empty()
                && shape.returns == Type::Void
                && shape.out_parameters.is_empty() && shape.out_when_true.is_empty());
            if f.instance
                || f.no_result
                || !f.out_parameters.is_empty()
                || !valid
                || !(self.args[i].is_empty() || self.args[i] == [Ty::Int])
            {
                return Err("native host bootstrap must be static with zero arguments or one Int32 argument and a zero-argument unit callback result".into());
            }
            return Ok(i);
        }
        if f.instance
            || !matches!(self.results[i], None | Some(Ty::Int | Ty::Unit))
            || !(self.args[i].is_empty()
                || self.args[i] == [Ty::Int]
                || self.args[i] == [Ty::StringArray])
            || !f.out_parameters.is_empty()
        {
            return Err("value root must be static with zero arguments, one Int32 or one String[] argument and Int32, Void or no result".into());
        }
        Ok(i)
    }
    // The interpreter checks callee output assignment dynamically. Native code has
    // no such runtime, so require a whole-slot write on every normal return path.
    fn verify_outputs(&self, i: usize, shapes: &Stacks) -> Result<(), Error> {
        let f = &self.input.functions[i];
        if f.out_parameters.is_empty() {
            return Ok(());
        }
        #[derive(Clone, PartialEq, Eq)]
        struct State {
            stack: Vec<Option<usize>>,
            assigned: Vec<bool>,
        }
        let mut states = vec![None; f.body.len()];
        states[0] = Some(State {
            stack: vec![],
            assigned: vec![false; f.parameters.len()],
        });
        let mut work = VecDeque::from([0]);
        let offset = usize::from(f.instance);
        while let Some(pc) = work.pop_front() {
            let mut state = states[pc].clone().unwrap();
            let op = &f.body[pc];
            let fail = || -> Error {
                format!(
                    "{} instruction {pc}: output requires a definite whole-slot assignment",
                    f.name
                )
                .into()
            };
            let readable = |origin: Option<usize>, assigned: &[bool]| -> Result<(), Error> {
                if origin.is_some_and(|n| !assigned[n]) {
                    Err(fail())
                } else {
                    Ok(())
                }
            };
            match op {
                Op::Arg(n) => state.stack.push(
                    n.checked_sub(offset)
                        .filter(|n| f.out_parameters.contains(n)),
                ),
                _ if crate::checked_conversion::spec(op).is_some() => (),
                Op::ConvertFloat32
                | Op::ConvertFloat64
                | Op::ConvertFloatUnsigned
                | Op::ConvertInt32
                | Op::ConvertUInt32
                | Op::ConvertUInt8
                | Op::ConvertInt8
                | Op::ConvertInt16
                | Op::ConvertUInt16
                | Op::ConvertInt64
                | Op::ConvertUInt64
                | Op::ConvertNativeInt
                | Op::ConvertNativeUInt => (),
                Op::Negate
                | Op::BitNot
                | Op::BoxValue(_)
                | Op::UnboxAny(_)
                | Op::PackValue(_)
                | Op::IsValue(_)
                | Op::UnpackValue(_)
                | Op::IsInstance(_)
                | Op::CastClass(_)
                | Op::ReferenceIsNull
                | Op::NewArray(_)
                | Op::ReserveArray(_)
                | Op::ArrayLength => {
                    readable(state.stack.pop().unwrap(), &state.assigned)?;
                    state.stack.push(None);
                }
                Op::Dup => state.stack.push(*state.stack.last().unwrap()),
                Op::StoreObject(_) => {
                    state.stack.pop();
                    if let Some(n) = state.stack.pop().unwrap() {
                        state.assigned[n] = true;
                    }
                }
                Op::InitializeObject(_) => {
                    if let Some(n) = state.stack.pop().unwrap() {
                        state.assigned[n] = true;
                    }
                }
                Op::StaticField { .. } => state.stack.push(None),
                Op::SetStaticField { .. } => {
                    readable(state.stack.pop().unwrap(), &state.assigned)?;
                }
                Op::Field(_) | Op::FieldAddress(_) | Op::LoadObject(_) => {
                    readable(state.stack.pop().unwrap(), &state.assigned)?;
                    state.stack.push(None);
                }
                Op::SetField(_) => {
                    state.stack.pop();
                    // Partial construction is deliberately outside this proof.
                    readable(state.stack.pop().unwrap(), &state.assigned)?;
                    let shape = shapes[pc].as_ref().unwrap();
                    if !matches!(shape[shape.len() - 2], Ty::Reference(_)) {
                        state.stack.push(None);
                    }
                }
                Op::ArrayElement(_) | Op::ArrayAddress(_) | Op::StoreArrayElement(_) => {
                    if matches!(op, Op::StoreArrayElement(_)) {
                        state.stack.pop();
                    }
                    state.stack.pop();
                    readable(state.stack.pop().unwrap(), &state.assigned)?;
                    if !matches!(op, Op::StoreArrayElement(_)) {
                        state.stack.push(None);
                    }
                }
                Op::Call(target) | Op::CallVirtual(target) | Op::Construct(target) => {
                    let c = self.callee(target)?;
                    let callee = &self.input.functions[c];
                    let construct = matches!(op, Op::Construct(_));
                    let count = self.args[c].len() - usize::from(construct);
                    let passed = state.stack.split_off(state.stack.len() - count);
                    let receiver = usize::from(callee.instance && !construct);
                    // Check all input borrows before marking any output assigned.
                    for (n, origin) in passed.iter().enumerate() {
                        if n.checked_sub(receiver)
                            .is_none_or(|n| !callee.out_parameters.contains(&n))
                        {
                            readable(*origin, &state.assigned)?;
                        }
                    }
                    for &n in &callee.out_parameters {
                        if let Some(origin) = passed[n + receiver] {
                            state.assigned[origin] = true;
                        }
                    }
                    if construct || self.results[c].is_some() {
                        state.stack.push(None);
                    }
                }
                Op::Fault(_) => continue,
                Op::Return => {
                    if f.out_parameters.iter().any(|n| !state.assigned[*n]) {
                        return Err(fail());
                    }
                    continue;
                }
                Op::New(t) => {
                    let count = match self.ty(t)? {
                        Ty::Record(owner) => self.input.types[owner].fields.len(),
                        Ty::Tuple(parts) => parts.len(),
                        _ => unreachable!(),
                    };
                    state.stack.truncate(state.stack.len() - count);
                    state.stack.push(None);
                }
                Op::Pop
                | Op::Store(_)
                | Op::StoreArg(_)
                | Op::BranchTrue(_)
                | Op::BranchFalse(_) => {
                    state.stack.pop();
                }
                Op::Branch(_) => (),
                Op::Int(_)
                | Op::Int64(_)
                | Op::Float64 { .. }
                | Op::Float32 { .. }
                | Op::String(_)
                | Op::Bool(_)
                | Op::Void
                | Op::LoadTypeToken(_)
                | Op::LoadMetadataToken(_)
                | Op::Load(_)
                | Op::LocalAddress(_)
                | Op::ArgumentAddress(_) => state.stack.push(None),
                _ => {
                    // All remaining admitted instructions are binary scalar operations.
                    state.stack.pop();
                    state.stack.pop();
                    if flow::comparison(op).is_none() {
                        state.stack.push(None);
                    }
                }
            }
            let successors = match op {
                Op::Branch(n) => vec![*n],
                Op::BranchTrue(n) | Op::BranchFalse(n) => vec![*n, pc + 1],
                _ if flow::comparison(op).is_some() => {
                    vec![flow::comparison(op).unwrap().0, pc + 1]
                }
                _ => vec![pc + 1],
            };
            for next in successors {
                debug_assert_eq!(state.stack.len(), shapes[next].as_ref().unwrap().len());
                if let Some(old) = &mut states[next] {
                    // Do not erase the identity of an output borrow at a CFG join.
                    if old.stack != state.stack {
                        return Err(fail());
                    }
                    let joined: Vec<_> = old
                        .assigned
                        .iter()
                        .zip(&state.assigned)
                        .map(|(a, b)| *a && *b)
                        .collect();
                    if joined != old.assigned {
                        old.assigned = joined;
                        work.push_back(next);
                    }
                } else {
                    states[next] = Some(state.clone());
                    work.push_back(next);
                }
            }
        }
        Ok(())
    }
    pub fn analyze(&self, i: usize) -> Result<Stacks, Error> {
        let f = &self.input.functions[i];
        let fail =
            |pc, message| -> Error { format!("{} instruction {pc}: {message}", f.name).into() };
        if self.dispatch.contains_key(&i) {
            return Ok(vec![]);
        }
        if f.body.is_empty() {
            return Err(fail(0, "empty body"));
        }
        // Admission is independent of reachability. Never silently trim unsupported IL.
        for (pc, op) in f.body.iter().enumerate() {
            match op {
                Op::IsInstance(t) | Op::UnboxAny(t)
                    if crate::scalar_unions::value_only(self.input, t) =>
                {
                    ()
                }
                Op::IsInstance(t) if crate::scalar_unions::inline(self.input, t) => {
                    return Err(format!("{} instruction {pc}: scalar union instance tests require tagged conversion lowering", f.name).into());
                }
                Op::BoxValue(_) if crate::scalar_unions::box_pair(self.input, f, pc) => (),
                Op::UnboxAny(t) | Op::IsInstance(t)
                    if crate::scalar_unions::tag(t).is_some()
                        || crate::scalar_unions::bank_scalar(t)
                        || crate::scalar_unions::record(self.input, t) =>
                {
                    ()
                }
                Op::CastClass(t) if crate::scalar_unions::inline(self.input, t) => (),
                Op::Arg(n) | Op::ArgumentAddress(n) | Op::StoreArg(n)
                    if *n < self.args[i].len() =>
                {
                    if matches!(op, Op::ArgumentAddress(_) | Op::StoreArg(_))
                        && matches!(self.args[i][*n], Ty::Address(_))
                    {
                        return Err(fail(pc, "cannot address or replace a borrowed parameter"));
                    }
                }
                Op::Load(n) | Op::Store(n) | Op::LocalAddress(n) if *n < self.locals[i].len() => (),
                _ if crate::checked_conversion::spec(op).is_some() => (),
                Op::Int(_)
                | Op::Int64(_)
                | Op::Float64 { .. }
                | Op::Float32 { .. }
                | Op::String(_)
                | Op::ConvertFloat32
                | Op::ConvertFloat64
                | Op::ConvertFloatUnsigned
                | Op::ConvertInt32
                | Op::ConvertUInt32
                | Op::ConvertInt64
                | Op::ConvertUInt64
                | Op::ConvertNativeInt
                | Op::ConvertNativeUInt
                | Op::ConvertUInt8
                | Op::ConvertInt8
                | Op::ConvertInt16
                | Op::ConvertUInt16
                | Op::Bool(_)
                | Op::Void
                | Op::LoadTypeToken(_)
                | Op::LoadMetadataToken(_)
                | Op::Dup
                | Op::Pop
                | Op::StaticField { .. }
                | Op::SetStaticField { .. }
                | Op::Field(_)
                | Op::SetField(_)
                | Op::FieldAddress(_)
                | Op::Call(_)
                | Op::CallVirtual(_)
                | Op::Construct(_)
                | Op::Return
                | Op::Fault(_)
                | Op::Branch(_)
                | Op::BranchTrue(_)
                | Op::BranchFalse(_)
                | Op::Equal
                | Op::Greater
                | Op::Less
                | Op::GreaterUnsigned
                | Op::LessUnsigned
                | Op::BitAnd
                | Op::BitOr
                | Op::BitXor
                | Op::BitNot
                | Op::Negate
                | Op::ShiftLeft
                | Op::ShiftRight
                | Op::ShiftRightUnsigned
                | Op::Add
                | Op::Sub
                | Op::Mul
                | Op::AddChecked
                | Op::SubChecked
                | Op::MulChecked
                | Op::AddCheckedUnsigned
                | Op::SubCheckedUnsigned
                | Op::MulCheckedUnsigned
                | Op::Divide
                | Op::DivideUnsigned
                | Op::Remainder
                | Op::RemainderUnsigned => (),
                Op::BindFunction { .. } if self.references => (),
                Op::IsInstance(Type::String)
                | Op::CastClass(Type::String)
                | Op::ReferenceIsNull
                | Op::ReferenceEqual => (),
                Op::IsInstance(t @ Type::ArrayRef(_)) | Op::CastClass(t @ Type::ArrayRef(_))
                    if self.backing_for_array(&self.ty(t)?).is_none() =>
                {
                    return Err(fail(pc, "array casts require verified backing"));
                }
                Op::IsInstance(t) | Op::CastClass(t)
                    if self.references
                        && (matches!(
                            self.ty(t)?,
                            Ty::Reference(_)
                                | Ty::Interface(_)
                                | Ty::Intersection(_)
                                | Ty::ReferenceUnion(_)
                                | Ty::Callable(_)
                        ) || self.backing_for_array(&self.ty(t)?).is_some()) =>
                {
                    ()
                }
                Op::NewArray(t)
                | Op::ReserveArray(t)
                | Op::ArrayElement(t)
                | Op::StoreArrayElement(t)
                    if self.references && crate::selection::scalar_array_element(t) =>
                {
                    ()
                }
                Op::NewArray(Type::String | Type::Char)
                | Op::ReserveArray(Type::String | Type::Char)
                | Op::ArrayElement(Type::String | Type::Char)
                | Op::StoreArrayElement(Type::String | Type::Char)
                    if self.references =>
                {
                    ()
                }
                Op::NewArray(t)
                | Op::ReserveArray(t)
                | Op::ArrayElement(t)
                | Op::StoreArrayElement(t)
                    if self.references && matches!(t, Type::Function(_)) =>
                {
                    self.ty(t)?;
                }
                Op::NewArray(Type::Byte)
                | Op::ReserveArray(Type::Byte)
                | Op::ArrayElement(Type::Byte)
                | Op::StoreArrayElement(Type::Byte)
                | Op::ArrayAddress(Type::Byte)
                | Op::ArrayLength
                    if self.references =>
                {
                    ()
                }
                Op::NewArray(t) if self.references && matches!(t, Type::ArrayRef(_)) => {
                    self.ty(t)?;
                }
                Op::NewArray(t)
                    if self.references
                        && matches!(t, Type::Named(_))
                        && matches!(self.ty(t)?, Ty::Reference(_) | Ty::Interface(_)) =>
                {
                    ()
                }
                Op::ReserveArray(t) | Op::ArrayElement(t) | Op::StoreArrayElement(t)
                    if self.references
                        && matches!(t, Type::Named(_))
                        && matches!(
                            self.ty(t)?,
                            Ty::Record(_) | Ty::Reference(_) | Ty::Interface(_)
                        ) =>
                {
                    ()
                }
                Op::ReserveArray(t) | Op::ArrayElement(t) | Op::StoreArrayElement(t)
                    if self.references
                        && matches!(
                            t,
                            Type::Tuple(_)
                                | Type::Union(_)
                                | Type::Intersection(_)
                                | Type::ArrayRef(_)
                        ) =>
                {
                    self.ty(&Type::ArrayRef(Box::new(t.clone())))?;
                }
                Op::PackValue(t) | Op::IsValue(t) | Op::UnpackValue(t) => {
                    erased_tag(t)?;
                }
                Op::InitializeObject(Type::Array(t)) if **t == Type::Byte => {
                    return Err(fail(
                        pc,
                        "byte value arrays require an explicit native producer; default initialization is unsupported",
                    ));
                }
                Op::InitializeObject(Type::Value) => {
                    return Err(fail(pc, "Value has no default initialization"));
                }
                Op::InitializeObject(t) | Op::LoadObject(t) | Op::StoreObject(t) | Op::New(t) => {
                    self.stored(t)?;
                }
                _ if flow::comparison(op).is_some() => (),
                _ => return Err(fail(pc, &format!("unsupported value instruction: {op:?}"))),
            }
        }
        let mut stacks: Stacks = vec![None; f.body.len()];
        stacks[0] = Some(vec![]);
        let mut work = VecDeque::from([0]);
        while let Some(pc) = work.pop_front() {
            let mut stack = stacks[pc].as_ref().unwrap().clone();
            let op = &f.body[pc];
            let pop = |s: &mut Vec<Ty>| s.pop().ok_or_else(|| fail(pc, "stack underflow"));
            let take = |s: &mut Vec<Ty>, expected: &Ty| -> Result<(), Error> {
                let actual = pop(s)?;
                let expected = Self::stack_type(expected);
                if actual != expected && !self.reference_assignable(&actual, &expected) {
                    return Err(format!("{} instruction {pc}: value operand type mismatch: expected {expected:?}, found {actual:?} for {op:?}", f.name).into());
                }
                Ok(())
            };
            match op {
                Op::BoxValue(t) => {
                    take(&mut stack, &self.ty(t)?)?;
                    stack.push(Ty::PendingUnionScalar(t.clone()));
                }
                Op::CastClass(t @ Type::Union(parts))
                    if crate::scalar_unions::inline(self.input, t) =>
                {
                    let source = pop(&mut stack)?;
                    if matches!(self.ty(t)?, Ty::MixedUnion(_) | Ty::ValueUnion(_))
                        && self.is_object_base(&source)
                    {
                        return Err(fail(
                            pc,
                            "mixed union injection from Object requires boxed-payload classification",
                        ));
                    }
                    if let Ty::Interface(index) = &source {
                        let view = Type::Named(self.input.types[*index].name.clone());
                        if parts
                            .iter()
                            .filter(|part| {
                                crate::scalar_unions::tag(part).is_some()
                                    || crate::scalar_unions::bank_scalar(part)
                                    || crate::scalar_unions::record(self.input, part)
                            })
                            .any(|part| {
                                neoclr::native_metadata::assignable(self.input, part, &view)
                                    .unwrap_or(true)
                            })
                        {
                            return Err(fail(
                                pc,
                                "mixed union injection from a scalar interface requires boxed-payload classification",
                            ));
                        }
                    }
                    if !matches!(&source, Ty::PendingUnionScalar(scalar) if parts.contains(scalar))
                        && source != self.ty(t)?
                        && !self.widens_union(&source, &self.ty(t)?)
                        && !(matches!(source, Ty::Callable(_) | Ty::Literal)
                            && parts
                                .iter()
                                .any(|part| self.ty(part).ok().as_ref() == Some(&source)))
                        && !(matches!(self.ty(t)?, Ty::MixedUnion(_) | Ty::ValueUnion(_))
                            && matches!(source, Ty::Reference(_) | Ty::Interface(_))
                            && parts.iter().any(|part| {
                                self.input.type_definition(part).is_some_and(|d| {
                                    d.is_reference_type
                                        || d.representation == Representation::Interface
                                })
                            }))
                    {
                        return Err(fail(
                            pc,
                            "scalar union injection requires an exact adjacent scalar box or the same union",
                        ));
                    }
                    stack.push(self.ty(t)?);
                }
                Op::IsInstance(t) | Op::UnboxAny(t)
                    if crate::scalar_unions::value_only(self.input, t) =>
                {
                    if pop(&mut stack)? != self.ty(t)? {
                        return Err(fail(
                            pc,
                            "value union identity query requires the same union",
                        ));
                    }
                    if matches!(op, Op::IsInstance(_)) {
                        if !matches!(f.body.get(pc + 1), Some(Op::ReferenceIsNull)) {
                            return Err(fail(
                                pc,
                                "value union identity test requires immediate null test",
                            ));
                        }
                        stack.push(Ty::ScalarUnionTest);
                    } else {
                        stack.push(self.ty(t)?);
                    }
                }
                Op::IsInstance(t)
                    if crate::scalar_unions::tag(t).is_some()
                        || crate::scalar_unions::bank_scalar(t)
                        || crate::scalar_unions::record(self.input, t) =>
                {
                    if (crate::scalar_unions::record(self.input, t)
                        || crate::scalar_unions::bank_scalar(t))
                        && !matches!(stack.last(), Some(Ty::ValueUnion(_)))
                    {
                        return Err(fail(
                            pc,
                            "nominal value query requires an inline value union",
                        ));
                    }
                    if !matches!(
                        pop(&mut stack)?,
                        Ty::ScalarUnion(_) | Ty::MixedUnion(_) | Ty::ValueUnion(_)
                    ) || !matches!(f.body.get(pc + 1), Some(Op::ReferenceIsNull))
                    {
                        return Err(fail(
                            pc,
                            "scalar union type test requires an inline union and immediate null test",
                        ));
                    }
                    stack.push(Ty::ScalarUnionTest);
                }
                Op::UnboxAny(t) => {
                    if (crate::scalar_unions::record(self.input, t)
                        || crate::scalar_unions::bank_scalar(t))
                        && !matches!(stack.last(), Some(Ty::ValueUnion(_)))
                    {
                        return Err(fail(
                            pc,
                            "nominal value query requires an inline value union",
                        ));
                    }
                    if !matches!(
                        pop(&mut stack)?,
                        Ty::ScalarUnion(_) | Ty::MixedUnion(_) | Ty::ValueUnion(_)
                    ) {
                        return Err(fail(pc, "inline union extraction requires a scalar union"));
                    }
                    stack.push(self.ty(t)?);
                }
                Op::Int(_) => stack.push(Ty::Int),
                Op::Int64(_) => stack.push(Ty::Wide),
                Op::Float64 { .. } => stack.push(Ty::Double),
                Op::Float32 { .. } => stack.push(Ty::Single),
                _ if crate::checked_conversion::spec(op).is_some() => {
                    let source = pop(&mut stack)?;
                    if !matches!(
                        source,
                        Ty::Int | Ty::Wide | Ty::Size | Ty::Single | Ty::Double
                    ) {
                        return Err(fail(pc, "checked conversion requires a numeric operand"));
                    }
                    stack.push(match op {
                        Op::CheckedNativeInt
                        | Op::CheckedNativeUInt
                        | Op::CheckedNativeIntUnsigned
                        | Op::CheckedNativeUIntUnsigned => Ty::Size,
                        _ if crate::checked_conversion::spec(op).unwrap().0 == 64 => Ty::Wide,
                        _ => Ty::Int,
                    });
                }
                Op::ConvertFloat32 | Op::ConvertFloat64 | Op::ConvertFloatUnsigned => {
                    let source = pop(&mut stack)?;
                    if !matches!(
                        source,
                        Ty::Int | Ty::Wide | Ty::Size | Ty::Single | Ty::Double
                    ) || matches!(op, Op::ConvertFloatUnsigned)
                        && matches!(source, Ty::Single | Ty::Double)
                    {
                        return Err(fail(pc, "floating conversion requires a numeric operand"));
                    }
                    stack.push(if matches!(op, Op::ConvertFloat32) {
                        Ty::Single
                    } else {
                        Ty::Double
                    });
                }
                Op::String(_) => stack.push(Ty::Literal),
                Op::ConvertInt32
                | Op::ConvertUInt32
                | Op::ConvertUInt8
                | Op::ConvertInt8
                | Op::ConvertInt16
                | Op::ConvertUInt16
                | Op::ConvertInt64
                | Op::ConvertUInt64
                | Op::ConvertNativeInt
                | Op::ConvertNativeUInt => {
                    let source = pop(&mut stack)?;
                    let enum_storage = matches!(op, Op::ConvertInt32)
                        && matches!(source, Ty::Record(n) if self.input.types[n].enum_info.is_some());
                    if !matches!(source, Ty::Int | Ty::Size | Ty::Wide) && !enum_storage {
                        return Err(fail(
                            pc,
                            "conversion requires an integer stack category or enum-to-Int32",
                        ));
                    }
                    stack.push(match op {
                        Op::ConvertInt64 | Op::ConvertUInt64 => Ty::Wide,
                        Op::ConvertNativeInt | Op::ConvertNativeUInt => Ty::Size,
                        _ => Ty::Int,
                    });
                }
                Op::Bool(_) => stack.push(Ty::Bool),
                Op::Void => stack.push(Ty::Unit),
                Op::LoadMetadataToken(index) => {
                    neoclr::native_metadata::descriptor(self.input, *index)
                        .map_err(|e| e.to_string())?;
                    stack.push(Ty::TypeToken);
                }
                Op::LoadTypeToken(t) => {
                    // A static owner has a type identity even though it has no
                    // instance storage. Keep ordinary value checks for every other shape.
                    let static_owner = matches!(t, Type::Named(name) if self.input.types.iter()
                        .any(|definition| definition.name == *name && crate::selection::static_owner(definition)));
                    if !static_owner {
                        self.ty(t)?;
                    }
                    stack.push(Ty::TypeToken);
                }
                Op::PackValue(t) => {
                    take(&mut stack, &self.ty(t)?)?;
                    stack.push(Ty::Erased);
                }
                Op::IsValue(_) => {
                    take(&mut stack, &Ty::Erased)?;
                    stack.push(Ty::Bool);
                }
                Op::UnpackValue(t) => {
                    take(&mut stack, &Ty::Erased)?;
                    stack.push(Self::stack_type(&self.ty(t)?));
                }
                Op::Arg(n) => stack.push(Self::stack_type(&self.args[i][*n])),
                Op::Load(n) => stack.push(Self::stack_type(&self.locals[i][*n])),
                Op::ArgumentAddress(n) => stack.push(self.args[i][*n].clone().address()),
                Op::LocalAddress(n) => stack.push(self.locals[i][*n].clone().address()),
                Op::StoreArg(n) => take(&mut stack, &self.args[i][*n])?,
                Op::Store(n) => take(&mut stack, &self.locals[i][*n])?,
                Op::Dup => {
                    let t = pop(&mut stack)?;
                    stack.extend([t.clone(), t]);
                }
                Op::Pop => {
                    pop(&mut stack)?;
                }
                Op::InitializeObject(t) => take(&mut stack, &self.ty(t)?.address())?,
                Op::LoadObject(t) => {
                    let t = self.ty(t)?;
                    take(&mut stack, &t.clone().address())?;
                    stack.push(Self::stack_type(&t));
                }
                Op::StoreObject(t) => {
                    let t = self.ty(t)?;
                    take(&mut stack, &t)?;
                    take(&mut stack, &t.address())?;
                }
                Op::New(t) => {
                    let t = self.ty(t)?;
                    let fields = match &t {
                        Ty::Record(owner) => self.input.types[*owner]
                            .fields
                            .iter()
                            .map(|f| self.ty(&f.ty))
                            .collect::<Result<Vec<_>, _>>()?,
                        Ty::Tuple(parts) => parts.clone(),
                        _ => return Err(fail(pc, "reference allocation requires a constructor")),
                    };
                    for field in fields.iter().rev() {
                        take(&mut stack, field)?;
                    }
                    stack.push(t);
                }
                Op::IsInstance(Type::String) | Op::CastClass(Type::String) => {
                    let source = pop(&mut stack)?;
                    if source != Ty::Literal
                        && !matches!(source, Ty::ValueUnion(_))
                        && !(self.references
                            && matches!(
                                source,
                                Ty::Reference(_)
                                    | Ty::Interface(_)
                                    | Ty::Intersection(_)
                                    | Ty::ReferenceUnion(_)
                                    | Ty::ByteArray
                                    | Ty::ReferenceArray(_)
                                    | Ty::StructuralArray(_)
                                    | Ty::ScalarArray(_)
                                    | Ty::StringArray
                                    | Ty::CharacterArray
                            ))
                    {
                        return Err(fail(pc, "String casts require text or managed references"));
                    }
                    stack.push(Ty::Literal);
                }
                Op::IsInstance(t @ Type::Function(_)) | Op::CastClass(t @ Type::Function(_))
                    if matches!(stack.last(), Some(Ty::ValueUnion(_))) =>
                {
                    pop(&mut stack)?;
                    stack.push(self.ty(t)?);
                }
                Op::IsInstance(t) | Op::CastClass(t) => {
                    if matches!(op, Op::IsInstance(_)) && matches!(self.ty(t)?, Ty::ValueUnion(_)) {
                        return Err(fail(
                            pc,
                            "value union target tests require structural result lowering",
                        ));
                    }
                    let source = pop(&mut stack)?;
                    if let Ty::MixedUnion(parts) | Ty::ValueUnion(parts) = &source {
                        let needs_box = self.is_object_base(&self.ty(t)?)
                            || parts
                                .iter()
                                .filter_map(|part| match part {
                                    Ty::Int => Some(Type::Int32),
                                    Ty::Wide => Some(Type::Int64),
                                    Ty::Bool => Some(Type::Boolean),
                                    Ty::Record(index) => {
                                        Some(Type::Named(self.input.types[*index].name.clone()))
                                    }
                                    _ => None,
                                })
                                .any(|scalar| {
                                    neoclr::native_metadata::assignable(self.input, &scalar, t)
                                        .unwrap_or(true)
                                });
                        if needs_box {
                            return Err(fail(
                                pc,
                                "mixed union Object or scalar-interface views require boxing",
                            ));
                        }
                    }
                    if matches!(source, Ty::MixedUnion(_) | Ty::ValueUnion(_))
                        && !matches!(self.ty(t)?, Ty::Reference(_) | Ty::Interface(_))
                    {
                        return Err(fail(
                            pc,
                            "mixed union extraction requires a nominal reference view",
                        ));
                    }
                    if !matches!(
                        source,
                        Ty::ValueUnion(_)
                            | Ty::MixedUnion(_)
                            | Ty::Reference(_)
                            | Ty::Interface(_)
                            | Ty::Intersection(_)
                            | Ty::ReferenceUnion(_)
                            | Ty::Callable(_)
                    ) && !(source == Ty::Literal && self.references)
                        && self.backing_for_array(&source).is_none()
                    {
                        return Err(fail(
                            pc,
                            "reference casts require a class, interface or verified array view",
                        ));
                    }
                    stack.push(self.ty(t)?);
                }
                Op::ReferenceIsNull => {
                    if !matches!(
                        pop(&mut stack)?,
                        Ty::ScalarUnion(_)
                            | Ty::ValueUnion(_)
                            | Ty::MixedUnion(_)
                            | Ty::ScalarUnionTest
                            | Ty::ScalarArray(_)
                            | Ty::ReferenceArray(_)
                            | Ty::RecordArray(_)
                            | Ty::StructuralArray(_)
                            | Ty::Callable(_)
                            | Ty::CallableArray(_)
                            | Ty::Literal
                            | Ty::Reference(_)
                            | Ty::Interface(_)
                            | Ty::Intersection(_)
                            | Ty::ReferenceUnion(_)
                            | Ty::ByteArray
                            | Ty::CharacterArray
                            | Ty::StringArray
                    ) {
                        return Err(fail(pc, "null test requires text or reference"));
                    }
                    stack.push(Ty::Bool);
                }
                Op::ReferenceEqual => {
                    for _ in 0..2 {
                        let ty = pop(&mut stack)?;
                        if !matches!(
                            ty,
                            Ty::ScalarArray(_)
                                | Ty::ReferenceArray(_)
                                | Ty::StructuralArray(_)
                                | Ty::Reference(_)
                                | Ty::Interface(_)
                                | Ty::Intersection(_)
                                | Ty::ReferenceUnion(_)
                                | Ty::ByteArray
                                | Ty::CharacterArray
                                | Ty::StringArray
                        ) && !(ty == Ty::Literal && self.references)
                        {
                            return Err(fail(
                                pc,
                                "identity requires managed references; text requires --reference-arena",
                            ));
                        }
                    }
                    stack.push(Ty::Bool);
                }
                Op::NewArray(t) | Op::ReserveArray(t)
                    if crate::selection::scalar_array_element(t) =>
                {
                    take(&mut stack, &Ty::Int)?;
                    stack.push(self.ty(&Type::ArrayRef(Box::new(t.clone())))?);
                }
                Op::ReserveArray(t)
                    if matches!(
                        t,
                        Type::Tuple(_) | Type::Union(_) | Type::Intersection(_) | Type::ArrayRef(_)
                    ) =>
                {
                    take(&mut stack, &Ty::Int)?;
                    stack.push(self.ty(&Type::ArrayRef(Box::new(t.clone())))?);
                }
                Op::NewArray(t) | Op::ReserveArray(t)
                    if matches!(t, Type::Named(_) | Type::ArrayRef(_)) =>
                {
                    take(&mut stack, &Ty::Int)?;
                    stack.push(self.ty(&Type::ArrayRef(Box::new(t.clone())))?);
                }
                Op::ArrayElement(t)
                    if matches!(
                        t,
                        Type::Named(_)
                            | Type::Tuple(_)
                            | Type::Union(_)
                            | Type::Intersection(_)
                            | Type::ArrayRef(_)
                    ) || crate::selection::scalar_array_element(t) =>
                {
                    take(&mut stack, &Ty::Int)?;
                    take(&mut stack, &self.ty(&Type::ArrayRef(Box::new(t.clone())))?)?;
                    stack.push(Self::stack_type(&self.ty(t)?));
                }
                Op::StoreArrayElement(t)
                    if matches!(
                        t,
                        Type::Named(_)
                            | Type::Tuple(_)
                            | Type::Union(_)
                            | Type::Intersection(_)
                            | Type::ArrayRef(_)
                    ) || crate::selection::scalar_array_element(t) =>
                {
                    take(&mut stack, &self.ty(t)?)?;
                    take(&mut stack, &Ty::Int)?;
                    take(&mut stack, &self.ty(&Type::ArrayRef(Box::new(t.clone())))?)?;
                }
                Op::NewArray(t) | Op::ReserveArray(t) if matches!(t, Type::Function(_)) => {
                    take(&mut stack, &Ty::Int)?;
                    stack.push(Ty::CallableArray(t.clone()));
                }
                Op::ArrayElement(t) if matches!(t, Type::Function(_)) => {
                    take(&mut stack, &Ty::Int)?;
                    take(&mut stack, &Ty::CallableArray(t.clone()))?;
                    stack.push(self.ty(t)?);
                }
                Op::StoreArrayElement(t) if matches!(t, Type::Function(_)) => {
                    take(&mut stack, &self.ty(t)?)?;
                    take(&mut stack, &Ty::Int)?;
                    take(&mut stack, &Ty::CallableArray(t.clone()))?;
                }
                Op::NewArray(t @ (Type::String | Type::Char))
                | Op::ReserveArray(t @ (Type::String | Type::Char)) => {
                    take(&mut stack, &Ty::Int)?;
                    stack.push(self.ty(&Type::ArrayRef(Box::new(t.clone())))?);
                }
                Op::ArrayElement(t @ (Type::String | Type::Char)) => {
                    take(&mut stack, &Ty::Int)?;
                    let array = pop(&mut stack)?;
                    if array != self.ty(&Type::ArrayRef(Box::new(t.clone())))?
                        && !(t == &Type::Char && array == Ty::CharacterValues)
                    {
                        return Err(fail(pc, "character value arrays permit indexed reads only"));
                    }
                    stack.push(self.ty(t)?);
                }
                Op::StoreArrayElement(t @ (Type::String | Type::Char)) => {
                    take(&mut stack, &self.ty(t)?)?;
                    take(&mut stack, &Ty::Int)?;
                    take(&mut stack, &self.ty(&Type::ArrayRef(Box::new(t.clone())))?)?;
                }
                Op::NewArray(Type::Byte) | Op::ReserveArray(Type::Byte) => {
                    take(&mut stack, &Ty::Int)?;
                    stack.push(Ty::ByteArray);
                }
                Op::ArrayLength => {
                    if !matches!(
                        pop(&mut stack)?,
                        Ty::ScalarArray(_)
                            | Ty::ReferenceArray(_)
                            | Ty::RecordArray(_)
                            | Ty::StructuralArray(_)
                            | Ty::CallableArray(_)
                            | Ty::ByteArray
                            | Ty::CharacterArray
                            | Ty::StringArray
                            | Ty::CharacterValues
                            | Ty::ByteValues
                    ) {
                        return Err(fail(
                            pc,
                            "array length requires a byte array or immutable byte values",
                        ));
                    }
                    stack.push(Ty::Size);
                }
                Op::ArrayElement(Type::Byte) | Op::ArrayAddress(Type::Byte) => {
                    take(&mut stack, &Ty::Int)?;
                    let array = pop(&mut stack)?;
                    if array != Ty::ByteArray
                        && !(array == Ty::ByteValues && matches!(op, Op::ArrayElement(_)))
                    {
                        return Err(fail(
                            pc,
                            "byte value arrays permit only indexed reads; mutable borrows are unsupported",
                        ));
                    }
                    stack.push(if matches!(op, Op::ArrayAddress(_)) {
                        Ty::Byte.address()
                    } else {
                        Ty::Int
                    });
                }
                Op::StoreArrayElement(Type::Byte) => {
                    take(&mut stack, &Ty::Int)?;
                    take(&mut stack, &Ty::Int)?;
                    take(&mut stack, &Ty::ByteArray)?;
                }
                Op::StaticField { owner, index } => {
                    stack.push(Self::stack_type(&self.static_field(owner, *index)?.1));
                }
                Op::SetStaticField { owner, index } => {
                    let (_, ty, _, _) = self.static_field(owner, *index)?;
                    let value = pop(&mut stack)?;
                    let expected = Self::stack_type(&ty);
                    if expected != value && !self.reference_assignable(&value, &expected) {
                        return Err(fail(pc, "static field store type mismatch"));
                    }
                }
                Op::Field(n) | Op::FieldAddress(n) => {
                    let owner = pop(&mut stack)?;
                    let t = self.field(&owner, *n)?;
                    if matches!(owner, Ty::Reference(i) if self.is_array_backing(i))
                        && matches!(op, Op::FieldAddress(_))
                    {
                        return Err(fail(pc, "nominal array storage field cannot be borrowed"));
                    }
                    if matches!(op, Op::FieldAddress(_)) {
                        if !matches!(owner, Ty::Address(_) | Ty::Reference(_)) {
                            return Err(fail(pc, "field address requires a borrowed record"));
                        }
                        stack.push(t.address());
                    } else {
                        stack.push(Self::stack_type(&t));
                    }
                }
                Op::SetField(n) => {
                    let value = pop(&mut stack)?;
                    let owner = pop(&mut stack)?;
                    if matches!(owner, Ty::Reference(i) if self.is_array_backing(i)) {
                        return Err(fail(pc, "nominal array storage field cannot be replaced"));
                    }
                    let expected = Self::stack_type(&self.field(&owner, *n)?);
                    if expected != value && !self.reference_assignable(&value, &expected) {
                        return Err(fail(pc, "field store type mismatch"));
                    }
                    if !matches!(owner, Ty::Reference(_)) {
                        stack.push(if matches!(owner, Ty::Address(_)) {
                            Ty::Unit
                        } else {
                            owner
                        });
                    }
                }
                Op::BindFunction {
                    function_type,
                    target,
                } => {
                    let c = self.callee(target)?;
                    if target.instance {
                        take(&mut stack, &self.args[c][0])?;
                    }
                    stack.push(self.ty(function_type)?);
                }
                Op::Call(target) | Op::CallVirtual(target) | Op::Construct(target) => {
                    if let Some(signature) = crate::selection::callable_invoke(target) {
                        for t in signature.parameters.iter().rev() {
                            take(&mut stack, &self.ty(t)?)?;
                        }
                        take(&mut stack, &self.ty(target.owner.as_ref().unwrap())?)?;
                        if !signature.no_result {
                            stack.push(Self::stack_type(&self.ty(&signature.returns)?));
                        }
                    } else {
                        let c = self.callee(target)?;
                        let construct = matches!(op, Op::Construct(_));
                        for t in self.args[c].iter().skip(usize::from(construct)).rev() {
                            take(&mut stack, t)?;
                        }
                        if construct {
                            stack.push(self.ty(target.owner.as_ref().unwrap())?);
                        } else if let Some(t) = &self.results[c] {
                            stack.push(Self::stack_type(t));
                        }
                    }
                }
                Op::Return => {
                    if let Some(t) = &self.results[i] {
                        take(&mut stack, t)?;
                    }
                    if !stack.is_empty() {
                        return Err(fail(pc, "nonempty return stack"));
                    }
                }
                Op::Fault(_) | Op::Branch(_) => (),
                Op::BranchTrue(_) | Op::BranchFalse(_) => {
                    if !matches!(pop(&mut stack)?, Ty::Int | Ty::Bool) {
                        return Err(fail(pc, "branch requires Int32 or Boolean"));
                    }
                }
                Op::Negate => {
                    let value = pop(&mut stack)?;
                    if !matches!(value, Ty::Int | Ty::Wide | Ty::Size | Ty::Double) {
                        return Err(fail(pc, "negation requires an integer or Double"));
                    }
                    stack.push(value);
                }
                Op::BitNot => {
                    take(&mut stack, &Ty::Int)?;
                    stack.push(Ty::Int);
                }
                Op::ShiftLeft | Op::ShiftRight | Op::ShiftRightUnsigned => {
                    take(&mut stack, &Ty::Int)?;
                    let value = pop(&mut stack)?;
                    if !matches!(value, Ty::Int | Ty::Wide) {
                        return Err(fail(pc, "shift requires Int32 or Int64 and an Int32 count"));
                    }
                    stack.push(value);
                }
                Op::BitAnd | Op::BitOr | Op::BitXor => {
                    take(&mut stack, &Ty::Int)?;
                    take(&mut stack, &Ty::Int)?;
                    stack.push(Ty::Int);
                }
                Op::AddChecked
                | Op::SubChecked
                | Op::MulChecked
                | Op::AddCheckedUnsigned
                | Op::SubCheckedUnsigned
                | Op::MulCheckedUnsigned => {
                    let t = pop(&mut stack)?;
                    if !matches!(t, Ty::Int | Ty::Wide | Ty::Size) {
                        return Err(fail(
                            pc,
                            "checked arithmetic requires matching integer operands",
                        ));
                    }
                    take(&mut stack, &t)?;
                    stack.push(t);
                }
                Op::Add | Op::Sub | Op::Mul => {
                    let t = pop(&mut stack)?;
                    if !matches!(t, Ty::Int | Ty::Wide | Ty::Single | Ty::Double) {
                        return Err(fail(
                            pc,
                            "arithmetic requires Int32, Int64, Single or Double",
                        ));
                    }
                    take(&mut stack, &t)?;
                    stack.push(t);
                }
                Op::Divide | Op::DivideUnsigned | Op::Remainder | Op::RemainderUnsigned => {
                    let t = pop(&mut stack)?;
                    if !matches!(t, Ty::Int | Ty::Wide)
                        && !(matches!(t, Ty::Single | Ty::Double) && matches!(op, Op::Divide))
                    {
                        return Err(fail(
                            pc,
                            "division requires matching integer operands or floating div",
                        ));
                    }
                    take(&mut stack, &t)?;
                    stack.push(t);
                }
                Op::Equal => {
                    let t = pop(&mut stack)?;
                    if !matches!(
                        t,
                        Ty::Int
                            | Ty::Wide
                            | Ty::Single
                            | Ty::Double
                            | Ty::Bool
                            | Ty::Literal
                            | Ty::TypeToken
                    ) {
                        return Err(fail(
                            pc,
                            "equality requires Int32, Int64/UInt64, Single/Double, Boolean, String or RuntimeTypeHandle",
                        ));
                    }
                    take(&mut stack, &t)?;
                    stack.push(Ty::Bool);
                }
                Op::Greater | Op::Less | Op::GreaterUnsigned | Op::LessUnsigned => {
                    let t = pop(&mut stack)?;
                    if !matches!(t, Ty::Int | Ty::Wide | Ty::Single | Ty::Double) {
                        return Err(fail(
                            pc,
                            "ordering requires matching integer or floating operands",
                        ));
                    }
                    take(&mut stack, &t)?;
                    stack.push(Ty::Bool);
                }
                _ if flow::comparison(op).is_some() => {
                    let t = pop(&mut stack)?;
                    if !matches!(t, Ty::Int | Ty::Wide | Ty::Single | Ty::Double) {
                        return Err(fail(
                            pc,
                            "comparison branch requires matching integer or floating operands",
                        ));
                    }
                    take(&mut stack, &t)?;
                }
                _ => {
                    take(&mut stack, &Ty::Int)?;
                    take(&mut stack, &Ty::Int)?;
                    if flow::comparison(op).is_none() {
                        stack.push(
                            if matches!(
                                op,
                                Op::Greater | Op::Less | Op::GreaterUnsigned | Op::LessUnsigned
                            ) {
                                Ty::Bool
                            } else {
                                Ty::Int
                            },
                        );
                    }
                }
            }
            let successors = match op {
                Op::Return | Op::Fault(_) => vec![],
                Op::Branch(n) => vec![*n],
                Op::BranchTrue(n) | Op::BranchFalse(n) => vec![*n, pc + 1],
                _ if flow::comparison(op).is_some() => {
                    vec![flow::comparison(op).unwrap().0, pc + 1]
                }
                _ => vec![pc + 1],
            };
            for next in successors {
                if next >= stacks.len() {
                    return Err(fail(pc, "control flow leaves function body"));
                }
                if let Some(old) = &stacks[next] {
                    if old != &stack {
                        return Err(fail(next, "incompatible value stack at join"));
                    }
                } else {
                    stacks[next] = Some(stack.clone());
                    work.push_back(next);
                }
            }
        }
        self.verify_outputs(i, &stacks)?;
        Ok(stacks)
    }
}

// Recognize a straight-line argument prefix. Calls may allocate or have effects,
// but can only consume independently evaluated values above the receiver. The
// ordinary typed verifier and GC stack maps still validate/root these values.
fn leading_base_initializer(
    body: &[Op],
    base: &Type,
    no_result: &dyn Fn(&FunctionRef) -> Option<bool>,
) -> Option<usize> {
    if !matches!(body.first(), Some(Op::Arg(0))) {
        return None;
    }
    let mut arguments = 0usize;
    for (pc, op) in body.iter().enumerate().skip(1) {
        match op {
            Op::Arg(n) if *n != 0 => arguments += 1,
            Op::Int(_)
            | Op::Int64(_)
            | Op::Float32 { .. }
            | Op::Float64 { .. }
            | Op::Bool(_)
            | Op::String(_) => arguments += 1,
            Op::Add
            | Op::Sub
            | Op::Mul
            | Op::Divide
            | Op::DivideUnsigned
            | Op::Remainder
            | Op::RemainderUnsigned
            | Op::AddChecked
            | Op::SubChecked
            | Op::MulChecked
            | Op::AddCheckedUnsigned
            | Op::SubCheckedUnsigned
            | Op::MulCheckedUnsigned
            | Op::BitAnd
            | Op::BitOr
            | Op::BitXor
            | Op::ShiftLeft
            | Op::ShiftRight
            | Op::ShiftRightUnsigned
                if arguments >= 2 =>
            {
                arguments -= 1
            }
            Op::CastClass(_)
            | Op::Negate
            | Op::BitNot
            | Op::ConvertInt32
            | Op::ConvertUInt32
            | Op::ConvertInt64
            | Op::ConvertUInt64
            | Op::ConvertFloat32
            | Op::ConvertFloat64
                if arguments >= 1 =>
            {
                ()
            }
            Op::Call(target)
                if target.instance
                    && target.owner.as_ref() == Some(base)
                    && target.name.ends_with("..ctor")
                    && arguments == target.parameters.len() =>
            {
                // A later branch cannot re-enter the initialization prefix.
                if body.iter().skip(pc + 1).any(|op| matches!(op,
                    Op::Branch(target) | Op::BranchTrue(target) | Op::BranchFalse(target) if *target <= pc)) {
                    return None;
                }
                return Some(pc);
            }
            Op::Call(target) | Op::CallVirtual(target) if !target.name.ends_with("..ctor") => {
                let consumed = target.parameters.len() + usize::from(target.instance);
                if consumed > arguments {
                    return None;
                }
                arguments = arguments - consumed + usize::from(!no_result(target)?);
            }
            Op::Construct(target) if arguments >= target.parameters.len() => {
                // Construct supplies its own receiver; it cannot consume ours.
                if !target.instance || !target.name.ends_with("..ctor") || !no_result(target)? {
                    return None;
                }
                arguments = arguments - target.parameters.len() + 1;
            }
            _ => return None,
        }
    }
    None
}

#[cfg(test)]
mod object_assignment_tests {
    use super::*;

    #[test]
    fn explicit_reference_methods_preserve_private_dispatch() {
        let input = neoclr::assemble(
            r#"
.module ExplicitReference
.interface Readable
.method instance Read() -> Int32
.end
.end
.type class Cell
.implements Readable
.method instance .ctor() -> noresult
ret
.end
.method private instance Hidden() -> Int32
.override instance Readable::Read()
ldc.i4 42
ret
.end
.method instance Read() -> Int32
ldc.i4 7
ret
.end
.end
.function Main() -> Int32
newobj instance Cell::.ctor()
callvirt instance Readable::Read()
ret
.end
"#,
        )
        .unwrap();
        let profile =
            Profile::new(&input, true, None, None, &[], None, None, None, None, false).unwrap();
        let contract = input
            .functions
            .iter()
            .position(|f| f.name == "Readable.Read")
            .unwrap();
        let hidden = input
            .functions
            .iter()
            .position(|f| f.name == "Cell.Hidden")
            .unwrap();
        assert_eq!(profile.dispatch[&contract][0].1, hidden);
        assert_eq!(
            input.functions[hidden].visibility,
            neoclr::metadata::Visibility::Private
        );
        // Admission does not loosen metadata verification for public mappings.
        let mut invalid = input.clone();
        invalid.functions[hidden].visibility = neoclr::metadata::Visibility::Public;
        assert!(neoclr::LoadedProgram::new(&invalid).is_err());
    }

    #[test]
    fn persistent_byte_memory_has_three_lanes_and_rejects_raw_construction() {
        use neoclr::metadata::MemoryViewKind;
        for kind in [
            MemoryViewKind::Memory,
            MemoryViewKind::ReadOnlyMemory,
            MemoryViewKind::Slice,
            MemoryViewKind::ReadOnlySlice,
        ] {
            let mut input = crate::memory_views::test_module(kind, Type::Byte);
            let ty = Type::Named(input.types[0].name.clone());
            let p =
                Profile::new(&input, true, None, None, &[], None, None, None, None, false).unwrap();
            let value = p.ty(&ty).unwrap();
            assert_eq!(value, Ty::ByteMemory(0));
            assert_eq!(p.lanes(&value), 3);
            assert_eq!(p.bytes(&value), 24);
            assert_eq!(p.pointer_lanes(&value), vec![true, false, false]);
            assert!(p.ty(&Type::ArrayRef(Box::new(ty.clone()))).is_err());
            assert!(p.ty(&Type::Union(vec![ty.clone(), Type::Int32])).is_err());
            assert!(!crate::scalar_unions::record(&input, &ty));
            assert!(
                Profile::new(
                    &input,
                    false,
                    None,
                    None,
                    &[],
                    None,
                    None,
                    None,
                    None,
                    false
                )
                .is_err()
            );
            input.functions.push(
                serde_json::from_value(serde_json::json!({
                    "name":"Forge", "returns":ty, "body":[Op::New(ty.clone()), Op::Return]
                }))
                .unwrap(),
            );
            let p =
                Profile::new(&input, true, None, None, &[], None, None, None, None, false).unwrap();
            assert!(p.analyze(0).is_err());
        }
    }

    #[test]
    fn borrowed_byte_views_cannot_enter_native_fields() {
        use neoclr::metadata::MemoryViewKind;
        for kind in [MemoryViewKind::Slice, MemoryViewKind::ReadOnlySlice] {
            let mut input = crate::memory_views::test_module(kind, Type::Byte);
            let ty = Type::Named(input.types[0].name.clone());
            let mut holder =
                neoclr::assemble(".module Test\n.type Holder\n.field Data Int32\n.end\n")
                    .unwrap()
                    .types
                    .remove(0);
            holder.fields[0].ty = ty;
            input.types.push(holder);
            let error = Profile::new(&input, true, None, None, &[], None, None, None, None, false)
                .err()
                .unwrap();
            assert!(error.to_string().contains("cannot be stored in fields"));
        }
    }

    #[test]
    fn memory_views_never_receive_empty_record_layouts() {
        for (name, kind) in [
            ("Slice", "slice"),
            ("ReadOnlySlice", "read_only_slice"),
            ("Memory", "memory"),
            ("ReadOnlyMemory", "read_only_memory"),
        ] {
            let source = neoclr::assemble(&format!(
                ".module System\n.type System.{name}<T>\n.memoryview {kind}\n.end\n"
            ))
            .unwrap();
            let mut closed = source.clone();
            neoclr::native_metadata::retain_memory_view_shape(
                &mut closed.types[0],
                &source,
                0,
                &[Type::Int32],
            )
            .unwrap();
            closed.name = "NativeMemoryShapes".into();
            closed.types[0].generic_parameters.clear();
            for clear_kind in [false, true] {
                if clear_kind {
                    closed.types[0].memory_view = None;
                }
                let error = Profile::new(
                    &closed,
                    true,
                    None,
                    None,
                    &[],
                    None,
                    None,
                    None,
                    None,
                    false,
                )
                .err()
                .expect("opaque shapes cannot become empty records");
                assert!(error.to_string().contains("native ABI and GC"));
            }
        }
    }

    #[test]
    fn base_initializer_accepts_independent_expressions_without_exposing_receiver() {
        let base = Type::Named("Base".into());
        let call =
            Op::Call(neoclr::assembler::parse_function_ref("instance Base::.ctor(Int32)").unwrap());
        assert_eq!(
            leading_base_initializer(&[Op::Arg(0), Op::Int(2), call.clone()], &base, &|_| Some(
                false
            )),
            Some(2)
        );
        assert_eq!(
            leading_base_initializer(
                &[Op::Arg(0), Op::Arg(1), Op::Int(1), Op::Add, call.clone()],
                &base,
                &|_| Some(false)
            ),
            Some(4)
        );
        for prefix in [
            vec![Op::Arg(0), Op::Arg(0)],
            vec![Op::Arg(0), Op::Int(1), Op::Add],
            vec![Op::Arg(0), Op::Dup],
            vec![Op::Arg(0), Op::Field(0)],
            vec![Op::Arg(0), Op::CastClass(base.clone())],
            vec![Op::Arg(0), Op::Int(1), Op::Store(0)],
            vec![Op::Arg(0), Op::Branch(2), Op::Int(1)],
        ] {
            let mut body = prefix;
            body.push(call.clone());
            assert_eq!(
                leading_base_initializer(&body, &base, &|_| Some(false)),
                None,
                "{body:?}"
            );
        }
        assert_eq!(
            leading_base_initializer(
                &[Op::Arg(0), Op::Int(2), call, Op::Branch(0)],
                &base,
                &|_| Some(false)
            ),
            None
        );
    }

    #[test]
    fn base_arguments_cannot_pass_uninitialized_receiver_to_helpers() {
        let base = Type::Named("Base".into());
        let target = |name| neoclr::assembler::parse_function_ref(name).unwrap();
        let finish = Op::Call(target("instance Base::.ctor(Int32)"));
        for helper in [
            Op::Call(target("Escape(Base)")),
            Op::CallVirtual(target("instance Base::Read()")),
            Op::Construct(target("instance Box::.ctor(Base)")),
        ] {
            assert_eq!(
                leading_base_initializer(&[Op::Arg(0), helper, finish.clone()], &base, &|_| Some(
                    false
                )),
                None
            );
        }
        let nested = [
            Op::Arg(0),
            Op::Int(42),
            Op::Construct(target("instance Box::.ctor(Int32)")),
            Op::CastClass(Type::Named("Box".into())),
            Op::Call(target("Read(Box)")),
            finish.clone(),
        ];
        assert_eq!(
            leading_base_initializer(&nested, &base, &|target| Some(
                target.name.ends_with("..ctor")
            )),
            Some(5)
        );
        // A no-result helper does not manufacture an argument for the base call.
        let body = [
            Op::Arg(0),
            Op::Int(42),
            Op::Call(target("Consume(Int32)")),
            finish,
        ];
        assert_eq!(
            leading_base_initializer(&body, &base, &|_| Some(true)),
            None
        );
    }

    #[test]
    fn interface_upcast_requires_the_explicit_object_base() {
        let input = neoclr::assemble(".module Upcasts\n.type class abstract Root\n.end\n.interface View\n.end\n.type class Other\n.end\n").unwrap();
        let index = |name: &str| input.types.iter().position(|t| t.name == name).unwrap();
        let root = index("Root");
        let view = Ty::Interface(index("View"));
        let make = |object_base| {
            Profile::new(
                &input,
                true,
                object_base,
                None,
                &[],
                None,
                None,
                None,
                None,
                false,
            )
            .unwrap()
        };
        let profile = make(Some(root));
        assert!(profile.reference_assignable(&view, &Ty::Reference(root)));
        assert!(!profile.reference_assignable(&view, &Ty::Reference(index("Other"))));
        assert!(!make(None).reference_assignable(&view, &Ty::Reference(root)));
    }
}

#[cfg(test)]
mod static_token_tests {
    use super::*;

    #[test]
    fn static_type_tokens_are_valid_without_admitting_static_values() {
        let mut input = neoclr::assemble(".module StaticTokens\n.entry Main\n.type class StaticOwner\n.end\n.function Main() -> Int32\nldtoken StaticOwner\nldtoken StaticOwner\nceq\nbrfalse Failed\nldc.i4 42\nret\nFailed:\nldc.i4 1\nret\n.end\n").unwrap();
        input.types[0].is_abstract = true;
        input.types[0].is_sealed = true;
        assert!(crate::selection::static_owner(&input.types[0]));
        let program = neoclr::LoadedProgram::new(&input).unwrap();
        program.verify().unwrap();
        assert_eq!(
            program.run(neoclr::Limits::default()).unwrap().value,
            neoclr::Value::Int32(42)
        );
        let (selected, _) = crate::selection::prepare(&input, "Main").unwrap();
        assert!(
            !crate::compiler::compile(&selected, "Main", false, None)
                .unwrap()
                .is_empty()
        );
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        crate::linking::descriptor_token_tests::assert_native_result(&selected, 42);
        let mut value = selected.clone();
        value.functions[0]
            .locals
            .push(Type::Named("StaticOwner".into()));
        assert!(
            crate::compiler::compile(&value, "Main", false, None)
                .unwrap_err()
                .to_string()
                .contains("static owners cannot be used as values")
        );
    }
}

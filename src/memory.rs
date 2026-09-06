//! Explicit native heap storage, with interpreter side tables for diagnostics.
use crate::{
    Fault, Module, Value,
    metadata::{Representation, Type},
};

#[derive(Debug, Clone)]
pub struct Pointer {
    pub address: usize,
    pub allocation: Option<usize>,
    pub offset: usize,
    pub target: Type,
}

impl Pointer {
    pub fn null(target: Type) -> Self {
        Self {
            address: 0,
            allocation: None,
            offset: 0,
            target,
        }
    }
}

impl PartialEq for Pointer {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address && self.target == other.target
    }
}
impl Eq for Pointer {}

#[derive(Debug, Clone)]
pub struct Layout {
    pub size: usize,
    pub alignment: usize,
    pub fields: Vec<FieldLayout>,
}

#[derive(Debug, Clone)]
pub struct FieldLayout {
    pub offset: usize,
    pub ty: Type,
    pub layout: Layout,
}

/// Sequential prototype layout with native-size pointer fields.
pub fn layout(module: &Module, ty: &Type) -> Result<Layout, Fault> {
    fn build(
        module: &Module,
        ty: &Type,
        path: &mut Vec<String>,
        budget: &mut usize,
    ) -> Result<Layout, Fault> {
        if *budget == 0 {
            return Err(Fault::new("layout complexity limit exceeded"));
        }
        *budget -= 1;
        let (size, alignment) = match ty {
            Type::Int32 => (4, 4),
            Type::Boolean => (1, 1),
            Type::Void => (0, 1),
            Type::Ptr(_) => (std::mem::size_of::<usize>(), std::mem::align_of::<usize>()),
            Type::Named(name) => {
                if path.contains(name) || path.len() >= 64 {
                    return Err(Fault::new("recursive or excessively nested record layout"));
                }
                let def = module
                    .type_definition(ty)
                    .ok_or_else(|| Fault::new("unknown layout type"))?;
                if def.representation != Representation::Record {
                    return Err(Fault::new("unsupported memory representation"));
                }
                path.push(name.clone());
                let mut size = 0usize;
                let mut alignment = 1usize;
                let mut fields = vec![];
                for field in &def.fields {
                    let layout = build(module, &field.ty, path, budget)?;
                    alignment = alignment.max(layout.alignment);
                    size = align_up(size, layout.alignment)?;
                    fields.push(FieldLayout {
                        offset: size,
                        ty: field.ty.clone(),
                        layout: layout.clone(),
                    });
                    size = size
                        .checked_add(layout.size)
                        .ok_or_else(|| Fault::new("layout size overflow"))?;
                }
                path.pop();
                size = align_up(size, alignment)?;
                if size > i32::MAX as usize {
                    return Err(Fault::new("layout exceeds prototype Int32 size range"));
                }
                return Ok(Layout {
                    size,
                    alignment,
                    fields,
                });
            }
            _ => {
                return Err(Fault::new(format!(
                    "memory layout is not implemented for {ty:?}"
                )));
            }
        };
        Ok(Layout {
            size,
            alignment,
            fields: vec![],
        })
    }
    build(module, ty, &mut vec![], &mut 16384)
}

fn align_up(size: usize, alignment: usize) -> Result<usize, Fault> {
    size.checked_add(alignment - 1)
        .map(|n| n & !(alignment - 1))
        .ok_or_else(|| Fault::new("layout size overflow"))
}

#[derive(Debug)]
struct NativeBytes {
    base: std::ptr::NonNull<u8>,
    size: usize,
    layout: std::alloc::Layout,
}

impl NativeBytes {
    fn new(size: usize, alignment: usize) -> Result<Self, Fault> {
        let layout = std::alloc::Layout::from_size_align(size.max(1), alignment)
            .map_err(|_| Fault::new("invalid native allocation layout"))?;
        // SAFETY: nonzero, valid Layout. Host bytes are zeroed to make internal
        // copying safe; the separate guest initialization bitmap starts false.
        let base = std::ptr::NonNull::new(unsafe { std::alloc::alloc_zeroed(layout) })
            .ok_or_else(|| Fault::new("native heap allocation failed"))?;
        Ok(Self { base, size, layout })
    }
    fn address(&self) -> usize {
        self.base.as_ptr() as usize
    }
    fn len(&self) -> usize {
        self.size
    }
    fn slice(&self) -> &[u8] {
        // SAFETY: live allocation contains at least size initialized bytes.
        unsafe { std::slice::from_raw_parts(self.base.as_ptr(), self.size) }
    }
    fn slice_mut(&mut self) -> &mut [u8] {
        // SAFETY: exclusive access; no guest operation dereferences stored integers.
        unsafe { std::slice::from_raw_parts_mut(self.base.as_ptr(), self.size) }
    }
}
impl Drop for NativeBytes {
    fn drop(&mut self) {
        // SAFETY: this buffer uniquely owns base, allocated with this same Layout.
        unsafe { std::alloc::dealloc(self.base.as_ptr(), self.layout) }
    }
}

#[derive(Debug)]
struct Allocation {
    bytes: NativeBytes,
    initialized: Vec<bool>,
    // Stored pointer provenance is diagnostics only; the bytes contain real addresses.
    pointers: std::collections::BTreeMap<usize, Pointer>,
}

#[derive(Debug, Default)]
pub struct PointerHeap {
    // Never reuse identities, so a stale pointer cannot address a later allocation.
    allocations: Vec<Option<Allocation>>,
    live_bytes: usize,
}

impl PointerHeap {
    pub fn live_bytes(&self) -> usize {
        self.live_bytes
    }
    pub fn live_allocations(&self) -> usize {
        self.allocations.iter().filter(|a| a.is_some()).count()
    }

    pub(crate) fn allocate(
        &mut self,
        ty: Type,
        layout: &Layout,
        count: i32,
        byte_limit: usize,
        allocation_limit: usize,
    ) -> Result<Pointer, Fault> {
        let count = usize::try_from(count).map_err(|_| Fault::new("negative allocation count"))?;
        let size = layout
            .size
            .checked_mul(count)
            .ok_or_else(|| Fault::new("allocation size overflow"))?;
        let total = self
            .live_bytes
            .checked_add(size)
            .ok_or_else(|| Fault::new("allocation size overflow"))?;
        if total > byte_limit {
            return Err(Fault::new("pointer heap byte limit exceeded"));
        }
        if self.allocations.len() >= allocation_limit {
            return Err(Fault::new("pointer allocation limit exceeded"));
        }
        let mut initialized = Vec::new();
        initialized
            .try_reserve_exact(size)
            .map_err(|_| Fault::new("initialization map allocation failed"))?;
        initialized.resize(size, false);
        self.allocations
            .try_reserve(1)
            .map_err(|_| Fault::new("allocation tracking failed"))?;
        let bytes = NativeBytes::new(size, layout.alignment)?;
        let address = bytes.address();
        let id = self.allocations.len();
        self.allocations.push(Some(Allocation {
            bytes,
            initialized,
            pointers: Default::default(),
        }));
        self.live_bytes = total;
        Ok(Pointer {
            address,
            allocation: Some(id),
            offset: 0,
            target: ty,
        })
    }

    fn allocation(&self, pointer: &Pointer) -> Result<&Allocation, Fault> {
        if pointer.address == 0 {
            return Err(Fault::new("null pointer access"));
        }
        let id = pointer
            .allocation
            .ok_or_else(|| Fault::new("untracked native pointer access"))?;
        let allocation = self
            .allocations
            .get(id)
            .ok_or_else(|| Fault::new("invalid allocation identity"))?
            .as_ref()
            .ok_or_else(|| Fault::new("use after free"))?;
        if allocation.bytes.address().checked_add(pointer.offset) != Some(pointer.address) {
            return Err(Fault::new("native pointer tracking mismatch"));
        }
        Ok(allocation)
    }

    fn range(&self, pointer: &Pointer, layout: &Layout) -> Result<std::ops::Range<usize>, Fault> {
        let allocation = self.allocation(pointer)?;
        if pointer.address & (layout.alignment - 1) != 0 {
            return Err(Fault::new("misaligned pointer access"));
        }
        let end = pointer
            .offset
            .checked_add(layout.size)
            .ok_or_else(|| Fault::new("pointer range overflow"))?;
        if end > allocation.bytes.len() {
            return Err(Fault::new("pointer access out of bounds"));
        }
        Ok(pointer.offset..end)
    }

    pub(crate) fn offset(&self, pointer: &Pointer, bytes: i32) -> Result<Pointer, Fault> {
        let allocation = self.allocation(pointer)?;
        let offset = pointer.offset as i128 + bytes as i128;
        if offset < 0 || offset > allocation.bytes.len() as i128 {
            return Err(Fault::new("pointer offset out of bounds"));
        }
        Ok(Pointer {
            address: allocation.bytes.address() + offset as usize,
            offset: offset as usize,
            ..pointer.clone()
        })
    }

    pub(crate) fn field(
        &self,
        pointer: &Pointer,
        layout: &Layout,
        index: usize,
    ) -> Result<Pointer, Fault> {
        self.range(pointer, layout)?;
        let field = layout
            .fields
            .get(index)
            .ok_or_else(|| Fault::new("field index out of range"))?;
        Ok(Pointer {
            address: pointer.address + field.offset,
            allocation: pointer.allocation,
            offset: pointer.offset + field.offset,
            target: field.ty.clone(),
        })
    }

    pub(crate) fn free(&mut self, pointer: &Pointer) -> Result<(), Fault> {
        if pointer.address == 0 {
            return Ok(());
        }
        let id = pointer
            .allocation
            .ok_or_else(|| Fault::new("cannot free untracked native pointer"))?;
        if pointer.offset != 0 {
            return Err(Fault::new("free requires allocation base pointer"));
        }
        let slot = self
            .allocations
            .get_mut(id)
            .ok_or_else(|| Fault::new("invalid allocation identity"))?;
        let allocation = slot.as_ref().ok_or_else(|| Fault::new("double free"))?;
        if pointer.address != allocation.bytes.address() {
            return Err(Fault::new("free requires allocation base address"));
        }
        let allocation = slot.take().ok_or_else(|| Fault::new("double free"))?;
        self.live_bytes -= allocation.bytes.len();
        Ok(())
    }

    pub(crate) fn read(&self, pointer: &Pointer, layout: &Layout) -> Result<Value, Fault> {
        self.range(pointer, layout)?;
        let allocation = self.allocation(pointer)?;
        decode(&pointer.target, layout, pointer.offset, allocation)
    }

    pub(crate) fn write(
        &mut self,
        pointer: &Pointer,
        layout: &Layout,
        value: &Value,
    ) -> Result<(), Fault> {
        let range = self.range(pointer, layout)?;
        // Encode before changing memory so an invalid aggregate cannot partially write.
        let mut bytes = vec![0u8; layout.size];
        let mut initialized = vec![false; layout.size];
        let mut pointers = std::collections::BTreeMap::new();
        encode(
            &pointer.target,
            layout,
            value,
            0,
            &mut bytes,
            &mut initialized,
            &mut pointers,
        )?;
        let allocation = self
            .allocations
            .get_mut(
                pointer
                    .allocation
                    .ok_or_else(|| Fault::new("null pointer"))?,
            )
            .and_then(Option::as_mut)
            .ok_or_else(|| Fault::new("invalid allocation"))?;
        allocation.bytes.slice_mut()[range.clone()].copy_from_slice(&bytes);
        allocation.pointers.retain(|offset, _| {
            *offset + std::mem::size_of::<usize>() <= range.start || *offset >= range.end
        });
        for (offset, value) in pointers {
            allocation.pointers.insert(range.start + offset, value);
        }
        allocation.initialized[range].copy_from_slice(&initialized);
        Ok(())
    }
}

fn decode(
    ty: &Type,
    layout: &Layout,
    offset: usize,
    allocation: &Allocation,
) -> Result<Value, Fault> {
    match ty {
        Type::Named(name) => {
            let fields = layout
                .fields
                .iter()
                .map(|f| decode(&f.ty, &f.layout, offset + f.offset, allocation))
                .collect::<Result<_, _>>()?;
            Ok(Value::Object {
                name: name.clone(),
                fields,
            })
        }
        Type::Void => Ok(Value::Void),
        Type::Int32 | Type::Boolean | Type::Ptr(_) => {
            if !allocation.initialized[offset..offset + layout.size]
                .iter()
                .all(|b| *b)
            {
                return Err(Fault::new("read of uninitialized memory"));
            }
            if let Type::Ptr(target) = ty {
                let mut bytes = [0; std::mem::size_of::<usize>()];
                bytes.copy_from_slice(&allocation.bytes.slice()[offset..offset + layout.size]);
                let address = usize::from_ne_bytes(bytes);
                let mut pointer = allocation
                    .pointers
                    .get(&offset)
                    .filter(|p| p.address == address)
                    .cloned()
                    .unwrap_or(Pointer {
                        address,
                        allocation: None,
                        offset: 0,
                        target: *target.clone(),
                    });
                pointer.target = *target.clone();
                Ok(Value::Pointer(pointer))
            } else if *ty == Type::Int32 {
                let mut bytes = [0; 4];
                bytes.copy_from_slice(&allocation.bytes.slice()[offset..offset + 4]);
                Ok(Value::Int32(i32::from_ne_bytes(bytes)))
            } else {
                match allocation.bytes.slice()[offset] {
                    0 => Ok(Value::Boolean(false)),
                    1 => Ok(Value::Boolean(true)),
                    _ => Err(Fault::new("invalid Boolean representation")),
                }
            }
        }
        _ => Err(Fault::new("unsupported memory read type")),
    }
}

fn encode(
    ty: &Type,
    layout: &Layout,
    value: &Value,
    offset: usize,
    bytes: &mut [u8],
    initialized: &mut [bool],
    pointers: &mut std::collections::BTreeMap<usize, Pointer>,
) -> Result<(), Fault> {
    if value.ty() != *ty {
        return Err(Fault::new("memory store type mismatch"));
    }
    match value {
        Value::Int32(n) => {
            bytes[offset..offset + 4].copy_from_slice(&n.to_ne_bytes());
            initialized[offset..offset + 4].fill(true);
        }
        Value::Boolean(b) => {
            bytes[offset] = u8::from(*b);
            initialized[offset] = true;
        }
        Value::Pointer(pointer) => {
            let size = std::mem::size_of::<usize>();
            bytes[offset..offset + size].copy_from_slice(&pointer.address.to_ne_bytes());
            initialized[offset..offset + size].fill(true);
            pointers.insert(offset, pointer.clone());
        }
        Value::Void => (),
        Value::Object { fields, .. } => {
            if fields.len() != layout.fields.len() {
                return Err(Fault::new("memory store field count mismatch"));
            }
            for (value, field) in fields.iter().zip(&layout.fields) {
                encode(
                    &field.ty,
                    &field.layout,
                    value,
                    offset + field.offset,
                    bytes,
                    initialized,
                    pointers,
                )?;
            }
        }
        _ => return Err(Fault::new("unsupported memory store type")),
    }
    Ok(())
}

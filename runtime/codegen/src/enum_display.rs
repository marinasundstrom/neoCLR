//! General enum display over ordinary boxes and existing text services.
//! Scratch parts contain image literals only: no managed temporary array/strings.
use super::{Error, slot};
use cranelift_codegen::ir::{self, AbiParam, InstBuilder, MemFlags, types};
use cranelift_frontend::FunctionBuilder;
use cranelift_module::{DataDescription, DataId, FuncId, Linkage, Module};

struct Shape {
    index: usize,
    names: Vec<(i32, DataId)>,
    masks: Vec<(i32, DataId, i32)>,
}

pub(super) struct Display {
    types: Vec<usize>,
    shapes: Vec<Shape>,
    separator: DataId,
    number: FuncId,
    join: FuncId,
}

fn literal(module: &mut impl Module, name: &str, text: &str) -> Result<DataId, Error> {
    let id = module.declare_data(name, Linkage::Local, false, false)?;
    let mut data = DataDescription::new();
    let mut bytes = (text.len() as u64).to_le_bytes().to_vec();
    bytes.extend_from_slice(text.as_bytes());
    data.define(bytes.into_boxed_slice());
    data.set_align(8);
    module.define_data(id, &data)?;
    Ok(id)
}

fn pointer(module: &mut impl Module, b: &mut FunctionBuilder<'_>, id: DataId) -> ir::Value {
    let data = module.declare_data_in_func(id, b.func);
    b.ins().global_value(types::I64, data)
}

impl Display {
    pub(super) fn prepare(
        input: &neoclr::Module,
        details: Option<&crate::fault_details::Options>,
        module: &mut impl Module,
    ) -> Result<Option<Self>, Error> {
        let Some(d) = details else { return Ok(None) };
        if d.object_display
            .keys()
            .all(|i| d.object_value_kinds.contains_key(i))
        {
            return Ok(None);
        }
        let mut types: Vec<_> = d
            .empty_record_boxes
            .values()
            .copied()
            .filter(|i| input.types[*i].enum_info.is_some())
            .collect();
        types.sort_unstable();
        types.dedup();
        if types.is_empty() {
            return Ok(None);
        }
        let separator = literal(module, "neoclr_enum_separator", ", ")?;
        let mut shapes = Vec::new();
        for &index in &types {
            let info = input.types[index].enum_info.as_ref().unwrap();
            let mut names = Vec::new();
            for (ordinal, member) in info.members.iter().enumerate() {
                names.push((
                    member.value,
                    literal(
                        module,
                        &format!("neoclr_enum_{index}_{ordinal}"),
                        &member.name,
                    )?,
                ));
            }
            let mut masks = Vec::new();
            if info.flags {
                let mut total = 0usize;
                for member in neoclr::native_metadata::enum_format_candidates(info) {
                    let ordinal = info
                        .members
                        .iter()
                        .position(|m| m.name == member.name)
                        .unwrap();
                    total = total
                        .checked_add(member.name.len() + 2)
                        .ok_or("enum text length overflow")?;
                    if total > i32::MAX as usize {
                        return Err("enum text exceeds String length limit".into());
                    }
                    masks.push((member.value, names[ordinal].1, member.name.len() as i32));
                }
            }
            shapes.push(Shape {
                index,
                names,
                masks,
            });
        }
        let mut sig = module.make_signature();
        sig.params
            .extend([types::I32, types::I64, types::I64].map(AbiParam::new));
        sig.returns.push(AbiParam::new(types::I32));
        let number = module.declare_function(
            if d.native_gc {
                "neoclr_int32_to_string_rooted_v1"
            } else {
                "neoclr_int32_to_string_v1"
            },
            Linkage::Import,
            &sig,
        )?;
        sig.params = [
            types::I64,
            types::I32,
            types::I64,
            types::I32,
            types::I64,
            types::I64,
        ]
        .map(AbiParam::new)
        .to_vec();
        let join = module.declare_function(
            if d.native_gc {
                "neoclr_string_join_parts_rooted_v1"
            } else {
                "neoclr_string_join_parts_v1"
            },
            Linkage::Import,
            &sig,
        )?;
        Ok(Some(Self {
            types,
            shapes,
            separator,
            number,
            join,
        }))
    }

    pub(super) fn types(&self) -> &[usize] {
        &self.types
    }

    pub(super) fn emit(
        &self,
        module: &mut impl Module,
        b: &mut FunctionBuilder<'_>,
        index: usize,
        value: ir::Value,
        arena: ir::Value,
        output: ir::Value,
    ) -> ir::Value {
        use ir::condcodes::IntCC;
        let shape = self.shapes.iter().find(|s| s.index == index).unwrap();
        let done = b.create_block();
        b.append_block_param(done, types::I32);
        let numeric = b.create_block();
        for &(bits, text) in &shape.names {
            let named = b.create_block();
            let next = b.create_block();
            let equal = b.ins().icmp_imm(IntCC::Equal, value, bits as i64);
            b.ins().brif(equal, named, &[], next, &[]);
            b.switch_to_block(named);
            let text = pointer(module, b, text);
            b.ins().store(MemFlags::new(), text, output, 0);
            let zero = b.ins().iconst(types::I32, 0);
            b.ins().jump(done, &[zero.into()]);
            b.switch_to_block(next);
        }
        if !shape.masks.is_empty() {
            // Each accepted nonzero mask clears at least one remaining bit,
            // so an Int32 needs at most 32 slots, regardless of metadata size.
            let scratch = slot(b, 16 + 32 * 8);
            let parts = b.ins().stack_addr(types::I64, scratch, 0);
            let mut remaining = value;
            let mut count = b.ins().iconst(types::I64, 0);
            let mut length = b.ins().iconst(types::I32, 0);
            for &(mask, text, bytes) in shape.masks.iter().rev() {
                let selected = b.create_block();
                let next = b.create_block();
                for ty in [types::I32, types::I64, types::I32] {
                    b.append_block_param(next, ty);
                }
                let bits = b.ins().band_imm(remaining, mask as i64);
                let fits = b.ins().icmp_imm(IntCC::Equal, bits, mask as i64);
                b.ins().brif(
                    fits,
                    selected,
                    &[],
                    next,
                    &[remaining.into(), count.into(), length.into()],
                );
                b.switch_to_block(selected);
                let reverse = b.ins().irsub_imm(count, 31);
                let offset = b.ins().imul_imm(reverse, 8);
                let address = b.ins().iadd(parts, offset);
                let name = pointer(module, b, text);
                b.ins().store(MemFlags::new(), name, address, 16);
                let first = b.ins().icmp_imm(IntCC::Equal, count, 0);
                let zero = b.ins().iconst(types::I32, 0);
                let two = b.ins().iconst(types::I32, 2);
                let separator = b.ins().select(first, zero, two);
                let size = b.ins().iadd(length, separator);
                let size = b.ins().iadd_imm(size, bytes as i64);
                let consumed = b.ins().band_imm(remaining, (!mask) as i64);
                let added = b.ins().iadd_imm(count, 1);
                b.ins()
                    .jump(next, &[consumed.into(), added.into(), size.into()]);
                b.switch_to_block(next);
                remaining = b.block_params(next)[0];
                count = b.block_params(next)[1];
                length = b.block_params(next)[2];
            }
            let joined = b.create_block();
            let complete = b.ins().icmp_imm(IntCC::Equal, remaining, 0);
            let any = b.ins().icmp_imm(IntCC::NotEqual, count, 0);
            let valid = b.ins().band(complete, any);
            b.ins().brif(valid, joined, &[], numeric, &[]);
            b.switch_to_block(joined);
            // A private String[] descriptor over stable image literals. The
            // header precedes the initialized suffix; it never escapes this call.
            // No managed owner is hidden in this stack descriptor during GC.
            let prefix = b.ins().irsub_imm(count, 32);
            let offset = b.ins().imul_imm(prefix, 8);
            let array = b.ins().iadd(parts, offset);
            let kind = b.ins().iconst(types::I64, 0x8000_0003);
            b.ins().store(MemFlags::new(), kind, array, 0);
            b.ins().store(MemFlags::new(), count, array, 8);
            let count = b.ins().ireduce(types::I32, count);
            let separator = pointer(module, b, self.separator);
            let join = module.declare_func_in_func(self.join, b.func);
            let call = b
                .ins()
                .call(join, &[array, count, separator, length, arena, output]);
            let status = b.inst_results(call)[0];
            b.ins().jump(done, &[status.into()]);
        } else {
            b.ins().jump(numeric, &[]);
        }
        b.switch_to_block(numeric);
        let number = module.declare_func_in_func(self.number, b.func);
        let call = b.ins().call(number, &[value, arena, output]);
        let status = b.inst_results(call)[0];
        b.ins().jump(done, &[status.into()]);
        b.switch_to_block(done);
        b.block_params(done)[0]
    }
}

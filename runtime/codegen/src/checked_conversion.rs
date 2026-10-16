//! Checked conversions validate before narrowing; no host traps escape the guest ABI.
use cranelift_codegen::ir::{
    self, InstBuilder,
    condcodes::{FloatCC, IntCC},
    types,
};
use cranelift_frontend::FunctionBuilder;
use neoclr::metadata::Instruction as Op;

/// Destination width/signedness and integer source interpretation.
/// Both currently supported codegen targets have 64-bit native integers.
pub(crate) fn spec(op: &Op) -> Option<(u32, bool, bool)> {
    Some(match op {
        Op::CheckedInt8 => (8, true, false),
        Op::CheckedUInt8 => (8, false, false),
        Op::CheckedInt16 => (16, true, false),
        Op::CheckedUInt16 => (16, false, false),
        Op::CheckedInt32 => (32, true, false),
        Op::CheckedUInt32 => (32, false, false),
        Op::CheckedInt64 | Op::CheckedNativeInt => (64, true, false),
        Op::CheckedUInt64 | Op::CheckedNativeUInt => (64, false, false),
        Op::CheckedInt8Unsigned => (8, true, true),
        Op::CheckedUInt8Unsigned => (8, false, true),
        Op::CheckedInt16Unsigned => (16, true, true),
        Op::CheckedUInt16Unsigned => (16, false, true),
        Op::CheckedInt32Unsigned => (32, true, true),
        Op::CheckedUInt32Unsigned => (32, false, true),
        Op::CheckedInt64Unsigned | Op::CheckedNativeIntUnsigned => (64, true, true),
        Op::CheckedUInt64Unsigned | Op::CheckedNativeUIntUnsigned => (64, false, true),
        _ => return None,
    })
}

pub(crate) fn emit(
    b: &mut FunctionBuilder<'_>,
    op: &Op,
    value: ir::Value,
    site: Option<&crate::fault_details::Site>,
) -> ir::Value {
    let (bits, signed, source_unsigned) = spec(op).expect("validated checked conversion");
    let source = b.func.dfg.value_type(value);
    let result_type = if bits <= 32 { types::I32 } else { types::I64 };
    let status = b.ins().iconst(types::I32, 2);
    if source.is_float() {
        let value = if source == types::F32 {
            b.ins().fpromote(types::F64, value)
        } else {
            value
        };
        let truncated = b.ins().trunc(value);
        // Exclusive power-of-two bounds stay exact even for UInt64. Ordered
        // comparisons reject NaN and infinities before the nontrapping conversion.
        let lower = if signed {
            -2f64.powi(bits as i32 - 1)
        } else {
            0.0
        };
        let upper = 2f64.powi(bits as i32 - i32::from(signed));
        let low = b.ins().f64const(lower);
        let high = b.ins().f64const(upper);
        let above = b.ins().fcmp(FloatCC::GreaterThanOrEqual, truncated, low);
        let below = b.ins().fcmp(FloatCC::LessThan, truncated, high);
        let valid = b.ins().band(above, below);
        let failed = b.ins().icmp_imm(IntCC::Equal, valid, 0);
        crate::compiler::return_if_detailed(b, failed, status, site);
        return if signed {
            b.ins().fcvt_to_sint_sat(result_type, truncated)
        } else {
            b.ins().fcvt_to_uint_sat(result_type, truncated)
        };
    }

    let value = if source == types::I32 {
        if source_unsigned {
            b.ins().uextend(types::I64, value)
        } else {
            b.ins().sextend(types::I64, value)
        }
    } else {
        value
    };
    if !source_unsigned && !(signed && bits == 64) {
        let lower = if signed { -(1i64 << (bits - 1)) } else { 0 };
        let failed = b.ins().icmp_imm(IntCC::SignedLessThan, value, lower);
        crate::compiler::return_if_detailed(b, failed, status, site);
    }
    if signed || bits < 64 {
        let max = if signed {
            (1u64 << (bits - 1)) - 1
        } else {
            (1u64 << bits) - 1
        };
        let comparison = if source_unsigned {
            IntCC::UnsignedGreaterThan
        } else {
            IntCC::SignedGreaterThan
        };
        let failed = b.ins().icmp_imm(comparison, value, max as i64);
        crate::compiler::return_if_detailed(b, failed, status, site);
    }
    if result_type == types::I32 {
        b.ins().ireduce(types::I32, value)
    } else {
        value
    }
}

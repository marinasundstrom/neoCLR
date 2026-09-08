//! Logical runtime-service uses; no target ABI or ownership policy is implied.
use crate::{
    Fault,
    metadata::{Function, Instruction as Op},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuntimeService {
    NativeAllocation,
    FrameAllocation,
    PointerMemory,
    ManagedHeap,
    ParseInt32,
    FormatInt32,
    ConsoleOutput,
    NativeInterop,
    StringOperations,
    CharacterClassification,
    MathOperations,
    LocalClock,
    ErrorValues,
    FileInput,
    ConsoleInput,
    ValueStorage,
    TypeInspection,
    InterfaceDispatch,
    SlotReferences,
    ManagedArrays,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceUse {
    pub service: RuntimeService,
    /// None identifies an import declaration; Some identifies an IL instruction.
    pub instruction: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingService {
    /// Index in the associated Reachability report, not a metadata token.
    pub function: usize,
    pub instruction: Option<usize>,
    pub service: RuntimeService,
}

pub(crate) fn uses(function: &Function) -> Result<Vec<ServiceUse>, Fault> {
    if function.pinvoke.is_some() {
        return Ok(vec![ServiceUse {
            service: RuntimeService::NativeInterop,
            instruction: None,
        }]);
    }
    if function.is_internal_call() {
        let service = match crate::native::bind(function)? {
            crate::native::Binding::LocalClock => RuntimeService::LocalClock,
            crate::native::Binding::Math(_) => RuntimeService::MathOperations,
            crate::native::Binding::Reflection(_)
            | crate::native::Binding::TypeName
            | crate::native::Binding::TypeEquals
            | crate::native::Binding::TypeArgumentCount
            | crate::native::Binding::TypeArgument => RuntimeService::TypeInspection,
            crate::native::Binding::ConsoleReadByte => RuntimeService::ConsoleInput,
            crate::native::Binding::ReadAllText => RuntimeService::FileInput,
            crate::native::Binding::ErrorFromMessage | crate::native::Binding::ErrorMessage => {
                RuntimeService::ErrorValues
            }
            crate::native::Binding::ParseInt32 => RuntimeService::ParseInt32,
            crate::native::Binding::Int32ToString => RuntimeService::FormatInt32,
            crate::native::Binding::WriteLine => RuntimeService::ConsoleOutput,
            crate::native::Binding::CharCategory => RuntimeService::CharacterClassification,
            crate::native::Binding::StringConcat
            | crate::native::Binding::StringCompareOrdinal
            | crate::native::Binding::StringContainsOrdinal
            | crate::native::Binding::StringStartsWithOrdinal
            | crate::native::Binding::StringEndsWithOrdinal
            | crate::native::Binding::StringByteCount
            | crate::native::Binding::StringSliceUtf8 => RuntimeService::StringOperations,
        };
        let mut uses = vec![ServiceUse {
            service,
            instruction: None,
        }];
        if matches!(
            crate::native::bind(function)?,
            crate::native::Binding::LocalClock
        ) {
            uses.push(ServiceUse {
                service: RuntimeService::ManagedArrays,
                instruction: None,
            });
        }
        if let crate::native::Binding::Reflection(query) = crate::native::bind(function)? {
            use crate::reflection::Query;
            if !matches!(
                query,
                Query::Shape | Query::DisplayName | Query::ElementType
            ) {
                uses.push(ServiceUse {
                    service: RuntimeService::ManagedArrays,
                    instruction: None,
                });
            }
            if matches!(query, Query::Properties | Query::ElementType) {
                uses.push(ServiceUse {
                    service: RuntimeService::ValueStorage,
                    instruction: None,
                });
            }
        }
        if function.returns == crate::metadata::Type::Value {
            uses.push(ServiceUse {
                service: RuntimeService::ValueStorage,
                instruction: None,
            });
        }
        return Ok(uses);
    }
    Ok(function
        .body
        .iter()
        .enumerate()
        .flat_map(|(instruction, op)| {
            instruction_services(op)
                .iter()
                .map(move |service| ServiceUse {
                    service: *service,
                    instruction: Some(instruction),
                })
        })
        .collect())
}

fn instruction_services(op: &Op) -> &'static [RuntimeService] {
    use RuntimeService::*;
    // Exhaustive so additions to IL require an explicit service classification.
    match op {
        Op::BindDelegate { .. } => &[ValueStorage, SlotReferences],
        Op::NewArray(_) | Op::AllocateArray(_) => &[ManagedArrays, ManagedHeap, SlotReferences],
        Op::CreateArray(_) => &[ManagedArrays],
        Op::ArrayLength | Op::ArrayElement(_) | Op::StoreArrayElement(_) | Op::ArrayAddress(_) => {
            &[ManagedArrays, SlotReferences]
        }
        Op::Allocate(..) | Op::Free => &[NativeAllocation, PointerMemory],
        Op::AllocateLocal => &[FrameAllocation, PointerMemory],
        Op::PackValue(..) | Op::IsValue(..) | Op::UnpackValue(..) => &[ValueStorage],
        Op::LoadTypeToken(..) => &[TypeInspection],
        Op::ReferenceType => &[TypeInspection, SlotReferences],
        Op::CastClass(_) => &[TypeInspection, SlotReferences],
        Op::ReferenceEqual => &[SlotReferences],
        Op::LocalAddress(..) | Op::ArgumentAddress(..) => &[SlotReferences],
        Op::LoadObject(..)
        | Op::StoreObject(..)
        | Op::FieldAddress(..)
        | Op::InitializeObject(..) => &[PointerMemory, SlotReferences],
        Op::BorrowInterface(..) | Op::CallVirtual(..) => &[InterfaceDispatch, SlotReferences],
        Op::PointerFromInt(..)
        | Op::PointerAdd
        | Op::CopyObject(..)
        | Op::CopyBlock
        | Op::InitializeBlock
        | Op::LoadIndirectInt8
        | Op::LoadIndirectUInt8
        | Op::LoadIndirectInt16
        | Op::LoadIndirectUInt16
        | Op::LoadIndirectUInt32
        | Op::LoadIndirectInt64
        | Op::LoadIndirectNative
        | Op::StoreIndirectInt8
        | Op::StoreIndirectInt16
        | Op::StoreIndirectInt64
        | Op::StoreIndirectNative
        | Op::LoadIndirectFloat32
        | Op::LoadIndirectFloat64
        | Op::StoreIndirectFloat32
        | Op::StoreIndirectFloat64
        | Op::LoadIndirectInt32
        | Op::StoreIndirectInt32 => &[PointerMemory],
        Op::HeapNew => &[ManagedHeap, SlotReferences],
        Op::Unaligned(..)
        | Op::Int(..)
        | Op::Int64(..)
        | Op::CheckedInt8
        | Op::CheckedUInt8
        | Op::CheckedInt16
        | Op::CheckedUInt16
        | Op::CheckedInt32
        | Op::CheckedUInt32
        | Op::CheckedInt64
        | Op::CheckedUInt64
        | Op::CheckedNativeInt
        | Op::CheckedNativeUInt
        | Op::CheckedInt8Unsigned
        | Op::CheckedUInt8Unsigned
        | Op::CheckedInt16Unsigned
        | Op::CheckedUInt16Unsigned
        | Op::CheckedInt32Unsigned
        | Op::CheckedUInt32Unsigned
        | Op::CheckedInt64Unsigned
        | Op::CheckedUInt64Unsigned
        | Op::CheckedNativeIntUnsigned
        | Op::CheckedNativeUIntUnsigned
        | Op::ConvertInt8
        | Op::ConvertUInt8
        | Op::ConvertInt16
        | Op::ConvertUInt16
        | Op::ConvertUInt32
        | Op::ConvertInt64
        | Op::ConvertUInt64
        | Op::Float32 { .. }
        | Op::Float64 { .. }
        | Op::ConvertFloat32
        | Op::ConvertFloat64
        | Op::ConvertFloatUnsigned
        | Op::CheckFinite
        | Op::Bool(..)
        | Op::String(..)
        | Op::Void
        | Op::Arg(..)
        | Op::StoreArg(..)
        | Op::Load(..)
        | Op::ResetLocal(..)
        | Op::Store(..)
        | Op::Dup
        | Op::Pop
        | Op::BitAnd
        | Op::BitOr
        | Op::BitXor
        | Op::BitNot
        | Op::Negate
        | Op::ShiftLeft
        | Op::ShiftRight
        | Op::ShiftRightUnsigned
        | Op::Remainder
        | Op::RemainderUnsigned
        | Op::Add
        | Op::Sub
        | Op::Mul
        | Op::AddChecked
        | Op::SubChecked
        | Op::MulChecked
        | Op::Divide
        | Op::AddCheckedUnsigned
        | Op::SubCheckedUnsigned
        | Op::MulCheckedUnsigned
        | Op::DivideUnsigned
        | Op::LessUnsigned
        | Op::ConvertNativeInt
        | Op::ConvertNativeUInt
        | Op::ConvertInt32
        | Op::Equal
        | Op::Greater
        | Op::GreaterUnsigned
        | Op::Less
        | Op::Branch(..)
        | Op::BranchTrue(..)
        | Op::BranchFalse(..)
        | Op::BranchEqual(..)
        | Op::BranchNotEqual(..)
        | Op::BranchGreater(..)
        | Op::BranchGreaterUnsigned(..)
        | Op::BranchLess(..)
        | Op::BranchLessUnsigned(..)
        | Op::BranchGreaterEqual(..)
        | Op::BranchGreaterEqualUnsigned(..)
        | Op::BranchLessEqual(..)
        | Op::BranchLessEqualUnsigned(..)
        | Op::Switch(..)
        | Op::Call(..)
        | Op::Construct(..)
        | Op::Return
        | Op::New(..)
        | Op::Field(..)
        | Op::SetField(..)
        | Op::SizeOf(..)
        | Op::AlignOf(..)
        | Op::NullPointer(..)
        | Op::PointerCast(..)
        | Op::Error(..)
        | Op::Fault(..) => &[],
    }
}
